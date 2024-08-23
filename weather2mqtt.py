import usb.core
import time
import struct
import math
import datetime
import pause
import paho.mqtt.client as mqtt
import json
import os
VENDOR = 0x1941
PRODUCT = 0x8021
WIND_DIRS = ['С', 'ССВ', 'СВ', 'ВСВ', 'В', 'ВЮВ', 'ЮВ', 'ЮЮВ', 'Ю', 'ЮЮЗ',
             'ЮЗ', 'ЗЮЗ', 'З', 'ЗСЗ', 'СЗ', 'ССЗ']
max_rain_jump = 10
previous_rain = 0
# interval for data collection
period = 1  # minutes

MQTT_BROKER = os.getenv('MQTT_BROKER_HOST')
MQTT_PORT = os.getenv('MQTT_BROKER_PORT')
#MQTT_TOPIC = "test/0x0x19418021"
MQTT_TOPIC= "homeassistant/sensor/0x19418021"

client = mqtt.Client(mqtt.CallbackAPIVersion.VERSION2)
client.username_pw_set(os.getenv('MQTT_USERNAME'),os.getenv('MQTT_PASSWORD')) # Change MQTT Login and Password
client.connect(MQTT_BROKER, MQTT_PORT, 60)

def open_ws():
    usb_device = usb.core.find(idVendor=VENDOR, idProduct=PRODUCT)

    if usb_device is None:
        raise ValueError('Device not found')

    usb_device.get_active_configuration()

    if usb_device.is_kernel_driver_active(0):
        usb_device.detach_kernel_driver(0)

    return usb_device


def read_block(device, offset):
    least_significant_bit = offset & 0xFF
    most_significant_bit = offset >> 8 & 0xFF

    tbuf = struct.pack('BBBBBBBB',
                       0xA1,
                       most_significant_bit,
                       least_significant_bit,
                       32,
                       0xA1,
                       most_significant_bit,
                       least_significant_bit,
                       32)

    timeout = 1000  # Milliseconds
    retval = dev.ctrl_transfer(0x21,  # USB Requesttype
                               0x09,  # USB Request
                               0x200,  # Value
                               0,  # Index
                               tbuf,  # Message
                               timeout)

    return dev.read(0x81, 32, timeout)


def dew_point(temperature, humidity):
    humidity /= 100.0
    gamma = (17.271 * temperature) / (237.7 + temperature) + math.log(humidity)
    return (237.7 * gamma) / (17.271 - gamma)


def wind_chill(temperature, wind):
    wind_kph = 3.6 * wind

    if (wind_kph <= 4.8) or (temperature > 10.0):
        return temperature

    wct = 13.12 + (0.6215 * temperature) - \
        (11.37 * (wind_kph ** 0.16)) + \
        (0.3965 * temperature * (wind_kph ** 0.16))

    return min(wct, temperature)


dev = open_ws()
dev.set_configuration()

start_time = datetime.datetime.now() + datetime.timedelta(minutes=1)
start_time = start_time.replace(second=0, microsecond=0)
sampling_time = start_time
print('Program started at ' + str(start_time))
pause.until(start_time)

