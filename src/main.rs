mod checker;
mod config;

use std::process;
use std::time::Duration;

use serde::Deserialize;
use serde_json::json;
use tokio::task;

use crate::checker::check_host;
use crate::config::{CheckConfig, Config};

#[derive(Deserialize)]
struct ClickEvent {
    name: String,
    button: u8,
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
    for check in Config::read().checks {
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

fn read_click_event() -> Result<ClickEvent, ()> {
    let stdin = std::io::stdin();
    let mut input = String::new();

    if stdin.read_line(&mut input).is_ok() {
        // Return click event after removing leading comma
        if let Ok(click_event) =
            serde_json::from_str::<ClickEvent>(input.replace(",{", "{").as_str())
        {
            return Ok(click_event);
        }
    }
    Err(())
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
        loop {
            if let Ok(click_event) = read_click_event() {
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
            let config = Config::read();
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
