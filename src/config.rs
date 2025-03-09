use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(default = "default_separator")]
    pub separator: String,

    #[serde(default = "default_socket_address")]
    pub socket_address: String,

    #[serde(default = "default_empty_text")]
    pub empty_text: String,

    #[serde(default = "default_width")]
    pub width: usize,

    #[serde(default = "default_line_height")]
    pub line_height: f32,
}

fn default_line_height() -> f32 {
    1.0
}

fn default_width() -> usize {
    100
}

fn default_separator() -> String {
    " || ".to_owned()
}

fn default_empty_text() -> String {
    "".to_owned()
}

fn default_socket_address() -> String {
    format!(
        "{}/hypr/{}/.socket2.sock",
        std::env::var("XDG_RUNTIME_DIR").unwrap(),
        std::env::var("HYPRLAND_INSTANCE_SIGNATURE").unwrap(),
    )
}

impl Config {
    pub fn load_from_env_path() -> Self {
        let config_path = std::env::var("GROUPIE_CONFIG_PATH")
            .unwrap_or(".config/groupie/config.json".to_owned());

        let file_content = match std::fs::read_to_string(config_path) {
            Ok(value) => value,
            Err(err) => {
                log::warn!("Couldn't load configuration file: {err}, using default values");
                "{}".to_owned()
            }
        };

        serde_json::from_str(&file_content).unwrap()
    }
}
