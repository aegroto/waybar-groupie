use std::{
    io::{BufRead, BufReader},
    process::Command,
    time::Duration,
};

use config::Config;
use error::Error;
use regex::Regex;
use serde::Serialize;
use serde_json::Value;
use socket::connect_to_hyprland_socket;

mod config;
mod error;
mod socket;

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
            Ok(value) => {
                log::debug!("Printing output: {value}");
                println!("{value}");
            }
            Err(err) => panic!("Failed output JSON serialization: {err:?}"),
        }
    }
}

fn main() {
    env_logger::init();

    let config = Config::load_from_env_path();

    log::debug!("Running with config: {config:?}");

    let stream = loop {
        if let Some(value) = connect_to_hyprland_socket(&config) {
            break value;
        }

        Output::with_error("Couldn't connect to Hyprland socket, retrying...").print_out();

        std::thread::sleep(Duration::from_secs(1));
    };

    let event_reader = BufReader::new(stream);

    for _ in event_reader.lines() {
        match run_update(&config) {
            Ok(update) => update.print_out(),
            Err(error) => Output::with_error(&error.as_string()).print_out(),
        };
    }
}

fn run_update(config: &Config) -> Result<Output, Error> {
    log::debug!("Running update...");

    let active_windows = fetch_active_windows_data()?;

    if active_windows.is_empty() {
        return Ok(Output {
            text: config.empty_text.clone(),
        });
    }

    let separator = &config.separator;
    let window_width = {
        let available_width = config.width - (separator.len() * (active_windows.len() - 1));
        available_width / active_windows.len()
    };

    let text = active_windows
        .iter()
        .map(|window| window.as_display_str(&config, window_width))
        .collect::<Vec<String>>()
        .join(separator);

    log::trace!("Output text length: {}", text.len());

    Ok(Output { text })
}

struct ActiveWindow {
    pub title: String,
    pub app_name: String,
    pub active: bool,
    pub group_index: usize,
}

impl ActiveWindow {
    fn from_json_data(data: Value, active_window_address: String) -> Result<Self, Error> {
        let address = data["address"]
            .as_str()
            .ok_or(Error::WindowDataParsingError("Non-string window address"))?
            .to_owned();

        let group_index: usize = {
            data["grouped"]
                .as_array()
                .ok_or(Error::WindowDataParsingError("Non-array window group ids"))?
                .to_owned()
                .into_iter()
                .map(|value| {
                    value
                        .as_str()
                        .map(str::to_owned)
                        .ok_or(Error::WindowDataParsingError("Non-array window group ids"))
                })
                .collect::<Result<Vec<String>, Error>>()?
                .into_iter()
                .position(|group_id| address == group_id)
                .ok_or(Error::WindowDataParsingError(
                    "Unable to find window id in group data",
                ))?
        };

        let title = data["title"]
            .as_str()
            .ok_or(Error::WindowDataParsingError("Non-string window title"))?
            .to_owned();

        Ok(Self {
            active: address == active_window_address,
            title,
            group_index,
            app_name: data["initialTitle"]
                .as_str()
                .ok_or(Error::WindowDataParsingError(
                    "Non-string window initialTitle",
                ))?
                .to_owned(),
        })
    }

    fn as_display_str(&self, config: &Config, width: usize) -> String {
        format!(
            "<tt><span line_height=\"{}\" background=\"{}\"> {} </span></tt>",
            config.line_height,
            self.display_background_color(),
            self.display_formatted_text(width)
        )
    }

    fn display_formatted_text(&self, width: usize) -> String {
        let appless_title = self.title.replace(&format!(" - {}", self.app_name), "");

        let mut text = format!("{}: {}", self.app_name, appless_title);

        let visible_text = {
            let re = Regex::new(r"[^ -~]+").unwrap();
            re.replace_all(&text, "")
        };

        if visible_text.len() > width {
            log::trace!("Text '{}' out of bounds, truncating...", visible_text);
            text = format!("{}...", &text[0..width - 3]);
        } else {
            let padding = " ".repeat((width - visible_text.len()) / 2);
            log::trace!(
                "Padding for '{}': {} (visible length: {}, target width: {})",
                visible_text,
                padding.len(),
                visible_text.len(),
                width
            );

            text = format!("{}{}{}", padding, visible_text, padding);
        }

        log::trace!("Resulting unformatted text length: {}", text.len());

        let formatted_text = if self.active {
            format!("{}", text)
        } else {
            text
        };

        formatted_text
    }

    fn display_background_color(&self) -> &str {
        if self.active {
            "#FFFFFF66"
        } else {
            "#99999966"
        }
    }
}

fn fetch_active_windows_data() -> Result<Vec<ActiveWindow>, Error> {
    let active_workspace_data = json_cmd("hyprctl", &["activeworkspace", "-j"]);

    let active_workspace_id = active_workspace_data["id"]
        .as_i64()
        .ok_or(Error::DataFetchError("Cannot get id from workspace data"))?;

    let active_window_address: String = active_workspace_data["lastwindow"]
        .as_str()
        .ok_or(Error::DataFetchError(
            "Cannot get active window address from workspace data",
        ))?
        .to_owned();

    let windows_fetch_result = json_cmd("hyprctl", &["clients", "-j"]);

    let active_windows_data: Vec<Value> = windows_fetch_result
        .as_array()
        .ok_or(Error::DataFetchError(
            "Cannot convert hyprctl clients output as array",
        ))?
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
        .map(|data| ActiveWindow::from_json_data(data, active_window_address.clone()))
        .collect::<Result<Vec<ActiveWindow>, Error>>()?;

    windows.sort_by(|w1, w2| w1.group_index.cmp(&w2.group_index));

    Ok(windows)
}
