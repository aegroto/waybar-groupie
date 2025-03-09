use serde_json::Value;

use crate::{error::Error, shell::json_cmd};

impl super::WindowData {
    pub fn fetch_active_windows_data() -> Result<Vec<Self>, Error> {
        let active_workspace_data = json_cmd("hyprctl", &["activeworkspace", "-j"])?;

        let active_workspace_id = active_workspace_data["id"]
            .as_i64()
            .ok_or(Error::DataFetchError("Cannot get id from workspace data"))?;

        let active_window_address: String = active_workspace_data["lastwindow"]
            .as_str()
            .ok_or(Error::DataFetchError(
                "Cannot get active window address from workspace data",
            ))?
            .to_owned();

        let windows_fetch_result = json_cmd("hyprctl", &["clients", "-j"])?;

        let active_windows_json_data = windows_fetch_result
            .as_array()
            .ok_or(Error::DataFetchError(
                "Cannot convert hyprctl clients output as array",
            ))?
            .into_iter()
            .cloned()
            .filter(|window_data| {
                window_data["workspace"]["id"]
                    .as_i64()
                    .is_some_and(|value| value.eq(&active_workspace_id))
            })
            .collect::<Vec<Value>>();

        let mut windows = active_windows_json_data
            .iter()
            .cloned()
            .map(|data| Self::from_json_data(data, active_window_address.clone()))
            .collect::<Result<Vec<Self>, Error>>()?;

        windows.sort_by(|w1, w2| w1.group_index.cmp(&w2.group_index));

        Ok(windows)
    }
}
