mod checker;
mod config;

use std::process;

pub use config::{CheckConfig, Config};
use serde::Deserialize;
use serde_json::json;
use serde_repr::Deserialize_repr;

use checker::check_host;

#[derive(Debug, Deserialize_repr, PartialEq)]
#[repr(u8)]
pub enum MouseButton {
    Left = 1,
    Middle = 2,
    Right = 3,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct ClickEvent {
    pub name: String,
    pub button: MouseButton,
}

pub async fn print_states(check_configs: &[CheckConfig]) {
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

pub async fn get_click_cmd(name: String) -> Option<String> {
    for check in Config::read().checks {
        if check.name == name {
            return check.click_cmd;
        }
    }
    None
}

pub async fn run_click_cmd(cmd: String) {
    if let Ok(mut child) = process::Command::new("sh")
        .stdin(process::Stdio::piped())
        .spawn()
    {
        use std::io::Write;
        let _ = child.stdin.as_mut().unwrap().write_all(cmd.as_bytes());
    };
}

pub fn read_click_event() -> Result<ClickEvent, String> {
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
    Err("Cannot read click event".to_string())
}

#[cfg(test)]
mod tests {
    use crate::{ClickEvent, MouseButton};

    #[test]
    fn test_should_deserialize_click_event() {
        let actual = serde_json::from_str::<ClickEvent>(r#"{"name": "test", "button": 1}"#);
        let expected = ClickEvent {
            name: "test".to_string(),
            button: MouseButton::Left,
        };

        assert!(actual.is_ok());
        let actual = actual.unwrap();
        assert_eq!(actual, expected);
    }
}
