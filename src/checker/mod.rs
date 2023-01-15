mod actuator;
mod http;
mod tcp;

use async_trait::async_trait;
use std::fmt::{Display, Formatter, Result};

use reqwest::Response;
use serde_json::json;

pub use crate::checker::actuator::Checker as ActuatorChecker;
pub use crate::checker::http::Checker as HttpChecker;
pub use crate::checker::tcp::Checker as TcpChecker;
use crate::config::{CheckConfig, Config};

pub struct CheckResult {
    pub name: String,
    pub state: CheckState,
}

impl Display for CheckResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
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
