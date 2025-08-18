use anyhow::{anyhow, Result};
use chrono::{DateTime, Local};
use libm::{log, pow};
use rusb::{Device, DeviceHandle, GlobalContext};
use std::time::Duration;

use crate::config::WeatherStationConfig;

#[derive(Debug, Clone)]
pub struct WeatherData {
    pub timestamp: DateTime<Local>,
    pub indoor_humidity: u8,
    pub outdoor_humidity: u8,
    pub indoor_temperature: f32,
    pub outdoor_temperature: f32,
    pub outdoor_dew_point: f32,
    pub wind_chill_temp: f32,
    pub wind_speed: f32,
    pub gust_speed: f32,
    pub wind_direction_index: usize,
    pub rain_diff: f32,
    pub total_rain: f32,
    pub abs_pressure: f32,
}

pub struct WeatherStation {
    device_handle: DeviceHandle<GlobalContext>,
    previous_rain: f32,
    config: WeatherStationConfig,
}

impl WeatherStation {
    pub fn new(config: WeatherStationConfig) -> Result<Self> {
        let device = Self::find_weather_device(&config)?;
        let handle = device.open()?;
        
        // Detach kernel driver if active
        if handle.kernel_driver_active(0)? {
            handle.detach_kernel_driver(0)?;
        }
        
        handle.set_active_configuration(1)?;
        
        Ok(WeatherStation {
            device_handle: handle,
            previous_rain: 0.0,
            config,
        })
    }
    
    fn find_weather_device(config: &WeatherStationConfig) -> Result<Device<GlobalContext>> {
        let devices = rusb::devices()?;
        
        for device in devices.iter() {
            let device_desc = device.device_descriptor()?;
            if device_desc.vendor_id() == config.vendor_id && device_desc.product_id() == config.product_id {
                return Ok(device);
            }
        }
        
        Err(anyhow!("Weather station device not found (vendor: {:#x}, product: {:#x})", 
                   config.vendor_id, config.product_id))
    }
    
    fn read_block(&self, offset: u16) -> Result<[u8; 32]> {
        let least_significant_bit = (offset & 0xFF) as u8;
        let most_significant_bit = ((offset >> 8) & 0xFF) as u8;
        
        // Construct the binary message as in the Python code
        let tbuf = [
            0xA1,
            most_significant_bit,
            least_significant_bit,
            32,
            0xA1,
            most_significant_bit,
            least_significant_bit,
            32,
        ];
        
        let timeout = Duration::from_millis(1000);
        
        // Send control transfer
        self.device_handle.write_control(
            0x21,    // bmRequestType
            0x09,    // bRequest
            0x200,   // wValue
            0,       // wIndex
            &tbuf,
            timeout,
        )?;
        
        // Read response
        let mut buf = [0u8; 32];
        let bytes_read = self.device_handle.read_interrupt(0x81, &mut buf, timeout)?;
        
        if bytes_read != 32 {
            return Err(anyhow!("Expected 32 bytes, got {}", bytes_read));
        }
        
        Ok(buf)
    }
    
