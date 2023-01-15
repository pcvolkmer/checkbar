use reqwest::Response;
use serde::Deserialize;

use crate::checker::{check_http_response, CheckResult, CheckState};
use crate::config::CheckConfig;

#[derive(Deserialize)]
struct ActuatorResponse {
    status: String,
}

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
            return match response.json::<ActuatorResponse>().await {
                Ok(ar) if ar.status == "UP" => CheckState::Up,
                _ => CheckState::Warn,
            };
        }
        CheckState::Warn
    }
}
