mod actuator;
mod http;
mod tcp;

use std::fmt::{Display, Formatter, Result};
use std::future::Future;

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

async fn check_http_response<F>(check_config: &CheckConfig, f: fn(Response) -> F) -> CheckResult
where
    F: Future<Output = CheckState>,
{
    CheckResult {
        name: check_config.name.to_string(),
        state: match reqwest::get(check_config.url.as_str()).await {
            Ok(r) => f(r).await,
            Err(_) => CheckState::Down,
        },
    }
}
