use anyhow::{anyhow, Result};
use rumqttc::{AsyncClient, MqttOptions, QoS};
use serde_json::{json, Value};
use std::time::Duration;
use tokio::time::sleep;
use chrono::Local;

use crate::config::MqttConfig;
use crate::translations::Translations;
use crate::weather::WeatherData;

#[derive(Debug, Clone)]
pub enum DeviceStatus {
    Online,
    Offline,
    Error(String),
    UsbNotFound,
    PermissionDenied,
    ReadError(String),
}

pub struct MqttClient {
    client: AsyncClient,
    config: MqttConfig,
    translations: Translations,
}

impl MqttClient {
    pub async fn new(config: MqttConfig) -> Result<Self> {
        let mut mqttoptions = MqttOptions::new(&config.client_id, &config.broker, config.port);
        mqttoptions.set_credentials(&config.username, &config.password);
        mqttoptions.set_keep_alive(Duration::from_secs(60));

        let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
        
        // Start the eventloop in a background task
        tokio::spawn(async move {
            loop {
                match eventloop.poll().await {
                    Ok(_) => {},
                    Err(e) => {
                        eprintln!("MQTT connection error: {}", e);
                        sleep(Duration::from_secs(5)).await;
                    }
                }
            }
        });

        // Wait a bit for connection to establish
        sleep(Duration::from_millis(1000)).await;

        Ok(Self {
            client,
            config,
            translations: Translations::new(),
        })
    }

    pub async fn publish_weather_data(&self, data: &WeatherData, language: &str) -> Result<()> {
        let device_info = self.create_device_info();
        
        let sensors = self.create_sensor_definitions(data, language, &device_info);

        for (sensor_key, attributes) in sensors {
            let config_topic = format!("{}/{}/config", self.config.topic_prefix, sensor_key);
            let state_topic = format!("{}/{}/state", self.config.topic_prefix, sensor_key);

            // Publish configuration
            let config_payload = serde_json::to_string(&attributes)
                .map_err(|e| anyhow!("Failed to serialize config: {}", e))?;

            self.client
                .publish(&config_topic, QoS::AtLeastOnce, true, config_payload)
                .await
                .map_err(|e| anyhow!("Failed to publish config: {}", e))?;

            // Publish state
            let state_value = attributes["value"].clone();
            let state_payload = match state_value {
                Value::String(s) => s,
                Value::Number(n) => n.to_string(),
                _ => state_value.to_string(),
            };

            self.client
                .publish(&state_topic, QoS::AtLeastOnce, true, state_payload)
                .await
                .map_err(|e| anyhow!("Failed to publish state: {}", e))?;

            println!("Published sensor '{}' to MQTT", sensor_key);
        }

        // Update device status to online
        self.publish_device_status(DeviceStatus::Online, language).await?;

        Ok(())
    }

    pub async fn publish_device_status(&self, status: DeviceStatus, language: &str) -> Result<()> {
        let device_info = self.create_device_info();
        
        // Create status sensor configuration
        let status_config = self.create_status_sensor_config(&device_info, language);
        let config_topic = format!("{}/device_status/config", self.config.topic_prefix);
        let state_topic = format!("{}/device_status/state", self.config.topic_prefix);
        
        // Publish status sensor configuration
        let config_payload = serde_json::to_string(&status_config)
            .map_err(|e| anyhow!("Failed to serialize status config: {}", e))?;

        self.client
            .publish(&config_topic, QoS::AtLeastOnce, true, config_payload)
            .await
            .map_err(|e| anyhow!("Failed to publish status config: {}", e))?;

        // Publish status state
        let status_text = self.format_device_status(&status, language);
        self.client
            .publish(&state_topic, QoS::AtLeastOnce, true, status_text.clone())
            .await
            .map_err(|e| anyhow!("Failed to publish status: {}", e))?;

        // Also publish to a diagnostic topic for logging
        let diagnostic_topic = format!("{}/diagnostics", self.config.topic_prefix);
        let diagnostic_payload = json!({
            "timestamp": Local::now().to_rfc3339(),
            "status": status_text,
            "details": self.get_status_details(&status)
        });

        self.client
            .publish(
                &diagnostic_topic, 
                QoS::AtLeastOnce, 
                false, 
                serde_json::to_string(&diagnostic_payload)?
            )
            .await
            .map_err(|e| anyhow!("Failed to publish diagnostics: {}", e))?;

        println!("Published device status: {}", status_text);
        Ok(())
    }

    pub async fn publish_error(&self, error_message: &str, language: &str) -> Result<()> {
        // Determine status type based on error message
        let status = if error_message.contains("not found") || error_message.contains("Device not found") {
            DeviceStatus::UsbNotFound
        } else if error_message.contains("permission") || error_message.contains("Access denied") {
            DeviceStatus::PermissionDenied
        } else if error_message.contains("read") || error_message.contains("Bad data") {
            DeviceStatus::ReadError(error_message.to_string())
        } else {
            DeviceStatus::Error(error_message.to_string())
        };

        self.publish_device_status(status, language).await
    }

