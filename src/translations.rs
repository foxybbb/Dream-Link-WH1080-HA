use std::collections::HashMap;

pub struct Translations {
    translations: HashMap<String, HashMap<String, String>>,
}

impl Translations {
    pub fn new() -> Self {
        let mut translations = HashMap::new();

        // English translations
        let mut en = HashMap::new();
        en.insert("indoor_humidity".to_string(), "Indoor Humidity".to_string());
        en.insert("outdoor_humidity".to_string(), "Outdoor Humidity".to_string());
        en.insert("indoor_temperature".to_string(), "Indoor Temperature".to_string());
        en.insert("outdoor_temperature".to_string(), "Outdoor Temperature".to_string());
        en.insert("outdoor_dew_point".to_string(), "Outdoor Dew Point".to_string());
        en.insert("wind_chill_temp".to_string(), "Wind Chill Temperature".to_string());
        en.insert("wind_speed".to_string(), "Wind Speed".to_string());
        en.insert("gust_speed".to_string(), "Gust Speed".to_string());
        en.insert("wind_direction".to_string(), "Wind Direction".to_string());
        en.insert("rain_diff".to_string(), "Rainfall".to_string());
        en.insert("total_rain".to_string(), "Total Rainfall".to_string());
        en.insert("abs_pressure".to_string(), "Atmospheric Pressure".to_string());
        
        // Wind directions in English
        en.insert("wind_dir_0".to_string(), "N".to_string());
        en.insert("wind_dir_1".to_string(), "NNE".to_string());
        en.insert("wind_dir_2".to_string(), "NE".to_string());
        en.insert("wind_dir_3".to_string(), "ENE".to_string());
        en.insert("wind_dir_4".to_string(), "E".to_string());
        en.insert("wind_dir_5".to_string(), "ESE".to_string());
        en.insert("wind_dir_6".to_string(), "SE".to_string());
        en.insert("wind_dir_7".to_string(), "SSE".to_string());
        en.insert("wind_dir_8".to_string(), "S".to_string());
        en.insert("wind_dir_9".to_string(), "SSW".to_string());
        en.insert("wind_dir_10".to_string(), "SW".to_string());
        en.insert("wind_dir_11".to_string(), "WSW".to_string());
        en.insert("wind_dir_12".to_string(), "W".to_string());
        en.insert("wind_dir_13".to_string(), "WNW".to_string());
        en.insert("wind_dir_14".to_string(), "NW".to_string());
        en.insert("wind_dir_15".to_string(), "NNW".to_string());

        translations.insert("en".to_string(), en);

        // Russian translations
        let mut ru = HashMap::new();
        ru.insert("indoor_humidity".to_string(), "Влажность внутри".to_string());
        ru.insert("outdoor_humidity".to_string(), "Влажность снаружи".to_string());
        ru.insert("indoor_temperature".to_string(), "Температура внутри".to_string());
        ru.insert("outdoor_temperature".to_string(), "Температура снаружи".to_string());
        ru.insert("outdoor_dew_point".to_string(), "Точка росы снаружи".to_string());
        ru.insert("wind_chill_temp".to_string(), "Ощущаемая температура".to_string());
        ru.insert("wind_speed".to_string(), "Скорость ветра".to_string());
        ru.insert("gust_speed".to_string(), "Скорость порывов ветра".to_string());
        ru.insert("wind_direction".to_string(), "Направление ветра".to_string());
        ru.insert("rain_diff".to_string(), "Количество осадков".to_string());
        ru.insert("total_rain".to_string(), "Общее количество осадков".to_string());
        ru.insert("abs_pressure".to_string(), "Давление".to_string());
        
        // Wind directions in Russian (from the Python code)
        ru.insert("wind_dir_0".to_string(), "С".to_string());
        ru.insert("wind_dir_1".to_string(), "ССВ".to_string());
        ru.insert("wind_dir_2".to_string(), "СВ".to_string());
        ru.insert("wind_dir_3".to_string(), "ВСВ".to_string());
        ru.insert("wind_dir_4".to_string(), "В".to_string());
        ru.insert("wind_dir_5".to_string(), "ВЮВ".to_string());
        ru.insert("wind_dir_6".to_string(), "ЮВ".to_string());
        ru.insert("wind_dir_7".to_string(), "ЮЮВ".to_string());
        ru.insert("wind_dir_8".to_string(), "Ю".to_string());
        ru.insert("wind_dir_9".to_string(), "ЮЮЗ".to_string());
        ru.insert("wind_dir_10".to_string(), "ЮЗ".to_string());
        ru.insert("wind_dir_11".to_string(), "ЗЮЗ".to_string());
        ru.insert("wind_dir_12".to_string(), "З".to_string());
        ru.insert("wind_dir_13".to_string(), "ЗСЗ".to_string());
        ru.insert("wind_dir_14".to_string(), "СЗ".to_string());
        ru.insert("wind_dir_15".to_string(), "ССЗ".to_string());

        translations.insert("ru".to_string(), ru);

        Self { translations }
    }

    pub fn get(&self, language: &str, key: &str) -> String {
        if let Some(lang_map) = self.translations.get(language) {
            if let Some(value) = lang_map.get(key) {
                return value.clone();
            }
        }
        
        // Fallback to English if key not found in requested language
        if let Some(en_map) = self.translations.get("en") {
            if let Some(value) = en_map.get(key) {
                return value.clone();
            }
        }
        
        // Final fallback to the key itself
        key.to_string()
    }

    pub fn get_wind_direction(&self, language: &str, direction_index: usize) -> String {
        let key = format!("wind_dir_{}", direction_index);
        self.get(language, &key)
    }

    pub fn get_available_languages(&self) -> Vec<String> {
        self.translations.keys().cloned().collect()
    }
}
