use regex::Regex;
use std::time::Duration;

pub fn parse_duration(value: &str) -> Option<Duration> {
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
