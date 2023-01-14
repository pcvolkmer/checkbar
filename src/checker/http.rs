use reqwest::Response;

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
        CheckResult {
            name: self.check_config.name.to_string(),
            state: match reqwest::get(self.check_config.url.as_str()).await {
                Ok(r) => Self::check_response(r).await,
                Err(_) => CheckState::Down,
            },
        }
    }

    async fn check_response(response: Response) -> CheckState {
        if response.status().is_success() {
            CheckState::Up
        } else {
            CheckState::Warn
        }
    }
}
