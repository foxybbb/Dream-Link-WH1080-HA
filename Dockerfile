# Use an official Python runtime as a parent image
FROM python:3.9-slim

WORKDIR /app

COPY . /app

RUN pip install --no-cache-dir -r requirements.txt

EXPOSE 1883

# Define environment variables for MQTT broker settings
ENV MQTT_BROKER_HOST=192.168.8.111
ENV MQTT_BROKER_PORT=1883
ENV MQTT_USERNAME=homeassistant
ENV MQTT_PASSWORD=3333

# Run the script when the container launches
CMD ["python", "weather2mqtt.py"]
