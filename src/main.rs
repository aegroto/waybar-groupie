use std::{
    io::{BufRead, BufReader},
    os::unix::net::UnixStream,
    process::Command,
    time::Duration,
};

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

impl Output {
    pub fn with_error(text: &str) -> Self {
        Self {
            text: format!("ERROR: {text}"),
        }
    }

    pub fn print_out(&self) {
        match serde_json::to_string(&self) {
            Ok(value) => println!("{value}"),
            Err(err) => panic!("Failed output JSON serialization: {err:?}"),
        }
    }
}

fn main() {
    let stream = loop {
        if let Some(value) = connect_to_hyprland_socket() {
            break value;
        }

        Output::with_error("Couldn't connect to Hyprland socket, retrying...").print_out();

        std::thread::sleep(Duration::from_secs(1));
    };

    let event_reader = BufReader::new(stream);

    for _ in event_reader.lines() {
        run_update().print_out();
    }
}

fn connect_to_hyprland_socket() -> Option<UnixStream> {
    let socket_address = format!(
        "{}/hypr/{}/.socket2.sock",
        std::env::var("XDG_RUNTIME_DIR").unwrap(),
        std::env::var("HYPRLAND_INSTANCE_SIGNATURE").unwrap(),
    );

    Some(match UnixStream::connect(socket_address) {
        Ok(socket) => socket,
        Err(err) => {
            Output::with_error(&format!("Failed to connect to Unix socket: {err:?}")).print_out();
            return None;
        }
    })
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
