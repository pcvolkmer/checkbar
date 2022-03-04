use serde::Deserialize;
use std::fmt::{Display, Formatter, Result};
use std::time::Duration;

#[derive(Deserialize)]
struct Config {
    interval: Option<u64>,
    checks: Vec<CheckConfig>,
}

#[derive(Deserialize)]
struct CheckConfig {
    name: String,
    url: String,
    check_type: Option<CheckType>,
}

#[derive(Deserialize)]
enum CheckType {
    Http,
    Actuator,
}

#[derive(Deserialize)]
struct ActuatorResponse {
    status: String,
}

struct CheckResult {
    name: String,
    state: CheckState,
}

impl Display for CheckResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let color = match &self.state {
            CheckState::Up => "#00FF00",
            CheckState::Warn => "#FFFF00",
            CheckState::Down => "#FF0000",
        };
        write!(
            f,
            "{{\"full_text\":\"{}\",\"separator_block_width\":16,\"color\":\"{}\"}}",
            self.name, color
        )
    }
}

enum CheckState {
    Up,
    Warn,
    Down,
}

async fn check_host(check_config: &CheckConfig) -> CheckResult {
    let state = match reqwest::get(check_config.url.as_str()).await {
        Ok(r) => {
            if r.status().is_success() {
                match check_config.check_type {
                    Some(CheckType::Actuator) => match r.json::<ActuatorResponse>().await {
                        Ok(ar) => {
                            if ar.status == "OK" {
                                CheckState::Up
                            } else {
                                CheckState::Warn
                            }
                        }
                        _ => CheckState::Warn,
                    },
                    // Default: HTTP
                    _ => CheckState::Up,
                }
            } else {
                CheckState::Warn
            }
        }
        Err(_) => CheckState::Down,
    };

    CheckResult {
        name: check_config.name.to_string(),
        state,
    }
}

async fn print_states(check_configs: &[CheckConfig]) {
    print!("[");
    let mut entries = vec![];
    for check_config in check_configs {
        entries.push(format!("{}", check_host(check_config).await));
    }
    entries.push(format!(
        "{{\"full_text\":\"check@{}\"}}",
        chrono::Local::now().format("%H:%M")
    ));
    println!("{}],", entries.join(","));
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    println!("{{\"version\":1,\"click_events\":false}}");
    println!("[");
    loop {
        let home_dir = dirs::home_dir().unwrap();
        let config = match std::fs::read_to_string(format!(
            "{}/.checkbar.toml",
            home_dir.to_str().unwrap_or("")
        )) {
            Ok(config) => match toml::from_str(config.as_str()) {
                Ok(config) => config,
                Err(_e) => Config {
                    interval: None,
                    checks: vec![],
                },
            },
            Err(_e) => Config {
                interval: None,
                checks: vec![],
            },
        };

        print_states(&config.checks).await;
        std::thread::sleep(Duration::from_secs(config.interval.unwrap_or(60)));
    }
}
