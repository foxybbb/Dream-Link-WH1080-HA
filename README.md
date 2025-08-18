# Weather Station Data Reader (Rust Port with MQTT)

A Rust port of the Python weather station data reader for the **Dreamlink WH1080** weather station with **Home Assistant MQTT integration**.

## Features

- **USB Communication**: Connects to the weather station via USB (Vendor ID: 0x1941, Product ID: 0x8021)
- **Real-time Data**: Reads weather data every minute
- **MQTT Integration**: Publishes data to Home Assistant via MQTT with auto-discovery
- **Console Output**: Displays formatted weather data on screen
- **Multilingual Support**: English and Russian translations
- **Modular Configuration**: Separate config file for easy customization
- **Comprehensive Measurements**: 
  - Indoor/Outdoor temperature and humidity
  - Dew point calculation
  - Wind chill calculation
  - Wind speed, gusts, and direction
  - Rainfall measurements
  - Atmospheric pressure

## Prerequisites

### System Requirements
- libusb >= 1.0
- Rust >= 1.70

### USB Permissions (Linux)

You may need to set up udev rules for USB access. Create `/etc/udev/rules.d/41-weather-device.rules`:

```
SUBSYSTEM=="usb", ENV{DEVTYPE}=="usb_device", ATTR{idVendor}=="1941", ATTR{idProduct}=="8021", MODE="0666", OWNER="yourusername"
```

Or with group permissions:
```
SUBSYSTEM=="usb", ENV{DEVTYPE}=="usb_device", ATTR{idVendor}=="1941", ATTR{idProduct}=="8021", MODE="0666", GROUPS="yourgroup"
```

After creating the rules file:
1. Unplug the weather station
2. Run: `sudo udevadm control --reload-rules`
3. Plug the weather station back in

## Installation & Usage

### Method 1: Docker (Recommended for Raspberry Pi)

**Quick Start:**
```bash
# Clone the repository
git clone <repository-url>
cd weather-station

# Update config.toml with your MQTT settings
nano config.toml

# Build and run with Docker Compose
docker-compose up -d

# Check logs
docker-compose logs -f
```

**Manual Docker Commands:**
```bash
# Build for Raspberry Pi
./docker/build.sh

# Run the container
./docker/run.sh start -d

# Check status
./docker/run.sh status

# View logs
./docker/run.sh logs -f
```

### Method 2: Using the Build Script (For development)

1. **Use the provided build script**:
   ```bash
   cd weather-station
   ./build.sh
   ```

2. **Run the application**:
   ```bash
   ./target/release/weather-station
   ```

### Method 2: System Package Manager (Alternative)

If you encounter Rust proxy issues with Cursor, install Rust via your system package manager:

```bash
# On Arch/Manjaro:
sudo pacman -S rust cargo

# On Ubuntu/Debian:
sudo apt install rustc cargo

# On Fedora:
sudo dnf install rust cargo
```

Then build normally:
```bash
cargo build --release
cargo run
```

### Method 3: Standard Rust Installation

1. **Install Rust** (if not using Cursor):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   ```

2. **Build and run**:
   ```bash
   cargo build --release
   ./target/release/weather-station
   ```

## Sample Output

```
Weather Station Data Reader (Rust Port)
Model: Dreamlink WH1080
Looking for USB device...
Weather station connected successfully!
Data collection period: 1 minute(s)
Program started at: 2024-01-15 14:30:00

================================================
Weather Station Data - 2024-01-15 14:30:00
================================================
Indoor:
  Temperature: 22.3°C
  Humidity:    45%

Outdoor:
  Temperature: 15.7°C
  Humidity:    68%
  Dew Point:   9.85°C
  Wind Chill:  15.7°C

Wind:
  Speed:       2.1 m/s
  Gust:        3.8 m/s
  Direction:   SW

Rain:
  Since last:  0.0 mm
  Total:       12.3 mm

Pressure:
  Absolute:    1013.2 hPa
================================================
```

## Configuration

The application uses a `config.toml` file for settings. On first run, a default configuration file will be created:

```toml
[weather_station]
vendor_id = 0x1941
product_id = 0x8021
period_minutes = 1
max_rain_jump = 10.0

[mqtt]
broker = "192.168.8.111"
port = 1883
username = "homeassistant"
password = "3333"
topic_prefix = "homeassistant/sensor/0x19418021"
client_id = "weather_station_rust"

