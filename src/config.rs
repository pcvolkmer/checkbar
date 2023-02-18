use std::fmt::Formatter;
use std::time::Duration;
use std::{env, fs};

use regex::Regex;
use serde::de::{Error, Visitor};
use serde::{Deserialize, Deserializer};

#[derive(Default, Deserialize)]
pub struct Config {
    #[serde(default, deserialize_with = "deserialize_duration")]
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

fn parse_duration(value: &str) -> Option<Duration> {
    let mut duration_in_secs = 0;
    if let Ok(re) = Regex::new(
        r"^((?P<hours>\d+)(h|hour|hours)\s*)?((?P<minutes>\d+)(m|min|mins|minute|minutes)\s*)?((?P<seconds>\d+)(s|sec|secs|second|seconds)?\s*)?$",
    ) {
        if re.is_match(value) {
            let parts = re.captures_iter(value).next().unwrap();
            if let Some(hours) = parts.name("hours") {
                duration_in_secs += hours.as_str().parse::<u64>().unwrap_or(0) * 60 * 60
            }
            if let Some(minutes) = parts.name("minutes") {
                duration_in_secs += minutes.as_str().parse::<u64>().unwrap_or(0) * 60
            }
            if let Some(seconds) = parts.name("seconds") {
                duration_in_secs += seconds.as_str().parse::<u64>().unwrap_or(0)
            }
        } else {
            return None;
        }
    }
    Some(Duration::from_secs(duration_in_secs))
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
        assert_eq!(config.interval, None);
        assert_eq!(config.checks.len(), 0);
    }

    macro_rules! multi_assert_eq {
        ($f:expr, $expected:expr, $ ( $left:expr ), + ) => {
            $(
                 assert_eq!($f($left), $expected, $left);
            )+
        };
    }

    #[test]
    fn test_should_parse_durations() {
        assert_eq!(parse_duration("1m30s"), Some(Duration::from_secs(90)));
        assert_eq!(parse_duration("2m"), Some(Duration::from_secs(120)));
        assert_eq!(parse_duration("90"), Some(Duration::from_secs(90)));

        multi_assert_eq!(
            parse_duration,
            Some(Duration::from_secs(3661)),
            "1h1m1s",
            "1hour1min1s",
            "1hour1mins1s",
            "1hour1minute1sec",
            "1hour1minute1secs",
            "1hour1minutes1second",
            "1hour1minutes1seconds",
            "1hours1minutes1seconds"
        );
    }

    #[test]
    fn test_should_parse_durations_with_whitespaces() {
        assert_eq!(parse_duration("1m 30s"), Some(Duration::from_secs(90)));
        assert_eq!(parse_duration("1h 1m 1s"), Some(Duration::from_secs(3661)));

        multi_assert_eq!(
            parse_duration,
            Some(Duration::from_secs(3661)),
            "1h 1m 1s",
            "1hour 1min 1s",
            "1hour 1mins 1s",
            "1hour 1minute 1sec",
            "1hour 1minute 1secs",
            "1hour 1minutes 1second",
            "1hour 1minutes 1seconds",
            "1hours 1minutes 1seconds"
        );
    }

    #[test]
    fn test_should_return_default_for_unparseable_durations() {
        assert_eq!(parse_duration("invalid"), None);
        assert_eq!(parse_duration("1x30m10q"), None);
    }
}
