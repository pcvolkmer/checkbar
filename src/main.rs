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
    click_cmd: Option<String>,
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

#[derive(Deserialize)]
struct ClickEvent {
    name: String,
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
            "{{\"full_text\":\"{}\",\"name\":\"{}\",\"separator_block_width\":16,\"color\":\"{}\"}}",
            self.name, self.name, color
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

async fn get_config() -> Config {
    let home_dir = dirs::home_dir().unwrap();
    match std::fs::read_to_string(format!(
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
    }
}

async fn get_click_cmd(name: String) -> Option<String> {
    for check in get_config().await.checks {
        if check.name == name {
            return check.click_cmd;
        }
    }
    None
}

async fn run_click_cmd(cmd: String) {
    let _r = match std::process::Command::new("sh")
        .stdin(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
    {
        Ok(mut child) => {
            use std::io::Write;
            let _r = child.stdin.as_mut().unwrap().write_all(cmd.as_bytes());
            child.wait()
        }
        Err(e) => Err(e),
    };
}

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() {
    println!("{{\"version\":1,\"click_events\":true}}");
    println!("[");

    let inputs = tokio::task::spawn(async {
        let stdin = std::io::stdin();
        loop {
            let mut input = String::new();

            if stdin.read_line(&mut input).is_err() {
                continue;
            }

            if input.is_empty() {
                continue;
            }

            // Remove leading comma
            let input = input.replace(",{", "{");

            if let Ok(click_event) = serde_json::from_str::<ClickEvent>(input.as_str()) {
                if let Some(click_cmd) = get_click_cmd(click_event.name).await {
                    run_click_cmd(click_cmd).await;
                }
            }
        }
    });

    let checks = tokio::task::spawn(async {
        loop {
            let config = get_config().await;
            print_states(&config.checks).await;
            std::thread::sleep(Duration::from_secs(config.interval.unwrap_or(60)));
        }
    });

    let _r = tokio::join!(inputs, checks);
}
