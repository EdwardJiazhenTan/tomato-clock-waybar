use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use toml;
use dirs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub default_workflow: String,
    pub default_status: String,
    pub notification_enabled: bool,
    pub waybar_integration: WaybarConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaybarConfig {
    pub enabled: bool,
    pub format: String,
    pub socket_path: Option<String>,
    pub click_events: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_workflow: "Default Pomodoro".to_string(),
            default_status: "work".to_string(),
            notification_enabled: true,
            waybar_integration: WaybarConfig::default(),
        }
    }
}

impl Default for WaybarConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            format: "{icon} {status}: {remaining}".to_string(),
            socket_path: None,
            click_events: true,
        }
    }
}

lazy_static::lazy_static! {
    static ref CONFIG: Arc<Mutex<Config>> = Arc::new(Mutex::new(Config::default()));
}

pub fn get_config_dir() -> PathBuf {
    let mut config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("./config"));
    config_dir.push("tomato-clock");
    config_dir
}

pub fn get_config_file_path(custom_path: Option<PathBuf>) -> PathBuf {
    match custom_path {
        Some(path) => path,
        None => {
            let mut path = get_config_dir();
            path.push("config.toml");
            path
        }
    }
}

pub fn init(custom_path: Option<PathBuf>) -> Result<(), String> {
    let config_path = get_config_file_path(custom_path);
    
    // Create config directory if it doesn't exist
    if let Some(parent) = config_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }
    }
    
    // Load or create config file
    let config = if config_path.exists() {
        // Load existing config
        let config_str = fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        
        toml::from_str::<Config>(&config_str)
            .map_err(|e| format!("Failed to parse config file: {}", e))?
    } else {
        // Create default config
        let config = Config::default();
        save_config(&config, Some(&config_path))?;
        config
    };
    
    // Update global config
    *CONFIG.lock().unwrap() = config;
    
    Ok(())
}

pub fn get() -> Config {
    CONFIG.lock().unwrap().clone()
}

#[allow(dead_code)]
pub fn update(config: Config) -> Result<(), String> {
    *CONFIG.lock().unwrap() = config.clone();
    save_config(&config, None)
}

pub fn save_config(config: &Config, custom_path: Option<&Path>) -> Result<(), String> {
    let config_path = match custom_path {
        Some(path) => PathBuf::from(path),
        None => get_config_file_path(None),
    };
    
    let config_str = toml::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    
    fs::write(&config_path, config_str)
        .map_err(|e| format!("Failed to write config file: {}", e))?;
    
    Ok(())
} 