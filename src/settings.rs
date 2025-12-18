use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::sync::Mutex;
use std::time::Duration;
use tracing::{info, error};

const SETTINGS_FILE: &str = "data/settings.json";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Settings {
    // We store duration as u64 seconds for JSON serialization
    #[serde(default)]
    pub timeout_seconds: u64,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            timeout_seconds: 0, // 0 means no timeout / default
        }
    }
}

impl Settings {
    pub fn load() -> Self {
        if !Path::new(SETTINGS_FILE).exists() {
            info!("Settings file not found, creating default.");
            let settings = Settings::default();
            settings.save();
            return settings;
        }

        match fs::read_to_string(SETTINGS_FILE) {
            Ok(content) => {
                match serde_json::from_str(&content) {
                    Ok(settings) => settings,
                    Err(e) => {
                        error!("Failed to parse settings file: {}. Using defaults.", e);
                        Settings::default()
                    }
                }
            },
            Err(e) => {
                error!("Failed to read settings file: {}. Using defaults.", e);
                Settings::default()
            }
        }
    }

    pub fn save(&self) {
        match serde_json::to_string_pretty(self) {
            Ok(json) => {
                if let Err(e) = fs::write(SETTINGS_FILE, json) {
                    error!("Failed to write settings file: {}", e);
                }
            },
            Err(e) => {
                error!("Failed to serialize settings: {}", e);
            }
        }
    }

    pub fn set(&mut self, key: &str, value: &str) -> Result<String, String> {
        match key {
            "timeout" => {
                let duration = parse_duration(value)?;
                self.timeout_seconds = duration.as_secs();
                self.save();
                Ok(format!("Timeout set to {} seconds ({}).", self.timeout_seconds, value))
            },
            _ => Err(format!("Unknown setting key: {}", key)),
        }
    }
}

fn parse_duration(s: &str) -> Result<Duration, String> {
    let s = s.trim();
    if s.is_empty() {
        return Err("Empty duration string".to_string());
    }

    let (num_str, unit) = s.split_at(s.len() - 1);
    let num: u64 = num_str.parse().map_err(|_| format!("Invalid number format: {}", num_str))?;

    match unit {
        "s" => Ok(Duration::from_secs(num)),
        "m" => Ok(Duration::from_secs(num * 60)),
        "h" => Ok(Duration::from_secs(num * 3600)),
        "d" => Ok(Duration::from_secs(num * 86400)),
        _ => {
            // Check if the whole things is a number, assume seconds if so
            if let Ok(n) = s.parse::<u64>() {
                Ok(Duration::from_secs(n))
            } else {
                Err(format!("Unknown time unit: {}", unit))
            }
        }
    }
}
