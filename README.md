# Rust Project (Implementation of Distributes Control System (DCS) for Temperature and Humidity Control on Smartplan Mushroom Cultivation)

## System
edge-gateway2/     → Program utama di ESP32-S3 (kontrol fan & servo)
cloud_mqtt/        → Program di PC/Server (MQTT + InfluxDB + ThingsBoard)


## 📘 Deskripsi Proyek  
Proyek ini merupakan implementasi Distributed Control System (DCS) berbasis IoT menggunakan Rust, ESP32-S3, dan sensor SHT20 untuk sistem budidaya jamur otomatis (Smartplan Mushroom Cultivation).

Tujuan utama sistem ini adalah untuk mengontrol suhu dan kelembapan ruang tanam jamur secara otomatis menggunakan kombinasi sensor, aktuator, dan komunikasi MQTT menuju cloud untuk monitoring dan analisis data.

Sistem bekerja secara terdistribusi antara perangkat Edge (ESP32-S3) dan Cloud (Server):

Jika kelembapan udara menurun, sistem akan mengaktifkan humidifier atau menyemprot air untuk menjaga kelembapan optimal bagi pertumbuhan jamur.

Jika suhu terlalu tinggi, maka kipas dinyalakan untuk menurunkan suhu ruangan.

Semua data sensor dikirim ke ThingsBoard untuk monitoring dan disimpan di InfluxDB untuk keperluan analisis historis.

Arsitektur sistem dibagi menjadi dua bagian utama:  

- **Edge (ESP32-S3)**  
  Berfungsi sebagai pengendali utama, membaca data dari sensor SHT20, dan mengontrol kipas (L9110) serta servo (SG90).  
  Data kemudian dikirim ke PC/server melalui protokol **MQTT**.  

- **Cloud (PC/Server)**  
  Bertugas menerima data dari Edge, meneruskan ke **ThingsBoard** untuk visualisasi, serta menyimpan data ke **InfluxDB** untuk analisis historis.  

## Teknologi yang digunakan :
- Sensor SHT20
- Kipas L9110
- Motor Servo SG90
- Modbus MAX485
- ESP32 S3
- RS485 to USB untuk PSU dari laptop


## 💡 EDGE: `edge-gateway2` (ESP32-S3)

### 🎯 Fungsi Utama  
Modul ini dijalankan pada **ESP32-S3** menggunakan framework **Rust esp-idf**, dan berfungsi sebagai sistem kontrol otomatis yang membaca data sensor serta mengontrol aktuator (fan dan servo).

### 🔧 Komponen Terkait  

| Komponen | Fungsi |
|-----------|--------|
| **SHT20 (MD-XY02)** | Sensor suhu dan kelembapan berbasis Modbus RS485 |
| **MAX485** | Modul konverter RS485 ke UART |
| **Driver L9110** | Mengendalikan motor kipas DC |
| **Servo SG90** | Membuka/menutup atap jemuran |
| **ESP32-S3** | Otak utama sistem |

### 🔁 Alur Kerja  
1. ESP32-S3 membaca suhu (`temp_c`) dan kelembapan (`hum_rh`) dari sensor SHT20 melalui **Modbus RTU (RS485)**.  
2. Berdasarkan nilai sensor:  
   - Jika **kelembapan < ambang batas**, servo membuka atap.  
   - Jika **kelembapan tinggi**, servo menutup atap & kipas menyala.  
3. Data dikirim ke Cloud melalui **MQTT** untuk disimpan & divisualisasikan.

---

## ☁️ CLOUD: `cloud_mqtt` (PC/Server)

### 🎯 Fungsi Utama  
Modul ini berjalan di PC atau server, berfungsi untuk menerima data dari ESP32-S3 melalui **MQTT**, kemudian:
- Menyimpan data ke **InfluxDB**
- Mengirim data ke **ThingsBoard** untuk visualisasi dashboard
- Memonitor performa sistem secara real-time

### 🔧 Komponen Terkait  

| Komponen | Fungsi |
|-----------|--------|
| **MQTT Broker (Mosquitto)** | Jalur komunikasi antar perangkat |
| **InfluxDB** | Penyimpanan data historis suhu & kelembapan |
| **ThingsBoard** | Platform dashboard untuk visualisasi data IoT |
| **Rust Runtime (tokio, rumqttc, reqwest)** | Framework pemrograman asinkron untuk server |

---

## 🚀 Cara Menjalankan  

### 🔹 Jalankan di cloud_mqtt (Thingsboard & Influxdb)  
```bash
cd cloud_mqtt
cargo run
sebelum itu nyalakan influxdb di terminal dengan command "influxd"

```
### 🔹 Jalankan di edge-gateway2 (ESP32-S3)  
```bash
cd edge-gateway2
cargo build --release
cargo espflash flash --port /dev/ttyACM0 --release --monitor

