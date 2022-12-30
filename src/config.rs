use std::fmt::Formatter;
use std::time::Duration;
use std::{env, fs};

use regex::Regex;
use serde::de::{Error, Visitor};
use serde::{Deserialize, Deserializer};

#[derive(Deserialize)]
pub struct Config {
    #[serde(default, deserialize_with = "deserialize_duration")]
    pub interval: Option<Duration>,
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

fn deserialize_duration<'de, D>(d: D) -> Result<Option<Duration>, D::Error>
where
    D: Deserializer<'de>,
{
    struct StringVisitor;

    impl<'de> Visitor<'de> for StringVisitor {
        type Value = String;

        fn expecting(&self, f: &mut Formatter) -> std::fmt::Result {
            f.write_str("a number or string with parsable duration")
        }

        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(format!("{}", v))
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(v.to_string())
        }
    }

    match d.deserialize_string(StringVisitor) {
        Ok(value) => Ok(parse_duration(value.as_str())),
        Err(err) => Err(err),
    }
}

fn parse_duration(value: &str) -> Option<Duration> {
    let mut result = 0;
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
            return None;
        }
    }
    Some(Duration::from_secs(result))
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::config::{parse_duration, Config};

    #[test]
    fn test_should_parse_config_with_number_interval() {
        let config: Config = toml::from_str(
            r#"
                interval = 123

                [[checks]]
                name = "example"
                url = "https://example.com"
            "#,
        )
        .unwrap();

        assert_eq!(config.interval, Some(Duration::from_secs(123)));
    }

    #[test]
    fn test_should_parse_config_with_parsed_interval() {
        let config: Config = toml::from_str(
            r#"
                interval = "2m 3s"

                [[checks]]
                name = "example"
                url = "https://example.com"
            "#,
        )
        .unwrap();

        assert_eq!(config.interval, Some(Duration::from_secs(123)));
    }

    #[test]
    fn test_should_parse_config_without_interval() {
        let config: Config = toml::from_str(
            r#"
                [[checks]]
                name = "example"
                url = "https://example.com"
            "#,
        )
        .unwrap();

        assert_eq!(config.interval, None);
    }

    #[test]
    fn test_should_parse_durations() {
        assert_eq!(parse_duration("1m30s"), Some(Duration::from_secs(90)));
        assert_eq!(parse_duration("2m"), Some(Duration::from_secs(120)));
        assert_eq!(parse_duration("1h1m1s"), Some(Duration::from_secs(3661)));
        assert_eq!(parse_duration("90"), Some(Duration::from_secs(90)));
    }

    #[test]
    fn test_should_parse_durations_with_whitespaces() {
        assert_eq!(parse_duration("1m 30s"), Some(Duration::from_secs(90)));
        assert_eq!(parse_duration("1h 1m 1s"), Some(Duration::from_secs(3661)));
    }

    #[test]
    fn test_should_return_default_for_unparseable_durations() {
        assert_eq!(parse_duration("invalid"), None);
        assert_eq!(parse_duration("1x30m10q"), None);
    }
}
