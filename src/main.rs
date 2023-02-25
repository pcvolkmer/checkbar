use checkbar::{get_click_cmd, print_states, read_click_event, run_click_cmd, Config, MouseButton};
use serde_json::json;
use tokio::task;

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
                if click_event.button != MouseButton::Left {
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
            std::thread::sleep(config.interval);
        }
    });

    let _r = tokio::join!(inputs, checks);
}
