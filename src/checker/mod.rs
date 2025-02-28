use std::fmt::{Display, Formatter, Result};

use console::{style, Term};
use reqwest::Response;
use serde_json::json;

pub use crate::checker::actuator::Checker as ActuatorChecker;
pub use crate::checker::http::Checker as HttpChecker;
pub use crate::checker::tcp::Checker as TcpChecker;
use crate::config::{CheckConfig, CheckType, Config};

mod actuator;
mod http;
mod tcp;

pub async fn check_host(check_config: &CheckConfig) -> CheckResult {
    match check_config.check_type {
        Some(CheckType::Actuator) => ActuatorChecker::new(check_config).check().await,
        Some(CheckType::Tcp) => TcpChecker::new(check_config).check().await,
        _ => HttpChecker::new(check_config).check().await,
    }
}

trait ToNonTerminalString: ToString {
    fn to_string(&self) -> String;
}
trait ToNonColoredTerminalString: ToString {
    fn to_string(&self) -> String;
}

trait ToColoredTerminalString: ToString {
    fn to_string(&self) -> String;
}

pub struct CheckResult {
    pub name: String,
    pub state: CheckState,
}

impl ToNonTerminalString for CheckResult {
    #[inline]
    fn to_string(&self) -> String {
        let color_config = Config::read().colors;
        let color = match &self.state {
            CheckState::Up => color_config.up,
            CheckState::Warn => color_config.warn,
            CheckState::Down => color_config.down,
        };

        format!(
            "{}",
            json!({
                "full_text": self.name,
                "name": self.name,
                "separator_block_width": 16,
                "color": color
            })
        )
    }
}

impl ToNonColoredTerminalString for CheckResult {
    #[inline]
    fn to_string(&self) -> String {
        self.name.to_string()
    }
}

impl ToColoredTerminalString for CheckResult {
    #[inline]
    fn to_string(&self) -> String {
        format!(
            "{}",
            match &self.state {
                CheckState::Up => style(&self.name).green().force_styling(true),
                CheckState::Warn => style(&self.name).yellow().force_styling(true),
                CheckState::Down => style(&self.name).red().force_styling(true),
            }
        )
    }
}

impl Display for CheckResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let term = Term::stdout();
        if term.is_term() && term.features().colors_supported() {
            return write!(f, "{}", ToColoredTerminalString::to_string(self));
        } else if term.is_term() && !term.features().colors_supported() {
            return write!(f, "{}", ToNonColoredTerminalString::to_string(self));
        }
        write!(f, "{}", ToNonTerminalString::to_string(self))
    }
}

#[derive(Debug, PartialEq)]
pub enum CheckState {
    Up,
    Warn,
    Down,
}

pub trait HttpBasedChecker {
    async fn check(&self) -> CheckResult {
        CheckResult {
            name: self.get_check_config().name.to_string(),
            state: match reqwest::get(self.get_check_config().url.as_str()).await {
                Ok(r) => Self::check_response(r).await,
                Err(_) => CheckState::Down,
            },
        }
    }

    async fn check_response(response: Response) -> CheckState;

    fn get_check_config(&self) -> &CheckConfig;
}

#[cfg(test)]
mod tests {
    use crate::checker::*;

    #[test]
    fn test_should_display_check_result_up_in_term() {
        let check_result = CheckResult {
            name: "test".to_string(),
            state: CheckState::Up,
        };

        assert_eq!(ToNonColoredTerminalString::to_string(&check_result), "test")
    }

    #[test]
    fn test_should_display_check_result_warn_in_term() {
        let check_result = CheckResult {
            name: "test".to_string(),
            state: CheckState::Warn,
        };

        assert_eq!(ToNonColoredTerminalString::to_string(&check_result), "test")
    }

    #[test]
    fn test_should_display_check_result_down_in_term() {
        let check_result = CheckResult {
            name: "test".to_string(),
            state: CheckState::Down,
        };

        assert_eq!(ToNonColoredTerminalString::to_string(&check_result), "test")
    }

    #[test]
    fn test_should_display_check_result_up_in_colored_term() {
        let check_result = CheckResult {
            name: "test".to_string(),
            state: CheckState::Up,
        };

        assert_eq!(
            ToColoredTerminalString::to_string(&check_result),
            "\u{1b}[32mtest\u{1b}[0m"
        )
    }

    #[test]
    fn test_should_display_check_result_warn_in_colored_term() {
        let check_result = CheckResult {
            name: "test".to_string(),
            state: CheckState::Warn,
        };

        assert_eq!(
            ToColoredTerminalString::to_string(&check_result),
            "\u{1b}[33mtest\u{1b}[0m"
        )
    }

    #[test]
    fn test_should_display_check_result_down_in_colored_term() {
        let check_result = CheckResult {
            name: "test".to_string(),
            state: CheckState::Down,
        };

        assert_eq!(
            ToColoredTerminalString::to_string(&check_result),
            "\u{1b}[31mtest\u{1b}[0m"
        )
    }

    #[test]
    fn test_should_display_check_result_up() {
        let check_result = CheckResult {
            name: "test".to_string(),
            state: CheckState::Up,
        };

        assert_eq!(
            ToNonTerminalString::to_string(&check_result),
            r##"{"color":"#00FF00","full_text":"test","name":"test","separator_block_width":16}"##
        )
    }

    #[test]
    fn test_should_display_check_result_warn() {
        let check_result = CheckResult {
            name: "test".to_string(),
            state: CheckState::Warn,
        };

        assert_eq!(
            ToNonTerminalString::to_string(&check_result),
            r##"{"color":"#FFFF00","full_text":"test","name":"test","separator_block_width":16}"##
        )
    }

    #[test]
    fn test_should_display_check_result_down() {
        let check_result = CheckResult {
            name: "test".to_string(),
            state: CheckState::Down,
        };

        assert_eq!(
            ToNonTerminalString::to_string(&check_result),
            r##"{"color":"#FF0000","full_text":"test","name":"test","separator_block_width":16}"##
        )
    }
}
