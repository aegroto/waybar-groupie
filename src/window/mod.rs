use serde_json::Value;

use crate::error::Error;

mod display;
mod fetch;

pub struct WindowData {
    pub title: String,
    pub app_name: String,
    pub active: bool,
    pub group_index: usize,
}

impl WindowData {
    fn from_json_data(data: Value, active_window_address: String) -> Result<Self, Error> {
        let address = data["address"]
            .as_str()
            .ok_or(Error::WindowDataParsing("Non-string window address"))?
            .to_owned();

        let group_index: usize = {
            data["grouped"]
                .as_array()
                .ok_or(Error::WindowDataParsing("Non-array window group ids"))?
                .iter()
                .map(|value| {
                    value
                        .as_str()
                        .map(str::to_owned)
                        .ok_or(Error::WindowDataParsing("Non-array window group ids"))
                })
                .collect::<Result<Vec<String>, Error>>()?
                .into_iter()
                .position(|group_address| address == group_address)
                .unwrap_or(usize::MAX)
        };

        let title = data["title"]
            .as_str()
            .ok_or(Error::WindowDataParsing("Non-string window title"))?
            .to_owned();

        Ok(Self {
            active: address == active_window_address,
            title,
            group_index,
            app_name: data["initialTitle"]
                .as_str()
                .ok_or(Error::WindowDataParsing(
                    "Non-string window initialTitle",
                ))?
                .to_owned(),
        })
    }
}