    pub async fn publish_heartbeat(&self, _language: &str) -> Result<()> {
        let heartbeat_topic = format!("{}/heartbeat", self.config.topic_prefix);
        let heartbeat_payload = json!({
            "timestamp": Local::now().to_rfc3339(),
            "status": "alive",
            "uptime": std::process::id()
        });

        self.client
            .publish(
                &heartbeat_topic,
                QoS::AtLeastOnce,
                false,
                serde_json::to_string(&heartbeat_payload)?
            )
            .await
            .map_err(|e| anyhow!("Failed to publish heartbeat: {}", e))?;

        Ok(())
    }

    fn create_device_info(&self) -> Value {
        json!({
            "name": "Weather Station",
            "identifiers": ["weather_station_001"],
            "manufacturer": "Dream-Link",
            "model": "WH1080 USB Weather Station"
        })
    }

    fn create_sensor_definitions(&self, data: &WeatherData, language: &str, device_info: &Value) -> Vec<(String, Value)> {
        let mut sensors = Vec::new();

        // Indoor humidity
        sensors.push((
            "indoor_humidity".to_string(),
            json!({
                "name": self.translations.get(language, "indoor_humidity"),
                "unit_of_measurement": "%",
                "device_class": "humidity",
                "unique_id": "indoor_humidity_sensor",
                "state_topic": format!("{}/indoor_humidity/state", self.config.topic_prefix),
                "device": device_info,
                "value": data.indoor_humidity
            })
        ));

        // Outdoor humidity
        sensors.push((
            "outdoor_humidity".to_string(),
            json!({
                "name": self.translations.get(language, "outdoor_humidity"),
                "unit_of_measurement": "%",
                "device_class": "humidity",
                "unique_id": "outdoor_humidity_sensor",
                "state_topic": format!("{}/outdoor_humidity/state", self.config.topic_prefix),
                "device": device_info,
                "value": data.outdoor_humidity
            })
        ));

        // Indoor temperature
        sensors.push((
            "indoor_temperature".to_string(),
            json!({
                "name": self.translations.get(language, "indoor_temperature"),
                "unit_of_measurement": "°C",
                "device_class": "temperature",
                "unique_id": "indoor_temperature_sensor",
                "state_topic": format!("{}/indoor_temperature/state", self.config.topic_prefix),
                "device": device_info,
                "value": (data.indoor_temperature * 100.0).round() / 100.0
            })
        ));

        // Outdoor temperature
        sensors.push((
            "outdoor_temperature".to_string(),
            json!({
                "name": self.translations.get(language, "outdoor_temperature"),
                "unit_of_measurement": "°C",
                "device_class": "temperature",
                "unique_id": "outdoor_temperature_sensor",
                "state_topic": format!("{}/outdoor_temperature/state", self.config.topic_prefix),
                "device": device_info,
                "value": (data.outdoor_temperature * 100.0).round() / 100.0
            })
        ));

        // Outdoor dew point
        sensors.push((
            "outdoor_dew_point".to_string(),
            json!({
                "name": self.translations.get(language, "outdoor_dew_point"),
                "unit_of_measurement": "°C",
                "device_class": "temperature",
                "unique_id": "outdoor_dew_point_sensor",
                "state_topic": format!("{}/outdoor_dew_point/state", self.config.topic_prefix),
                "device": device_info,
                "value": (data.outdoor_dew_point * 100.0).round() / 100.0
            })
        ));

        // Wind chill temperature
        sensors.push((
            "wind_chill_temp".to_string(),
            json!({
                "name": self.translations.get(language, "wind_chill_temp"),
                "unit_of_measurement": "°C",
                "device_class": "temperature",
                "unique_id": "wind_chill_sensor",
                "state_topic": format!("{}/wind_chill_temp/state", self.config.topic_prefix),
                "device": device_info,
                "value": (data.wind_chill_temp * 100.0).round() / 100.0
            })
        ));

        // Wind speed (convert from m/s to km/h like in Python code)
        sensors.push((
            "wind_speed".to_string(),
            json!({
                "name": self.translations.get(language, "wind_speed"),
                "unit_of_measurement": "km/h",
                "device_class": "speed",
                "unique_id": "wind_speed_sensor",
                "state_topic": format!("{}/wind_speed/state", self.config.topic_prefix),
                "device": device_info,
                "value": (data.wind_speed * 3.6 * 100.0).round() / 100.0  // Convert m/s to km/h
            })
        ));

        // Gust speed (convert from m/s to km/h)
        sensors.push((
            "gust_speed".to_string(),
            json!({
                "name": self.translations.get(language, "gust_speed"),
                "unit_of_measurement": "km/h",
                "device_class": "speed",
                "unique_id": "gust_speed_sensor",
                "state_topic": format!("{}/gust_speed/state", self.config.topic_prefix),
                "device": device_info,
                "value": (data.gust_speed * 3.6 * 100.0).round() / 100.0  // Convert m/s to km/h
            })
        ));

        // Wind direction
        sensors.push((
            "wind_direction".to_string(),
            json!({
                "name": self.translations.get(language, "wind_direction"),
                "icon": "mdi:compass",
                "unique_id": "wind_direction_sensor",
                "state_topic": format!("{}/wind_direction/state", self.config.topic_prefix),
                "device": device_info,
                "value": self.translations.get_wind_direction(language, data.wind_direction_index)
            })
        ));

        // Rain difference
        sensors.push((
            "rain_diff".to_string(),
            json!({
                "name": self.translations.get(language, "rain_diff"),
                "unit_of_measurement": "mm",
                "device_class": "precipitation",
                "unique_id": "rainfall_sensor",
                "state_topic": format!("{}/rain_diff/state", self.config.topic_prefix),
                "device": device_info,
                "value": (data.rain_diff * 100.0).round() / 100.0
            })
        ));

        // Total rain
        sensors.push((
            "total_rain".to_string(),
            json!({
                "name": self.translations.get(language, "total_rain"),
                "unit_of_measurement": "mm",
                "device_class": "precipitation",
                "unique_id": "total_rain_sensor",
                "state_topic": format!("{}/total_rain/state", self.config.topic_prefix),
                "device": device_info,
                "value": (data.total_rain * 100.0).round() / 100.0
            })
        ));

        // Absolute pressure
        sensors.push((
            "abs_pressure".to_string(),
            json!({
                "name": self.translations.get(language, "abs_pressure"),
                "unit_of_measurement": "hPa",
                "device_class": "pressure",
                "unique_id": "pressure_sensor",
                "state_topic": format!("{}/abs_pressure/state", self.config.topic_prefix),
                "device": device_info,
                "value": (data.abs_pressure * 100.0).round() / 100.0
            })
        ));

        sensors
    }

