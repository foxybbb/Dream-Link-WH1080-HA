mod config;
mod mqtt_client;
mod translations;
mod weather;

use anyhow::Result;
use chrono::{Local, Timelike};
use std::time::Duration;
use tokio::time::sleep;

use crate::config::AppConfig;
use crate::mqtt_client::{MqttClient, DeviceStatus};
use crate::translations::Translations;
use crate::weather::WeatherStation;

async fn wait_until_next_minute(period_minutes: u64) {
    let now = Local::now();
    let next_minute = now
        .with_second(0)
        .unwrap()
        .with_nanosecond(0)
        .unwrap()
        + chrono::Duration::minutes(period_minutes as i64);
    
    let duration_until_next = (next_minute - now).to_std().unwrap_or(Duration::from_secs(1));
    
    println!("Waiting until {} for next reading...", next_minute.format("%H:%M:%S"));
    sleep(duration_until_next).await;
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("Weather Station Data Reader (Rust Port with MQTT)");
    println!("Model: Dreamlink WH1080");
    
    // Load configuration
    let config = AppConfig::load_or_create_default("config.toml")?;
    
    println!("Configuration loaded:");
    println!("  MQTT Broker: {}:{}", config.mqtt.broker, config.mqtt.port);
    println!("  Language: {}", config.language);
    println!("  Data collection period: {} minute(s)", config.weather_station.period_minutes);
    
    // Initialize translations
    let translations = Translations::new();
    
    println!("Looking for USB device...");
    
    // Initialize MQTT client first for error reporting
    let mqtt_client = MqttClient::new(config.mqtt.clone()).await?;
    println!("MQTT client connected successfully!");

    // Initialize weather station with error reporting
    let mut weather_station = match WeatherStation::new(config.weather_station.clone()) {
        Ok(station) => {
            println!("Weather station connected successfully!");
            // Report successful connection
            if let Err(e) = mqtt_client.publish_device_status(DeviceStatus::Online, &config.language).await {
                eprintln!("Failed to publish initial status: {}", e);
            }
            station
        }
        Err(e) => {
            eprintln!("Failed to initialize weather station: {}", e);
            // Report error via MQTT
            if let Err(mqtt_err) = mqtt_client.publish_error(&e.to_string(), &config.language).await {
                eprintln!("Failed to publish error status: {}", mqtt_err);
            }
            return Err(e);
        }
    };
    
    // Start at the next round minute
    let start_time = Local::now() + chrono::Duration::minutes(1);
    let start_time = start_time.with_second(0).unwrap().with_nanosecond(0).unwrap();
    println!("Program started at: {}", start_time.format("%Y-%m-%d %H:%M:%S"));
    
    wait_until_next_minute(config.weather_station.period_minutes).await;
    
    // Heartbeat counter for periodic status updates
    let mut heartbeat_counter = 0;
    let heartbeat_interval = 10; // Send heartbeat every 10 readings (10 minutes by default)
    
    loop {
        match weather_station.read_weather_data() {
            Ok(data) => {
                // Print data to console
                data.print(&config.language, &translations);
                
                // Publish to MQTT (this also updates status to Online)
                match mqtt_client.publish_weather_data(&data, &config.language).await {
                    Ok(()) => {
                        println!("Data published to MQTT successfully");
                    }
                    Err(e) => {
                        eprintln!("Failed to publish to MQTT: {}", e);
                        // Try to report MQTT error
                        if let Err(mqtt_err) = mqtt_client.publish_error(&format!("MQTT publish failed: {}", e), &config.language).await {
                            eprintln!("Failed to report MQTT error: {}", mqtt_err);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading weather data: {}", e);
                eprintln!("Retrying in {} minute(s)...", config.weather_station.period_minutes);
                
                // Report error via MQTT
                if let Err(mqtt_err) = mqtt_client.publish_error(&e.to_string(), &config.language).await {
                    eprintln!("Failed to publish error status: {}", mqtt_err);
                }
            }
        }
        
        // Send periodic heartbeat
        heartbeat_counter += 1;
        if heartbeat_counter >= heartbeat_interval {
            if let Err(e) = mqtt_client.publish_heartbeat(&config.language).await {
                eprintln!("Failed to publish heartbeat: {}", e);
            }
            heartbeat_counter = 0;
        }
        
        wait_until_next_minute(config.weather_station.period_minutes).await;
    }
} 