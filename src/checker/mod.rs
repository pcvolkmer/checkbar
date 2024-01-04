mod actuator;
mod http;
mod tcp;

use async_trait::async_trait;
use console::{style, Term};
use std::fmt::{Display, Formatter, Result};

use reqwest::Response;
use serde_json::json;

pub use crate::checker::actuator::Checker as ActuatorChecker;
pub use crate::checker::http::Checker as HttpChecker;
pub use crate::checker::tcp::Checker as TcpChecker;
use crate::config::{CheckConfig, CheckType, Config};

pub async fn check_host(check_config: &CheckConfig) -> CheckResult {
    match check_config.check_type {
        Some(CheckType::Actuator) => ActuatorChecker::new(check_config).check().await,
        Some(CheckType::Tcp) => TcpChecker::new(check_config).check().await,
        _ => HttpChecker::new(check_config).check().await,
    }
}

pub struct CheckResult {
    pub name: String,
    pub state: CheckState,
}

impl Display for CheckResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if Term::stdout().is_term() {
            return write!(
                f,
                "{}",
                match &self.state {
                    CheckState::Up => style(&self.name).green(),
                    CheckState::Warn => style(&self.name).yellow(),
                    CheckState::Down => style(&self.name).red(),
                }
            );
        }

        let color_config = Config::read().colors;
        let color = match &self.state {
            CheckState::Up => color_config.up,
            CheckState::Warn => color_config.warn,
            CheckState::Down => color_config.down,
        };

        write!(
            f,
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

#[derive(Debug, PartialEq)]
pub enum CheckState {
    Up,
    Warn,
    Down,
}

#[async_trait]
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
    fn test_should_display_check_result_up() {
        let check_result = CheckResult {
            name: "test".to_string(),
            state: CheckState::Up,
        };

        if Term::stdout().is_term() {
            assert_eq!(check_result.to_string(), "\u{1b}[32mtest\u{1b}[0m")
        } else {
            assert_eq!(
                check_result.to_string(),
                r##"{"color":"#00FF00","full_text":"test","name":"test","separator_block_width":16}"##
            )
        }
    }

    #[test]
    fn test_should_display_check_result_warn() {
        let check_result = CheckResult {
            name: "test".to_string(),
            state: CheckState::Warn,
        };

        if Term::stdout().is_term() {
            assert_eq!(check_result.to_string(), "\u{1b}[33mtest\u{1b}[0m")
        } else {
            assert_eq!(
                check_result.to_string(),
                r##"{"color":"#FFFF00","full_text":"test","name":"test","separator_block_width":16}"##
            )
        }
    }

    #[test]
    fn test_should_display_check_result_down() {
        let check_result = CheckResult {
            name: "test".to_string(),
            state: CheckState::Down,
        };

        if Term::stdout().is_term() {
            assert_eq!(check_result.to_string(), "\u{1b}[31mtest\u{1b}[0m")
        } else {
            assert_eq!(
                check_result.to_string(),
                r##"{"color":"#FF0000","full_text":"test","name":"test","separator_block_width":16}"##
            )
        }
    }
}
