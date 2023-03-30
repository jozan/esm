use serde_derive::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use toml;

use crate::dirs::get_esm_root_dir;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub empty_epsilon_path: Option<String>,
    pub registry: Option<String>,
}

pub fn get_config() -> Result<Config, Box<dyn Error>> {
    let config_path = get_config_path();
    let config_content = fs::read_to_string(config_path)?;
    let config: Config = toml::from_str(&config_content)?;
    Ok(config)
}

pub fn create_config() -> Result<Config, Box<dyn Error>> {
    let config_path = get_config_path();

    let config = Config {
        empty_epsilon_path: None,
        registry: Some("https://registry.esm.latehours.net/v1".to_string()),
    };

    let config_content = toml::to_string(&config)?;
    fs::write(config_path, config_content)?;

    Ok(config)
}


fn get_config_path() -> std::path::PathBuf {
    get_esm_root_dir().join("config.toml")
}
