use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::PathBuf;
use std::io::Write;
use std::env;
use std::sync::OnceLock;

pub const DEFAULT_CONFIG_FILENAME: &str = "config.json";

pub static CONFIG: OnceLock<AppConfig> = OnceLock::new();

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub service_name: String,
    pub service_display_name: String,
    pub target_adapter_name: String,
    pub link_speed_threshold_bps: u64,
    pub wait_after_wake_secs: u64,
    pub restart_delay_secs: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            service_name: "RelinkNetworkService".to_string(),
            service_display_name: "Relink Network Monitor Service".to_string(),
            target_adapter_name: "Realtek Gaming USB 2.5GbE Family Controller".to_string(),
            link_speed_threshold_bps: 100_000_000,
            wait_after_wake_secs: 15,
            restart_delay_secs: 3,
        }
    }
}

impl AppConfig {
    pub fn get_path() -> PathBuf {
        let mut path = env::current_exe().unwrap_or_default();
        path.set_file_name(DEFAULT_CONFIG_FILENAME);
        path
    }

    pub fn load() -> Self {
        let path = Self::get_path();
        if let Ok(file) = File::open(&path) {
            if let Ok(config) = serde_json::from_reader(file) {
                return config;
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::get_path();
        let json = serde_json::to_string_pretty(self)?;
        let mut file = File::create(path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }
    
    pub fn init() {
        let config = Self::load();
        let _ = CONFIG.set(config);
    }
    
    pub fn global() -> &'static AppConfig {
        CONFIG.get().expect("Config not initialized")
    }
}