try:
    while True:
        sampling_time = sampling_time + datetime.timedelta(minutes=period)
        pause.until(sampling_time)
        
        now = str(datetime.datetime.now())

        fixed_block = read_block(dev, 0)

        if fixed_block[0] != 0x55:
            raise ValueError('Bad data returned')

        curpos = struct.unpack('H', fixed_block[30:32])[0]
        current_block = read_block(dev, curpos)

        indoor_humidity = current_block[1]
        tlsb = current_block[2]
        tmsb = current_block[3] & 0x7f
        tsign = current_block[3] >> 7
        indoor_temperature = (tmsb * 256 + tlsb) * 0.1
        if tsign:
            indoor_temperature *= -1

        outdoor_humidity = current_block[4]
        tlsb = current_block[5]
        tmsb = current_block[6] & 0x7f
        tsign = current_block[6] >> 7
        outdoor_temperature = (tmsb * 256 + tlsb) * 0.1
        if tsign:
            outdoor_temperature *= -1

        abs_pressure = struct.unpack('H', current_block[7:9])[0] * 0.1

        wind = current_block[9]
        gust = current_block[10]
        wind_extra = current_block[11]
        wind_dir = current_block[12]

        total_rain = struct.unpack('H', current_block[13:15])[0] * 0.3

        wind_speed = (wind + ((wind_extra & 0x0F) << 8)) * 0.38
        gust_speed = (gust + ((wind_extra & 0xF0) << 4)) * 0.38

        outdoor_dew_point = dew_point(outdoor_temperature, outdoor_humidity)
        wind_chill_temp = wind_chill(outdoor_temperature, wind_speed)

        if previous_rain == 0:
            previous_rain = total_rain

        rain_diff = total_rain - previous_rain

        if rain_diff > max_rain_jump:
            rain_diff = 0
            total_rain = previous_rain

        previous_rain = total_rain
            

        # Device information
        device_info = {
            "name": "Weather Station",
            "identifiers": ["weather_station_001"],
            "manufacturer": "Dream-Link",
            "model": "WH1080 USB Weather Station"
        }

        # Sample sensor data with default values
        sensors = {
            "indoor_humidity": {
                "name": "Влажность внутри",
                "unit_of_measurement": "%",
                "device_class": "humidity",
                "unique_id": "indoor_humidity_sensor",
                "value": indoor_humidity  
            },
            "outdoor_humidity": {
                "name": "Влажность снаружи",
                "unit_of_measurement": "%",
                "device_class": "humidity",
                "unique_id": "outdoor_humidity_sensor",
                "value": outdoor_humidity  
            },
            "indoor_temperature": {
                "name": "Температура внутри",
                "unit_of_measurement": "°C",
                "device_class": "temperature",
                "unique_id": "indoor_temperature_sensor",
                "value":  round(indoor_temperature, 2) 
            },
            "outdoor_temperature": {
                "name": "Температура снаружи",
                "unit_of_measurement": "°C",
                "device_class": "temperature",
                "unique_id": "outdoor_temperature_sensor",
                "value":  round(outdoor_temperature, 2)   
            },
            "outdoor_dew_point": {
                "name": "Точка росы снаружи",
                "unit_of_measurement": "°C",
                "device_class": "temperature",
                "unique_id": "outdoor_dew_point_sensor",
                "value": round(outdoor_dew_point, 2)  
            },
            "wind_chill_temp": {
                "name": "Ощущаемая температура",
                "unit_of_measurement": "°C",
                "device_class": "temperature",
                "unique_id": "wind_chill_sensor",
                "value": wind_chill_temp  
            },
            "wind_speed": {
                "name": "Скорость ветра",
                "unit_of_measurement": "км/ч",
                "device_class": "speed",
                "unique_id": "wind_speed_sensor",
                "value": wind_speed  
            },
            "gust_speed": {
                "name": "Скорость порывов ветра",
                "unit_of_measurement": "км/ч",
                "device_class": "speed",
                "unique_id": "gust_speed_sensor",
                "value": gust_speed  
            },
            "wind_dir": {
                "name": "Направление ветра",
                "icon": "mdi:compass",
                "unique_id": "wind_direction_sensor",
                "value": WIND_DIRS[wind_dir]  
            },
            "rain_diff": {
                "name": "Количество осадков",
                "unit_of_measurement": "мм",
                "device_class": "precipitation",
                "unique_id": "rainfall_sensor",
                "value": rain_diff  
            },
            "total_rain": {
                "name": "Общее количество осадков",
                "unit_of_measurement": "мм",
                "device_class": "precipitation",
                "unique_id": "total_rain_sensor",
                "value": total_rain 
            },
            "abs_pressure": {
                "name": "Давление",
                "unit_of_measurement": "гПа",
                "device_class": "pressure",
                "unique_id": "pressure_sensor",
                "value": abs_pressure  
            }
        }


        for sensor, attributes in sensors.items():
            config_topic = f"{MQTT_TOPIC}/{sensor}/config"
            state_topic = f"{MQTT_TOPIC}/{sensor}/state"
            
            # Add state topic and device information to attributes
            attributes["state_topic"] = state_topic
            attributes["device"] = device_info
            
            # Create the payload for the configuration
            config_payload = json.dumps(attributes)
            
            # Publish the configuration payload to the MQTT broker
            client.publish(config_topic, config_payload, retain=True)
            client.publish(state_topic, attributes["value"], retain=True)

            print(f"Published data to MQTT: {config_payload}")
        client.disconnect()
	
except (KeyboardInterrupt, SystemExit):
    print('\n...Program Stopped Manually!')
    client.disconnect()
