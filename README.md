# Weather Station MQTT Integration

This project provides a Python script to read weather data from a USB-connected weather station and publish it to an MQTT broker. The published data is automatically discovered by Home Assistant as a device with multiple sensors.
## Acknowledgements

Special thanks to [Shane Howearth](https://github.com/shaneHowearth) for the [Dream-Link-WH1080-Weather-Station](https://github.com/shaneHowearth/Dream-Link-WH1080-Weather-Station) project, which served as the foundation for this code. This project builds upon and extends the work done in that repository to integrate with Home Assistant via MQTT.

## Key Features
- Weather Station Integration: Connect to a USB weather station to collect environmental data.
- MQTT Publishing: Publish sensor data to an MQTT broker, making it available for smart home systems.
- Home Assistant Auto-Discovery: The script formats and publishes data in a way that Home Assistant automatically recognizes the weather station as a device, with individual sensors for temperature, humidity, wind speed, etc.
- Flexible Configuration: Easily customize the MQTT topics and sensor configurations.

## Sensors and Data Points

The following data points are read from the weather station and published:

- Indoor Humidity
- Outdoor Humidity
- Indoor Temperature
- Outdoor Temperature
- Dew Point
- Wind Chill Temperature
- Wind Speed
- Gust Speed
- Wind Direction
- Rainfall
- Total Rain
- Atmospheric Pressure

# Getting Started

## Clone the Repository

First, clone the repository to your local machine:

```bash
git clone https://github.com/yourusername/weather-station-mqtt.git
cd weather-station-mqtt
```

## Create a Virtual Environment

Create a virtual environment to manage your Python dependencies:
```bash
python3 -m venv venv
```
## Activate the Virtual Environment

On Windows:

```bash
venv\Scripts\activate
```

On macOS/Linux:

```bash
source venv/bin/activate
```
## Install the Required Packages

Install the necessary Python packages from the requirements.txt file:

```bash
pip install -r requirements.txt
```
## Running the Script
Before running the script it is necessary to change the data for mqtt and for authorization `<IP>`, `<<MQTT_USER>>`, `<MQTT_PASSWORD>`
### Testing

To send data via the mqtt protocol, test script can be used
```bash
python test/test.py
```

To run the weather station script, simply execute the following command:

```bash
python weather2mqtt.py
```




Home Assistant Integration

Once the data is published to the MQTT broker, Home Assistant will automatically recognize the weather station and create a new device with multiple sensors. These sensors will update in real-time based on the data collected from the weather station.
Example Use Case

This project is ideal for integrating a personal weather station into a Home Assistant setup, allowing for real-time monitoring and automation based on environmental conditions.