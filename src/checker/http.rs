use async_trait::async_trait;
use reqwest::Response;

use crate::checker::{CheckState, HttpBasedChecker};
use crate::config::CheckConfig;

pub struct Checker<'a> {
    check_config: &'a CheckConfig,
}

impl Checker<'_> {
    pub fn new(check_config: &CheckConfig) -> Checker {
        Checker { check_config }
    }
}

#[async_trait]
impl HttpBasedChecker for Checker<'_> {
    async fn check_response(response: Response) -> CheckState {
        if response.status().is_success() {
            CheckState::Up
        } else {
            CheckState::Warn
        }
    }

    fn get_check_config(&self) -> &CheckConfig {
        self.check_config
    }
}