language = "en"  # "en" or "ru"
```

**Important**: Update the MQTT settings in `config.toml` before running!

## Docker Deployment for Raspberry Pi

The application includes full Docker support optimized for Raspberry Pi:

### Docker Features
- **Multi-Architecture**: Supports ARM64 (Pi 4/5) and ARM32 (Pi 3/Zero)
- **USB Device Access**: Privileged mode for weather station communication
- **Auto-Restart**: Container automatically restarts on failure
- **Health Checks**: Built-in container health monitoring
- **Volume Mapping**: Configuration and logs preserved across restarts

### Docker Files
- `Dockerfile` - Multi-stage build for efficient ARM images
- `docker-compose.yml` - Complete deployment configuration
- `docker/build.sh` - Build script with architecture detection
- `docker/run.sh` - Management script with status monitoring

## Dependencies

- **rusb**: USB device communication
- **tokio**: Async runtime
- **rumqttc**: MQTT client
- **chrono**: Date and time handling
- **libm**: Mathematical functions (log, pow)
- **anyhow**: Error handling
- **serde**: Serialization/deserialization
- **toml**: Configuration file parsing

## Key Features vs Python Version

- **MQTT Integration**: Full Home Assistant auto-discovery support
- **Modular Architecture**: Clean separation of concerns (config, translations, MQTT, weather)
- **Multilingual Support**: Built-in English and Russian translations
- **Configuration Management**: External TOML config file
- **Async Architecture**: Non-blocking MQTT operations
- **Error Handling**: Robust Rust error handling with detailed messages
- **Memory Safety**: Rust's ownership system prevents memory-related bugs
- **Performance**: Compiled binary for better performance
- **Type Safety**: Strong typing prevents runtime errors

## Home Assistant Integration

The application automatically creates Home Assistant sensors via MQTT discovery:

- **Device Information**: Appears as "Weather Station" in Home Assistant
- **Auto-Discovery**: Sensors are automatically created with proper device classes
- **Units**: Temperature (°C), Humidity (%), Pressure (hPa), Wind Speed (km/h), Rain (mm)
- **Translations**: Sensor names appear in your configured language
- **Icons**: Proper MDI icons for each sensor type

### Status Monitoring & Error Reporting

The application includes comprehensive monitoring and error reporting via MQTT:

#### Device Status Sensor
- **Sensor Name**: "Device Status" (English) / "Статус устройства" (Russian)
- **States**: 
  - `Online` - Device working normally
  - `USB device not found` - Weather station not detected
  - `USB permission denied` - Permission issues
  - `Read error: [details]` - Data reading problems
  - `Error: [details]` - Other errors

#### Diagnostic Information
- **Heartbeat**: Published every 10 minutes to `heartbeat` topic
- **Diagnostics**: Detailed error information with troubleshooting tips
- **Availability**: Device availability tracking for Home Assistant

#### Error Handling Features
- **Automatic Error Detection**: USB connection, permission, and data reading errors
- **MQTT Error Reporting**: All errors sent to Home Assistant for monitoring/alerting
- **Detailed Troubleshooting**: Error messages include specific troubleshooting steps
- **Recovery Monitoring**: Status automatically updates when device recovers

#### MQTT Topics Structure
```
homeassistant/sensor/0x19418021/
├── device_status/config          # Status sensor configuration
├── device_status/state           # Current device status
├── heartbeat                     # Periodic heartbeat (every 10 min)
├── diagnostics                   # Detailed error information
└── [sensor_name]/state           # Individual sensor data
```

## Troubleshooting

### Cursor Proxy Issues

If you see `error: unknown proxy name: 'cursor-bin'`:

1. **Use the build script**: `./build.sh` (recommended)
2. **Install system Rust**: `sudo pacman -S rust cargo` 
3. **Try clean environment**: 
   ```bash
   env -i PATH=/usr/bin:/bin /usr/bin/cargo build --release
   ```

### Device Not Found
- Check USB connection
- Verify udev rules are set up correctly
- Make sure the device vendor/product ID matches (0x1941/0x8021)

### Permission Denied
- Set up udev rules as described above
- Try running with sudo (not recommended for production)
- Check if your user is in the correct group

### Compilation Errors
- Ensure you have libusb development headers installed:
  - Ubuntu/Debian: `sudo apt install libusb-1.0-0-dev`
  - Fedora: `sudo dnf install libusb1-devel`
  - Arch: `sudo pacman -S libusb` 