// src/config.rs
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub device_port: u16,
    pub router_port: Option<u16>,
}

impl Config {
    /// Loads the configuration from the specified path.
    /// If the file does not exist, creates one with default values.
    pub fn load_or_create(path: &Path) -> io::Result<Self> {
        if path.exists() {
            let content = fs::read_to_string(path)?;
            let config: Config = toml::from_str(&content).unwrap_or(Config {
                device_port: 0,
                router_port: None,
            });
            Ok(config)
        } else {
            let config = Config {
                device_port: 0,
                router_port: None,
            };
            let toml_str = toml::to_string(&config).unwrap();
            fs::write(path, toml_str)?;
            Ok(config)
        }
    }

    /// Saves the current configuration to the specified path.
    pub fn save(&self, path: &Path) -> io::Result<()> {
        let toml_str = toml::to_string(&self).unwrap();
        fs::write(path, toml_str)
    }

    /// Checks if the configuration has at least the device port set.
    pub fn is_complete(&self) -> bool {
        self.device_port != 0
    }
}
