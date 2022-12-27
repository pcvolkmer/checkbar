use serde::Deserialize;
use std::{env, fs};

#[derive(Deserialize)]
pub struct Config {
    pub interval: Option<u64>,
    pub colors: Option<ColorConfig>,
    pub checks: Vec<CheckConfig>,
}

#[derive(Deserialize)]
pub struct ColorConfig {
    pub up: String,
    pub warn: String,
    pub down: String,
}

#[derive(Deserialize)]
pub struct CheckConfig {
    pub name: String,
    pub url: String,
    pub check_type: Option<CheckType>,
    pub click_cmd: Option<String>,
}

#[derive(Deserialize, PartialEq, Eq)]
pub enum CheckType {
    Http,
    Actuator,
    Tcp,
}

fn get_config_file() -> String {
    match env::args().nth(1) {
        Some(config_file) => config_file,
        None => format!(
            "{}/.checkbar.toml",
            dirs::home_dir().unwrap().to_str().unwrap_or("")
        ),
    }
}

pub fn get_config() -> Config {
    match fs::read_to_string(get_config_file()) {
        Ok(config) => match toml::from_str(config.as_str()) {
            Ok(config) => config,
            Err(_e) => Config {
                interval: None,
                colors: None,
                checks: vec![],
            },
        },
        Err(_) => Config {
            interval: None,
            colors: None,
            checks: vec![],
        },
    }
}
