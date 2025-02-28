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

impl HttpBasedChecker for Checker<'_> {
    async fn check_response(response: Response) -> CheckState {
        if response.status().is_success() {
            return CheckState::Up;
        }
        CheckState::Warn
    }

    fn get_check_config(&self) -> &CheckConfig {
        self.check_config
    }
}

#[cfg(test)]
mod tests {
    use crate::checker::http::Checker;
    use crate::checker::{CheckState, HttpBasedChecker};
    use reqwest::Response;

    #[tokio::test]
    async fn test_should_return_up_state() {
        let response = Response::from(
            http::Response::builder()
                .status(200)
                .body("Any response")
                .unwrap(),
        );
        let check_state = Checker::check_response(response).await;

        assert_eq!(check_state, CheckState::Up)
    }

    #[tokio::test]
    async fn test_should_return_warn_state_on_response_not_success() {
        let response = Response::from(
            http::Response::builder()
                .status(404)
                .body(String::from("Http Response Not Found"))
                .unwrap(),
        );
        let check_state = Checker::check_response(response).await;

        assert_eq!(check_state, CheckState::Warn)
    }
}
