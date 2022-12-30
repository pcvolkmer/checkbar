use regex::Regex;
use serde::Deserialize;
use std::time::Duration;
use std::{env, fs};

#[derive(Deserialize)]
pub struct Config {
    pub interval: Option<String>,
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

pub fn parse_duration(value: Option<String>) -> Duration {
    let mut result = 0;
    let value = match &value {
        Some(value) => {
            let result = value;
            result.as_str()
        }
        _ => return Duration::from_secs(60),
    };
    if let Ok(re) =
        Regex::new(r"^((?P<hours>\d+)h\s*)?((?P<minutes>\d+)m\s*)?((?P<seconds>\d+)s?\s*)?$")
    {
        if re.is_match(value) {
            let parts = re.captures_iter(value).next().unwrap();
            if let Some(hours) = parts.name("hours") {
                result += match hours.as_str().parse::<u64>() {
                    Ok(value) => value * 60 * 60,
                    _ => 0,
                };
            }
            if let Some(minutes) = parts.name("minutes") {
                result += match minutes.as_str().parse::<u64>() {
                    Ok(value) => value * 60,
                    _ => 0,
                };
            }
            if let Some(seconds) = parts.name("seconds") {
                result += match seconds.as_str().parse::<u64>() {
                    Ok(value) => value,
                    _ => 0,
                };
            }
        } else {
            return Duration::from_secs(60);
        }
    }
    Duration::from_secs(result)
}

#[cfg(test)]
mod tests {
    use crate::config::parse_duration;
    use std::time::Duration;

    #[test]
    fn test_should_parse_durations() {
        assert_eq!(
            parse_duration(Some("1m30s".to_string())),
            Duration::from_secs(90)
        );
        assert_eq!(
            parse_duration(Some("2m".to_string())),
            Duration::from_secs(120)
        );
        assert_eq!(
            parse_duration(Some("1h1m1s".to_string())),
            Duration::from_secs(3661)
        );
        assert_eq!(
            parse_duration(Some("90".to_string())),
            Duration::from_secs(90)
        );
    }

    #[test]
    fn test_should_parse_durations_with_whitespaces() {
        assert_eq!(
            parse_duration(Some("1m 30s".to_string())),
            Duration::from_secs(90)
        );
        assert_eq!(
            parse_duration(Some("1h 1m 1s".to_string())),
            Duration::from_secs(3661)
        );
    }

    #[test]
    fn test_should_return_default_for_unparseable_durations() {
        assert_eq!(parse_duration(None), Duration::from_secs(60));
        assert_eq!(
            parse_duration(Some("invalid".to_string())),
            Duration::from_secs(60)
        );
        assert_eq!(
            parse_duration(Some("1x30m10q".to_string())),
            Duration::from_secs(60)
        );
    }
}
