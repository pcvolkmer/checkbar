use std::process;
use std::time::Duration;

use serde::Deserialize;
use serde_json::json;
use tokio::task;

use crate::checker::{ActuatorChecker, CheckResult, HttpChecker, TcpChecker};
use crate::config::{get_config, CheckConfig, CheckType};

mod checker;
mod config;

#[derive(Deserialize)]
struct ClickEvent {
    name: String,
    button: u8,
}

async fn check_host(check_config: &CheckConfig) -> CheckResult {
    match check_config.check_type {
        Some(CheckType::Actuator) => ActuatorChecker::new(check_config).check().await,
        Some(CheckType::Tcp) => TcpChecker::new(check_config).check().await,
        _ => HttpChecker::new(check_config).check().await,
    }
}

async fn print_states(check_configs: &[CheckConfig]) {
    print!("[");
    let mut entries = vec![];
    for check_config in check_configs {
        entries.push(format!("{}", check_host(check_config).await));
    }
    entries.push(
        json!({
            "full_text": chrono::Local::now().format("%H:%M").to_string()
        })
        .to_string(),
    );
    println!("{}],", entries.join(","));
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
    println!(
        "{}",
        json!({
            "version": 1,
            "click_events": true
        })
    );
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
            let interval = match config.interval {
                Some(value) => value,
                _ => Duration::from_secs(60),
            };
            std::thread::sleep(interval);
        }
    });

    let _r = tokio::join!(inputs, checks);
}
