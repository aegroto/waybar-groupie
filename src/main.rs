use std::{process::Command, time::Duration};

use serde::Serialize;
use serde_json::Value;

fn json_cmd(command: &str, args: &[&str]) -> Value {
    let output_str = String::from_utf8(
        Command::new(command)
            .args(args.to_vec())
            .output()
            .expect(&format!(
                "Unable to run command {command} with args {args:?}"
            ))
            .stdout,
    )
    .expect(&format!(
        "Unable to convert command {command} {args:?} output to string"
    ));

    serde_json::from_str(&output_str)
        .expect("Unable to parse command {command} {args:?} output as JSON")
}

#[derive(Serialize)]
struct Output {
    text: String,
}

fn main() {
    loop {
        let output = run_update();
        match serde_json::to_string(&output) {
            Ok(value) => println!("{value}"),
            Err(err) => println!("Failed output JSON serialization: {err:?}"),
        }

        std::thread::sleep(Duration::from_secs(1));
    }
}

fn run_update() -> Output {
    let active_windows = fetch_active_windows_data();

    let separator = " || ";
    let text = active_windows
        .iter()
        .map(ActiveWindow::as_display_str)
        .collect::<Vec<String>>()
        .join(separator);

    Output { text }
}

struct ActiveWindow {
    pub title: String,
    pub app_name: String,
    pub active: bool,
    pub group_index: usize,
}

impl ActiveWindow {
    fn from_json_data(data: Value, active_window_title: String) -> Self {
        let group_index: usize = {
            let id = data["address"]
                .as_str()
                .expect("Non-string window id")
                .to_owned();

            data["grouped"]
                .as_array()
                .expect("Non-array window group ids")
                .to_owned()
                .into_iter()
                .map(|value| {
                    value
                        .as_str()
                        .expect("Cannot cast window group id to string")
                        .to_owned()
                })
                .position(|group_id| id == group_id)
                .expect("Unable to find window id in group data")
        };

        let title = data["title"]
            .as_str()
            .expect("Non-string window title")
            .to_owned();

        Self {
            active: title == active_window_title,
            title,
            group_index,
            app_name: data["initialTitle"]
                .as_str()
                .expect("Non-string window initialTitle")
                .to_owned(),
        }
    }

    fn as_display_str(&self) -> String {
        let appless_title = self.title.replace(&format!(" - {}", self.app_name), "");

        let window_content = if self.active {
            format!("<b>{}</b>: {}", self.app_name, appless_title)
        } else {
            format!("{}", self.app_name)
        };

        format!("<span line_height=\"1.5\">{}</span>", window_content)
    }
}

fn fetch_active_windows_data() -> Vec<ActiveWindow> {
    let active_workspace_data = json_cmd("hyprctl", &["activeworkspace", "-j"]);

    let active_workspace_id = active_workspace_data["id"]
        .as_i64()
        .expect("Cannot get id from workspace data");

    let active_window_title: String = active_workspace_data["lastwindowtitle"]
        .as_str()
        .expect("Cannot get active window title from workspace data")
        .to_owned();

    let windows_fetch_result = json_cmd("hyprctl", &["clients", "-j"]);

    let active_windows_data: Vec<Value> = windows_fetch_result
        .as_array()
        .expect("Cannot convert hyprctl clients output as array")
        .into_iter()
        .cloned()
        .filter(|window_data| {
            window_data["workspace"]["id"]
                .as_i64()
                .expect("Missing id in window data")
                .eq(&active_workspace_id)
        })
        .collect();

    let mut windows = active_windows_data
        .iter()
        .cloned()
        .map(|data| ActiveWindow::from_json_data(data, active_window_title.clone()))
        .collect::<Vec<ActiveWindow>>();
    windows.sort_by(|w1, w2| w1.group_index.cmp(&w2.group_index));
    windows
}
