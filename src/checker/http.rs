use reqwest::Response;

use crate::checker::{check_http_response, CheckResult, CheckState};
use crate::config::CheckConfig;

pub struct Checker<'a> {
    check_config: &'a CheckConfig,
}

impl Checker<'_> {
    pub fn new(check_config: &CheckConfig) -> Checker {
        Checker { check_config }
    }

    pub async fn check(&self) -> CheckResult {
        check_http_response(self.check_config, Self::check_response).await
    }

    async fn check_response(response: Response) -> CheckState {
        if response.status().is_success() {
            CheckState::Up
        } else {
            CheckState::Warn
        }
    }
}
