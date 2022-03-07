use serde::Deserialize;
use std::fmt::{Display, Formatter, Result};
use std::fs;
use std::process;
use std::time::Duration;
use tokio::task;

#[derive(Deserialize)]
struct Config {
    interval: Option<u64>,
    colors: Option<ColorConfig>,
    checks: Vec<CheckConfig>,
}

#[derive(Deserialize)]
struct ColorConfig {
    up: String,
    warn: String,
    down: String,
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
    button: u8,
}

struct CheckResult {
    name: String,
    state: CheckState,
}

impl Display for CheckResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let color_config = match get_config().colors {
            Some(color_config) => color_config,
            None => ColorConfig {
                up: String::from("#00FF00"),
                warn: String::from("#FFFF00"),
                down: String::from("#FF0000"),
            },
        };
        let color = match &self.state {
            CheckState::Up => color_config.up,
            CheckState::Warn => color_config.warn,
            CheckState::Down => color_config.down,
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

fn get_config() -> Config {
    let home_dir = dirs::home_dir().unwrap();
    match fs::read_to_string(format!(
        "{}/.checkbar.toml",
        home_dir.to_str().unwrap_or("")
    )) {
        Ok(config) => match toml::from_str(config.as_str()) {
            Ok(config) => config,
            Err(_e) => Config {
                interval: None,
                colors: None,
                checks: vec![],
            },
        },
        Err(_) => Config {
            interval: None,
            colors: None,
            checks: vec![],
        },
    }
}

async fn get_click_cmd(name: String) -> Option<String> {
    for check in get_config().checks {
        if check.name == name {
            return check.click_cmd;
        }
    }
    None
}

async fn run_click_cmd(cmd: String) {
    if let Ok(mut child) = process::Command::new("sh")
        .stdin(process::Stdio::piped())
        .spawn()
    {
        use std::io::Write;
        let _ = child.stdin.as_mut().unwrap().write_all(cmd.as_bytes());
    };
}

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() {
    println!("{{\"version\":1,\"click_events\":true}}");
    println!("[");

    let inputs = task::spawn(async {
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
                // Ignore click event if not left mouse button
                if click_event.button != 1 {
                    continue;
                };
                if let Some(click_cmd) = get_click_cmd(click_event.name).await {
                    run_click_cmd(click_cmd).await;
                }
            }
        }
    });

    let checks = task::spawn(async {
        loop {
            let config = get_config();
            print_states(&config.checks).await;
            std::thread::sleep(Duration::from_secs(config.interval.unwrap_or(60)));
        }
    });

    let _r = tokio::join!(inputs, checks);
}
