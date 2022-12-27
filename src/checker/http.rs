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
        let state = match reqwest::get(self.check_config.url.as_str()).await {
            Ok(r) => {
                if r.status().is_success() {
                    CheckState::Up
                } else {
                    CheckState::Warn
                }
            }
            Err(_) => CheckState::Down,
        };

        CheckResult {
            name: self.check_config.name.to_string(),
            state,
        }
    }
}
