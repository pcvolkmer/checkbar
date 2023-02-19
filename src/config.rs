use parse_duration::parse_duration;
use std::fmt::Formatter;
use std::time::Duration;
use std::{env, fs};

use serde::de::{Error, Visitor};
use serde::{Deserialize, Deserializer};

#[derive(Deserialize)]
pub struct Config {
    #[serde(
        default = "default_duration",
        deserialize_with = "deserialize_duration"
    )]
    pub interval: Option<Duration>,
    #[serde(default)]
    pub colors: ColorConfig,
    #[serde(default)]
    pub checks: Vec<CheckConfig>,
}

impl Config {
    fn get_config_file() -> String {
        match env::args().nth(1) {
            Some(config_file) => config_file,
            None => format!(
                "{}/.checkbar.toml",
                dirs::home_dir().unwrap().to_str().unwrap_or("")
            ),
        }
    }

    pub fn read() -> Self {
        Self::read_file(Self::get_config_file().as_str())
    }

    pub fn read_file(filename: &str) -> Self {
        match fs::read_to_string(filename) {
            Ok(config) => toml::from_str(config.as_str()).unwrap_or_default(),
            Err(_) => Config::default(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            interval: Some(Duration::from_secs(60)),
            colors: ColorConfig::default(),
            checks: vec![],
        }
    }
}

#[derive(Deserialize)]
pub struct ColorConfig {
    pub up: String,
    pub warn: String,
    pub down: String,
}

impl Default for ColorConfig {
    fn default() -> Self {
        Self {
            up: String::from("#00FF00"),
            warn: String::from("#FFFF00"),
            down: String::from("#FF0000"),
        }
    }
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

fn default_duration() -> Option<Duration> {
    Some(Duration::from_secs(60))
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
            Ok(format!("{v}"))
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

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::config::Config;

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
    fn test_should_parse_config_and_use_default_interval() {
        let config: Config = toml::from_str(
            r#"
                [[checks]]
                name = "example"
                url = "https://example.com"
            "#,
        )
        .unwrap();

        assert_eq!(config.interval, Some(Duration::from_secs(60)));
    }

    #[test]
    fn test_should_parse_config_without_checks() {
        let config: Config = toml::from_str(
            r#"
                interval = "2m 3s"
            "#,
        )
        .unwrap();

        assert_eq!(config.checks.len(), 0);
    }

    #[test]
    fn test_should_parse_config_with_default_colors() {
        let config: Config = toml::from_str(
            r#"
                interval = "2m 3s"
            "#,
        )
        .unwrap();

        assert_eq!(config.colors.up, "#00FF00".to_string());
        assert_eq!(config.colors.warn, "#FFFF00".to_string());
        assert_eq!(config.colors.down, "#FF0000".to_string());
    }

    #[test]
    fn test_should_read_and_parse_file() {
        let config = Config::read_file("./tests/testconfig1.toml");
        assert_eq!(config.interval, Some(Duration::from_secs(10)));
        assert_eq!(config.checks.len(), 1);
        assert_eq!(config.checks[0].name, "www");
        assert_eq!(config.checks[0].url, "https://example.com");
    }

    #[test]
    fn test_should_return_default_if_no_config_file() {
        let config = Config::read_file("./tests/no_testconfig.toml");
        assert_eq!(config.interval, Some(Duration::from_secs(60)));
        assert_eq!(config.colors.up, "#00FF00".to_string());
        assert_eq!(config.colors.warn, "#FFFF00".to_string());
        assert_eq!(config.colors.down, "#FF0000".to_string());
        assert_eq!(config.checks.len(), 0);
    }
}