    fn create_status_sensor_config(&self, device_info: &Value, language: &str) -> Value {
        json!({
            "name": if language == "ru" { "Статус устройства" } else { "Device Status" },
            "icon": "mdi:weather-station",
            "unique_id": "weather_station_status",
            "state_topic": format!("{}/device_status/state", self.config.topic_prefix),
            "device": device_info,
            "availability": {
                "topic": format!("{}/heartbeat", self.config.topic_prefix),
                "payload_available": "alive",
                "payload_not_available": "offline"
            }
        })
    }

    fn format_device_status(&self, status: &DeviceStatus, language: &str) -> String {
        match status {
            DeviceStatus::Online => {
                if language == "ru" { "В сети" } else { "Online" }.to_string()
            },
            DeviceStatus::Offline => {
                if language == "ru" { "Отключено" } else { "Offline" }.to_string()
            },
            DeviceStatus::UsbNotFound => {
                if language == "ru" { "USB устройство не найдено" } else { "USB device not found" }.to_string()
            },
            DeviceStatus::PermissionDenied => {
                if language == "ru" { "Нет доступа к USB" } else { "USB permission denied" }.to_string()
            },
            DeviceStatus::ReadError(msg) => {
                if language == "ru" { 
                    format!("Ошибка чтения: {}", msg)
                } else { 
                    format!("Read error: {}", msg)
                }
            },
            DeviceStatus::Error(msg) => {
                if language == "ru" { 
                    format!("Ошибка: {}", msg)
                } else { 
                    format!("Error: {}", msg)
                }
            }
        }
    }

    fn get_status_details(&self, status: &DeviceStatus) -> Value {
        match status {
            DeviceStatus::Online => json!({
                "code": "online",
                "severity": "info",
                "description": "Device is functioning normally"
            }),
            DeviceStatus::Offline => json!({
                "code": "offline",
                "severity": "warning", 
                "description": "Device is not responding"
            }),
            DeviceStatus::UsbNotFound => json!({
                "code": "usb_not_found",
                "severity": "error",
                "description": "USB weather station device not detected",
                "troubleshooting": [
                    "Check USB connection",
                    "Verify device vendor/product ID (1941:8021)",
                    "Try different USB port"
                ]
            }),
            DeviceStatus::PermissionDenied => json!({
                "code": "permission_denied", 
                "severity": "error",
                "description": "No permission to access USB device",
                "troubleshooting": [
                    "Check udev rules in /etc/udev/rules.d/",
                    "Add user to plugdev group",
                    "Run with elevated privileges (not recommended)"
                ]
            }),
            DeviceStatus::ReadError(msg) => json!({
                "code": "read_error",
                "severity": "error", 
                "description": "Failed to read data from device",
                "error_message": msg,
                "troubleshooting": [
                    "Check USB connection stability",
                    "Verify device is not in use by another process",
                    "Try reconnecting the device"
                ]
            }),
            DeviceStatus::Error(msg) => json!({
                "code": "general_error",
                "severity": "error",
                "description": "General device error", 
                "error_message": msg
            })
        }
    }
}
