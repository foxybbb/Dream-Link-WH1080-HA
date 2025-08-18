use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MqttConfig {
    pub broker: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub topic_prefix: String,
    pub client_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherStationConfig {
    pub vendor_id: u16,
    pub product_id: u16,
    pub period_minutes: u64,
    pub max_rain_jump: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub weather_station: WeatherStationConfig,
    pub mqtt: MqttConfig,
    pub language: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            weather_station: WeatherStationConfig {
                vendor_id: 0x1941,
                product_id: 0x8021,
                period_minutes: 1,
                max_rain_jump: 10.0,
            },
            mqtt: MqttConfig {
                broker: "192.168.8.111".to_string(),
                port: 1883,
                username: "homeassistant".to_string(),
                password: "3333".to_string(),
                topic_prefix: "homeassistant/sensor/0x19418021".to_string(),
                client_id: "weather_station_rust".to_string(),
            },
            language: "en".to_string(),
        }
    }
}

impl AppConfig {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let contents = fs::read_to_string(path)
            .map_err(|e| anyhow!("Failed to read config file: {}", e))?;
        
        let config: AppConfig = toml::from_str(&contents)
            .map_err(|e| anyhow!("Failed to parse config file: {}", e))?;
        
        Ok(config)
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let contents = toml::to_string_pretty(self)
            .map_err(|e| anyhow!("Failed to serialize config: {}", e))?;
        
        fs::write(path, contents)
            .map_err(|e| anyhow!("Failed to write config file: {}", e))?;
        
        Ok(())
    }

    pub fn load_or_create_default<P: AsRef<Path>>(path: P) -> Result<Self> {
        if path.as_ref().exists() {
            Self::load_from_file(path)
        } else {
            let config = Self::default();
            config.save_to_file(&path)?;
            println!("Created default config file at: {}", path.as_ref().display());
            Ok(config)
        }
    }
}
