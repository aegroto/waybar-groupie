use std::{
    io::{BufRead, BufReader},
    time::Duration,
};

use config::Config;
use error::Error;
use serde::Serialize;
use socket::connect_to_hyprland_socket;
use window::WindowData;

mod config;
mod error;
mod shell;
mod socket;
mod window;

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

    let active_windows = WindowData::fetch_active_windows_data()?;

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
