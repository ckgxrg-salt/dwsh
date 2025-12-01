//! Simple library for reading a base16 toml colourscheme.

use iced::Color;
use iced::theme::{Palette, Theme};
use serde::{Deserialize, Deserializer};
use std::error::Error;
use std::path::{Path, PathBuf};

/// Structure of a base16 colourscheme
#[derive(Deserialize, PartialEq, Debug)]
pub struct ColourScheme {
    #[serde(rename = "base00")]
    #[serde(deserialize_with = "deserialize_color")]
    pub bg: Color,
    #[serde(rename = "base01")]
    #[serde(deserialize_with = "deserialize_color")]
    pub light_bg: Color,
    #[serde(rename = "base02")]
    #[serde(deserialize_with = "deserialize_color")]
    pub select_bg: Color,
    #[serde(rename = "base03")]
    #[serde(deserialize_with = "deserialize_color")]
    pub invisible: Color,
    #[serde(rename = "base04")]
    #[serde(deserialize_with = "deserialize_color")]
    pub dark_fg: Color,
    #[serde(rename = "base05")]
    #[serde(deserialize_with = "deserialize_color")]
    pub fg: Color,
    #[serde(rename = "base06")]
    #[serde(deserialize_with = "deserialize_color")]
    pub light_fg: Color,
    #[serde(rename = "base07")]
    #[serde(deserialize_with = "deserialize_color")]
    pub lightest_fg: Color,
    #[serde(rename = "base08")]
    #[serde(deserialize_with = "deserialize_color")]
    pub red: Color,
    #[serde(rename = "base09")]
    #[serde(deserialize_with = "deserialize_color")]
    pub orange: Color,
    #[serde(rename = "base0A")]
    #[serde(deserialize_with = "deserialize_color")]
    pub yellow: Color,
    #[serde(rename = "base0B")]
    #[serde(deserialize_with = "deserialize_color")]
    pub green: Color,
    #[serde(rename = "base0C")]
    #[serde(deserialize_with = "deserialize_color")]
    pub cyan: Color,
    #[serde(rename = "base0D")]
    #[serde(deserialize_with = "deserialize_color")]
    pub blue: Color,
    #[serde(rename = "base0E")]
    #[serde(deserialize_with = "deserialize_color")]
    pub magenta: Color,
    #[serde(rename = "base0F")]
    #[serde(deserialize_with = "deserialize_color")]
    pub dark_red: Color,
}

impl Default for ColourScheme {
    fn default() -> Self {
        Self {
            bg: Color::BLACK,
            light_bg: Color::BLACK,
            select_bg: Color::BLACK,
            invisible: Color::BLACK,
            dark_fg: Color::BLACK,
            fg: Color::BLACK,
            light_fg: Color::BLACK,
            lightest_fg: Color::BLACK,
            red: Color::BLACK,
            orange: Color::BLACK,
            yellow: Color::BLACK,
            green: Color::BLACK,
            cyan: Color::BLACK,
            blue: Color::BLACK,
            magenta: Color::BLACK,
            dark_red: Color::BLACK,
        }
    }
}

fn deserialize_color<'de, D>(deserializer: D) -> Result<Color, D::Error>
where
    D: Deserializer<'de>,
{
    let buf = String::deserialize(deserializer)?;
    Color::parse(&buf).ok_or(serde::de::Error::custom("invalid colour format"))
}

/// Gets the default path to the colourscheme configuration.
/// The default path is: `%XDG_CONFIG_HOME%/dwsh/theme.toml` or `~/.config/dwsh/theme.toml`.
#[must_use]
pub fn get_config() -> PathBuf {
    if let Ok(path) = std::env::var("XDG_CONFIG_HOME") {
        PathBuf::from(path).join("dwsh/theme.toml")
    } else if let Some(path) = std::env::home_dir() {
        path.join(".config/dwsh/theme.toml")
    } else {
        PathBuf::from("theme.toml")
    }
}

impl ColourScheme {
    /// Reads a toml file to a colourscheme configuration.
    ///
    /// # Errors
    /// fs errors and toml parsing errors are passed as is.
    pub fn from_path(path: &Path) -> Result<ColourScheme, Box<dyn Error>> {
        let file = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&file)?)
    }

    /// Converts self to an iced [`Theme`].
    #[must_use]
    pub fn to_iced_theme(&self) -> iced::theme::Theme {
        Theme::custom(
            "dwsh".to_string(),
            Palette {
                background: self.bg,
                text: self.fg,
                primary: self.blue,
                success: self.green,
                danger: self.red,
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_color() {
        let deserialised = ColourScheme::from_path(&PathBuf::from("test/test.toml")).unwrap();
        assert_eq!(deserialised, ColourScheme::default());
    }
}
