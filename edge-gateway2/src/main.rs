use esp_idf_svc::hal as hal;
use hal::{
    gpio::*,
    ledc::*,
    peripherals::Peripherals,
    prelude::*,
    uart::*,
};
use hal::uart::config::Config as UartConfig;
use esp_idf_svc::hal::ledc::config::{TimerConfig, Resolution};
use esp_idf_svc::hal::units::Hertz;

use rmodbus::{client::ModbusRequest, ModbusProto};
use serde::Serialize;
use std::{
    thread,
    time::{Duration, Instant},
};

#[derive(Serialize)]
struct Sample {
    temp_c: f32,
    hum_rh: f32,
}

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let p = Peripherals::take().unwrap();

    // ================= MAX485 / SHT20 =================
    let mut de_re = PinDriver::output(p.pins.gpio4)?; // DE/RE MAX485
    let config = UartConfig::default().baudrate(Hertz(9600));
    let uart = UartDriver::new(
        p.uart1,
        p.pins.gpio18, // TX → DI MAX485
        p.pins.gpio17, // RX ← RO MAX485
        Option::<AnyIOPin>::None,
        Option::<AnyIOPin>::None,
        &config,
    )?;

    // ================= L9110 FAN =================
    let mut fan_in_a = PinDriver::output(p.pins.gpio15)?; // IN A
    let mut fan_in_b = PinDriver::output(p.pins.gpio16)?; // IN B
    fan_in_a.set_low()?;
    fan_in_b.set_low()?;

    // ================= SERVO (GPIO40) =================
    let timer = LedcTimerDriver::new(
        p.ledc.timer0,
        &TimerConfig {
            frequency: 50.Hz(),
            resolution: Resolution::Bits13,
            ..Default::default()
        },
    )?;

    let mut pwm_servo = LedcDriver::new(p.ledc.channel0, &timer, p.pins.gpio40)?;
    pwm_servo.set_duty(servo_duty(&pwm_servo, 0.0))?;
    log::info!("🤖 Servo initialized at 0° (atap tertutup)");

    // ================= LOOP =================
    log::info!("🚀 Smart Mushroom House Control Started");

    let mut fan_on_until: Option<Instant> = None;
    let mut servo_open = false;

    loop {
        if let (Some(t), Some(h)) = (
            read_input_register(&uart, &mut de_re, 1, 0x0001),
            read_input_register(&uart, &mut de_re, 1, 0x0002),
        ) {
            let sample = Sample { temp_c: t, hum_rh: h };
            println!("{}", serde_json::to_string(&sample).unwrap());
            log::info!("✅ Read OK: temp={:.1}°C hum={:.1}%", t, h);

            // --- KONTROL SUHU ---
            if t > 26.5 {
                // Nyalakan kipas
                if fan_on_until.is_none() {
                    fan_in_a.set_low()?;
                    fan_in_b.set_high()?;
                    fan_on_until = Some(Instant::now() + Duration::from_secs(30));
                    log::info!("🔥 Suhu {:.1}°C > 26.5°C → Kipas ON (pendinginan)", t);
                }
            }

            // Matikan kipas jika waktu ON habis
            if let Some(end_time) = fan_on_until {
                if Instant::now() >= end_time {
                    fan_in_a.set_low()?;
                    fan_in_b.set_low()?;
                    fan_on_until = None;
                    log::info!("🧊 Kipas OFF setelah pendinginan 30 detik");
                }
            }

            // --- KONTROL KELEMBAPAN (Servo) ---
            if h < 70.0 && !servo_open {
                pwm_servo.set_duty(servo_duty(&pwm_servo, 120.0))?;
                servo_open = true;
                log::info!("💧 Humidity {:.1}% < 70% → Atap dibuka (120°)", h);
            } else if h > 72.0 && servo_open {
                pwm_servo.set_duty(servo_duty(&pwm_servo, 0.0))?;
                servo_open = false;
                log::info!("☔ Humidity {:.1}% > 72% → Atap ditutup (0°)", h);
            }

        } else {
            log::warn!("⚠️ Gagal baca data sensor — coba lagi 30 detik");
        }

        thread::sleep(Duration::from_secs(30));
    }
}

/// Fungsi baca 1 register input (function 0x04)
fn read_input_register(
    uart: &UartDriver,
    de_re: &mut PinDriver<'_, Gpio4, Output>,
    unit_id: u8,
    register: u16,
) -> Option<f32> {
    let mut mreq = ModbusRequest::new(unit_id, ModbusProto::Rtu);
    let mut txbuf: Vec<u8> = Vec::with_capacity(256);
    if mreq.generate_get_inputs(register, 1, &mut txbuf).is_err() {
        log::error!("❌ generate_get_inputs failed for 0x{:04X}", register);
        return None;
    }

    let _ = de_re.set_high();
    let _ = uart.write(&txbuf);
    let _ = uart.wait_tx_done(100);
    let _ = de_re.set_low();

    let mut rxbuf = vec![0u8; 512];
    let n = match uart.read(&mut rxbuf, 500) {
        Ok(n) if n > 0 => n,
        _ => return None,
    };

    let mut vals = Vec::new();
    if mreq.parse_u16(&rxbuf[..n], &mut vals).is_ok() && !vals.is_empty() {
        Some(vals[0] as f32 / 10.0)
    } else {
        None
    }
}

/// Konversi sudut servo (0–180°) ke duty cycle PWM (50 Hz)
fn servo_duty(pwm: &LedcDriver, angle: f32) -> u32 {
    let duty = 0.025 + (angle / 180.0) * 0.1; // 0.5–2.5 ms
    (duty * pwm.get_max_duty() as f32) as u32
}
