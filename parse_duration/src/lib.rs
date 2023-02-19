use regex::Regex;
use std::error::Error as ErrorTrait;
use std::fmt::{Debug, Display, Formatter};
use std::time::Duration;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Error {
    EmptyValue,
    NotParseable(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyValue => write!(f, "EmptyValueError: Empty value found"),
            Self::NotParseable(s) => write!(f, "NotParseableError: Cannot parse '{s}'"),
        }
    }
}

impl ErrorTrait for Error {
    fn description(&self) -> &str {
        match self {
            Self::EmptyValue => "Empty value found",
            Self::NotParseable(_) => "Not parsable",
        }
    }
}

pub fn parse(value: &str) -> Result<Duration, Error> {
    let mut duration_in_secs = 0;
    if value.trim().is_empty() {
        return Err(Error::EmptyValue);
    } else if let Ok(re) = Regex::new(
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
            return Ok(Duration::from_secs(duration_in_secs));
        }
    }
    Err(Error::NotParseable(value.to_string()))
}

#[deprecated(note = "Please use `parse` instead")]
pub fn parse_duration(value: &str) -> Option<Duration> {
    match parse(value) {
        Ok(duration) => Some(duration),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! multi_assert_eq {
        ($f:expr, $expected:expr, $ ( $left:expr ), + ) => {
            $(
                 assert_eq!($f($left), $expected, $left);
            )+
        };
    }

    #[test]
    fn test_should_parse_durations() {
        assert_eq!(parse("1m30s"), Ok(Duration::from_secs(90)));
        assert_eq!(parse("2m"), Ok(Duration::from_secs(120)));
        assert_eq!(parse("90"), Ok(Duration::from_secs(90)));

        multi_assert_eq!(
            parse,
            Ok(Duration::from_secs(3661)),
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
        assert_eq!(parse("1m 30s"), Ok(Duration::from_secs(90)));
        assert_eq!(parse("1h 1m 1s"), Ok(Duration::from_secs(3661)));

        multi_assert_eq!(
            parse,
            Ok(Duration::from_secs(3661)),
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
        assert_eq!(parse("   "), Err(Error::EmptyValue));
        assert_eq!(
            parse("invalid"),
            Err(Error::NotParseable("invalid".to_string()))
        );
        assert_eq!(
            parse("1x30m10q"),
            Err(Error::NotParseable("1x30m10q".to_string()))
        );
    }
}
