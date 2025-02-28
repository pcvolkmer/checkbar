use reqwest::Response;
use serde::Deserialize;

use crate::checker::{CheckState, HttpBasedChecker};
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
}

impl HttpBasedChecker for Checker<'_> {
    async fn check_response(response: Response) -> CheckState {
        if response.status().is_success() {
            return match response.json::<ActuatorResponse>().await {
                Ok(ar) if ar.status == "UP" => CheckState::Up,
                _ => CheckState::Warn,
            };
        }
        CheckState::Warn
    }

    fn get_check_config(&self) -> &CheckConfig {
        self.check_config
    }
}

#[cfg(test)]
mod tests {
    use crate::checker::actuator::Checker;
    use crate::checker::{CheckState, HttpBasedChecker};
    use reqwest::Response;
    use serde_json::json;

    #[tokio::test]
    async fn test_should_return_up_state() {
        let response = Response::from(
            http::Response::builder()
                .status(200)
                .body(json!({"status":"UP"}).to_string())
                .unwrap(),
        );
        let check_state = Checker::check_response(response).await;

        assert_eq!(check_state, CheckState::Up)
    }

    #[tokio::test]
    async fn test_should_return_warn_state_on_status_not_up() {
        let response = Response::from(
            http::Response::builder()
                .status(200)
                .body(json!({"status":"DOWN"}).to_string())
                .unwrap(),
        );
        let check_state = Checker::check_response(response).await;

        assert_eq!(check_state, CheckState::Warn)
    }

    #[tokio::test]
    async fn test_should_return_warn_state_on_response_not_success() {
        let response = Response::from(
            http::Response::builder()
                .status(404)
                .body(String::from("Actuator Response Not Found"))
                .unwrap(),
        );
        let check_state = Checker::check_response(response).await;

        assert_eq!(check_state, CheckState::Warn)
    }
}
