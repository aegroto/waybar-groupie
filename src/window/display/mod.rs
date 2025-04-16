use regex::Regex;

use crate::config::Config;

use super::WindowData;

#[cfg(test)]
mod test;

impl WindowData {
    pub fn as_display_str(&self, config: &Config, width: usize) -> String {
        format!(
            "<tt><span line_height=\"{}\" background=\"{}\"> {} </span></tt>",
            config.line_height,
            self.display_background_color(config),
            self.display_formatted_text(width)
        )
    }

    fn sanitize_text(text: &str) -> String {
        Regex::new(r"[<>]+").unwrap().replace_all(text, "").to_string()
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
            let (last_char_index, _) = text.char_indices().nth(width - 3).unwrap();
            text.truncate(last_char_index);
            text = format!("{}...", text);
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

        text = Self::sanitize_text(&text);
        log::trace!("Resulting sanitized text: {}", text);

        log::trace!("Resulting unformatted text length: {}", text.len());

        

        if self.active {
            format!("<b>{}</b>", text)
        } else {
            text
        }
    }

    fn display_background_color(&self, config: &Config) -> String {
        if self.active {
            config.active_background_color.clone()
        } else {
            config.background_color.clone()
        }
    }
}
