// src/config.rs
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::Path;
use std::process;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub device_port: u16,
    pub router_port: u16,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            device_port: 0,
            router_port: 0,
        }
    }
}
impl Config {
    /// Loads the configuration from the specified path.
    /// If the file does not exist, creates one with default values.
    pub fn load_or_create(path: &Path) -> io::Result<Self> {
        if path.exists() {
            let content = fs::read_to_string(path)?;
            let mut config: Config = toml::from_str(&content).unwrap_or_default();

            // Only write if the file is empty or invalidz
            if !is_config_complete(&config) {
                let toml_str = toml::to_string_pretty(&config).unwrap();
                let content = format!(
                    "# device_port is mandatory. Set it to a non-zero value to proceed.\n\
                    # router_port is optional. If set to 0, it will be equal to the device port.\n\n\
                    {}\n",
                    toml_str
                );
                fs::write(path, content)?;
                prompt();
            }

            if config.router_port == 0 {
                config.router_port = config.device_port;
            }

            Ok(config)
        } else {
            let config = Config::default();

            let toml_str = toml::to_string_pretty(&config).unwrap();
            let content = format!(
                "# Device port must be correctly set to non-zero value.\n\
                # If external port is set to 0, it will default to the device port.\n\n\
                {}\n",
                toml_str
            );

            fs::write(path, content)?;
            prompt();
            process::exit(0);
        }
    }

    // pub fn save(&self, path: &Path) -> io::Result<()> {
    //     let toml_str = toml::to_string(&self).unwrap();
    //     fs::write(path, toml_str)
    // }
}

fn prompt() {
    println!(
        "Please set the device port in the config file at `config.toml`, in this directory.\n",
    );
    println!("Example:");
    println!("device_port = 8080");
    println!("external_port = 0");

    println!("\nDevice port must be correctly set to non-zero value.");
    println!("If external port is set to 0, it will default to the device port.");
    println!("Please edit the file and rerun the program.");
    println!("Press Enter to close.");
    let mut buffer = String::new();
    let _ = io::stdin().read_line(&mut buffer);
    process::exit(0);
}

pub fn is_config_complete(config: &Config) -> bool {
    config.device_port != 0
}
