use serde::Deserialize;
use config::{Config as ConfigLoader, File, Environment};
use std::path::Path;
use color_eyre::eyre::{eyre};
use std::fs;

#[derive(Deserialize, Default)]
pub struct Config {
    pub ip: Option<String>,
    pub port: Option<u16>,
    pub configuration: Option<String>,
    pub polling_interval: Option<u64>,
    pub debug: Option<bool>,
}

pub fn validate_toml(file_path: &str) -> color_eyre::Result<()> {
    let content = fs::read_to_string(file_path)
        .map_err(|e| eyre!("Failed to read config file: {}", e))?;
    
    toml::de::from_str::<Config>(&content)
        .map_err(|e| eyre!("TOML validation error: {}", e))?;
    
    Ok(())
}

pub fn load_config(file_path: &str) -> color_eyre::Result<Config> {
    let mut settings = ConfigLoader::builder();

    // Check if the file exists before adding it as a source
    if Path::new(file_path).exists() {
        // Validate the TOML file
        validate_toml(file_path)?;

        // Add the source
        settings = settings.add_source(File::with_name(file_path));
    } else {
        println!("Config file does not exist: {}. Using default configuration.", file_path);
    }

    // Add the source for environment variables
    settings = settings.add_source(Environment::with_prefix("VITAOXIPAD"));

    // Build the settings and deserialize
    settings.build()?.try_deserialize().map_err(|e| eyre!(e))
}

pub fn print_sample_config() {
    println!(r#"
# Sample Configuration file for the VitaOxiPad
# Refer ReadMe for all available options

# The IP address to bind to
ip = "192.168.0.100"

# The port to listen on
port = 5000

# Path to another configuration file (if needed)
# Touchpad config to use:
# - standart
# - alt_triggers
# - rear_touchpad
# - front_touchpad
config = "standart"

# Polling interval in seconds
polling_interval = 4000

# Enable or disable debug mode
debug = false
"#);
}
