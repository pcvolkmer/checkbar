use std::str::FromStr;

use reqwest::Url;
use tokio::net::TcpStream;

use crate::checker::{CheckResult, CheckState};
use crate::config::CheckConfig;

pub struct Checker<'a> {
    check_config: &'a CheckConfig,
}

impl Checker<'_> {
    pub fn new(check_config: &CheckConfig) -> Checker {
        Checker { check_config }
    }

    pub async fn check(&self) -> CheckResult {
        if let Ok(url) = Url::from_str(self.check_config.url.as_str()) {
            if url.scheme() == "tcp" && url.host_str().is_some() && url.port().is_some() {
                let state = match TcpStream::connect(format!(
                    "{}:{}",
                    url.host_str().unwrap(),
                    url.port().unwrap()
                ))
                .await
                {
                    Ok(_) => CheckState::Up,
                    _ => CheckState::Down,
                };
                return CheckResult {
                    name: self.check_config.name.to_string(),
                    state,
                };
            }
        }

        CheckResult {
            name: self.check_config.name.to_string(),
            state: CheckState::Down,
        }
    }
}