    pub fn read_weather_data(&mut self) -> Result<WeatherData> {
        // Get the first 32 bytes of the fixed block
        let fixed_block = self.read_block(0)?;
        
        // Check that we have good data
        if fixed_block[0] != 0x55 {
            return Err(anyhow!("Bad data returned, first byte: {:#x}", fixed_block[0]));
        }
        
        // Bytes 30 and 31 (0-indexed) when combined create an unsigned short
        // that tells us where to find the weather data we want
        let curpos = u16::from_le_bytes([fixed_block[30], fixed_block[31]]);
        let current_block = self.read_block(curpos)?;
        
        let timestamp = Local::now();
        
        // Indoor information
        let indoor_humidity = current_block[1];
        let tlsb = current_block[2] as u16;
        let tmsb = (current_block[3] & 0x7f) as u16;
        let tsign = (current_block[3] >> 7) != 0;
        let mut indoor_temperature = (tmsb * 256 + tlsb) as f32 * 0.1;
        if tsign {
            indoor_temperature *= -1.0;
        }
        
        // Outdoor information
        let outdoor_humidity = current_block[4];
        let tlsb = current_block[5] as u16;
        let tmsb = (current_block[6] & 0x7f) as u16;
        let tsign = (current_block[6] >> 7) != 0;
        let mut outdoor_temperature = (tmsb * 256 + tlsb) as f32 * 0.1;
        if tsign {
            outdoor_temperature *= -1.0;
        }
        
        // Absolute pressure from bytes 7 and 8
        let abs_pressure = u16::from_le_bytes([current_block[7], current_block[8]]) as f32 * 0.1;
        
        let wind = current_block[9] as u16;
        let gust = current_block[10] as u16;
        let wind_extra = current_block[11];
        let wind_dir = current_block[12] as usize;
        
        // Total rain from bytes 13 and 14
        let total_rain = u16::from_le_bytes([current_block[13], current_block[14]]) as f32 * 0.3;
        
        // Calculate wind speeds
        let wind_speed = (wind + ((wind_extra & 0x0F) as u16) << 8) as f32 * 0.38;
        let gust_speed = (gust + (((wind_extra & 0xF0) as u16) << 4)) as f32 * 0.38;
        
        let outdoor_dew_point = Self::dew_point(outdoor_temperature, outdoor_humidity as f32);
        let wind_chill_temp = Self::wind_chill(outdoor_temperature, wind_speed);
        
        // Calculate rainfall rates
        if self.previous_rain == 0.0 {
            self.previous_rain = total_rain;
        }
        
        let mut rain_diff = total_rain - self.previous_rain;
        let mut total_rain_final = total_rain;
        
        if rain_diff > self.config.max_rain_jump {
            // Filter rainfall spikes
            rain_diff = 0.0;
            total_rain_final = self.previous_rain;
        }
        
        self.previous_rain = total_rain_final;
        
        Ok(WeatherData {
            timestamp,
            indoor_humidity,
            outdoor_humidity,
            indoor_temperature,
            outdoor_temperature,
            outdoor_dew_point,
            wind_chill_temp,
            wind_speed,
            gust_speed,
            wind_direction_index: wind_dir.min(15), // Ensure valid index
            rain_diff,
            total_rain: total_rain_final,
            abs_pressure,
        })
    }
    
    fn dew_point(temperature: f32, humidity: f32) -> f32 {
        let humidity_fraction = humidity / 100.0;
        let gamma = (17.271 * temperature) / (237.7 + temperature) + log(humidity_fraction as f64) as f32;
        (237.7 * gamma) / (17.271 - gamma)
    }
    
    fn wind_chill(temperature: f32, wind: f32) -> f32 {
        let wind_kph = 3.6 * wind;
        
        // Low wind speed, or high temperature, negates any perceived wind chill
        if wind_kph <= 4.8 || temperature > 10.0 {
            return temperature;
        }
        
        let wct = 13.12 + (0.6215 * temperature) 
            - (11.37 * pow(wind_kph as f64, 0.16) as f32)
            + (0.3965 * temperature * pow(wind_kph as f64, 0.16) as f32);
        
        // Return the lower of temperature or wind chill temperature
        if wct < temperature {
            wct
        } else {
            temperature
        }
    }
}

impl WeatherData {
    pub fn print(&self, language: &str, translations: &crate::translations::Translations) {
        println!("\n================================================");
        println!("Weather Station Data - {}", self.timestamp.format("%Y-%m-%d %H:%M:%S"));
        println!("================================================");
        println!("Indoor:");
        println!("  {}: {:.1}°C", translations.get(language, "indoor_temperature"), self.indoor_temperature);
        println!("  {}: {}%", translations.get(language, "indoor_humidity"), self.indoor_humidity);
        println!();
        println!("Outdoor:");
        println!("  {}: {:.1}°C", translations.get(language, "outdoor_temperature"), self.outdoor_temperature);
        println!("  {}: {}%", translations.get(language, "outdoor_humidity"), self.outdoor_humidity);
        println!("  {}: {:.2}°C", translations.get(language, "outdoor_dew_point"), self.outdoor_dew_point);
        println!("  {}: {:.1}°C", translations.get(language, "wind_chill_temp"), self.wind_chill_temp);
        println!();
        println!("Wind:");
        println!("  {}: {:.1} m/s", translations.get(language, "wind_speed"), self.wind_speed);
        println!("  {}: {:.1} m/s", translations.get(language, "gust_speed"), self.gust_speed);
        println!("  {}: {}", translations.get(language, "wind_direction"), 
                translations.get_wind_direction(language, self.wind_direction_index));
        println!();
        println!("Rain:");
        println!("  {}: {:.1} mm", translations.get(language, "rain_diff"), self.rain_diff);
        println!("  {}: {:.1} mm", translations.get(language, "total_rain"), self.total_rain);
        println!();
        println!("Pressure:");
        println!("  {}: {:.1} hPa", translations.get(language, "abs_pressure"), self.abs_pressure);
        println!("================================================");
    }
}
