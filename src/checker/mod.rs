mod actuator;
mod http;
mod tcp;

pub use crate::checker::actuator::Checker as ActuatorChecker;
pub use crate::checker::http::Checker as HttpChecker;
pub use crate::checker::tcp::Checker as TcpChecker;
use crate::config;
use crate::config::get_config;
use serde_json::json;
use std::fmt::{Display, Formatter, Result};

pub struct CheckResult {
    pub name: String,
    pub state: CheckState,
}

impl Display for CheckResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let color_config = match get_config().colors {
            Some(color_config) => color_config,
            None => config::ColorConfig {
                up: String::from("#00FF00"),
                warn: String::from("#FFFF00"),
                down: String::from("#FF0000"),
            },
        };
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
