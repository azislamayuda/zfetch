use crate::cli::LogoAlign;

use serde::Deserialize;
use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

#[derive(Debug, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub logo: LogoConfig,
    pub theme: ThemeConfig,
}

#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct LogoConfig {
    pub enabled: bool,
    pub source: Option<String>,
    pub align: LogoAlign,
}

impl Default for LogoConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            source: None,
            align: LogoAlign::Center,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NamedColor {
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    Gray,
}

impl NamedColor {
    pub fn ansi(self) -> &'static str {
        match self {
            Self::Red => "1;31",
            Self::Green => "1;32",
            Self::Yellow => "1;33",
            Self::Blue => "1;34",
            Self::Magenta => "1;35",
            Self::Cyan => "1;36",
            Self::White => "1;37",
            Self::Gray => "90",
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ThemeConfig {
    pub color: bool,
    pub logo_color: NamedColor,
    pub heading_color: NamedColor,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            color: true,
            logo_color: NamedColor::Red,
            heading_color: NamedColor::Cyan,
        }
    }
}

fn default_config_path() -> Option<PathBuf> {
    if let Some(xdg) = env::var_os("XDG_CONFIG_HOME") {
        return Some(PathBuf::from(xdg).join("zfetch/config.jsonc"));
    }

    env::var_os("HOME").map(|home| PathBuf::from(home).join(".config/zfetch/config.jsonc"))
}

impl Config {
    pub fn load(explicit: Option<&Path>) -> Result<Self, String> {
        let Some(path) = explicit.map(Path::to_path_buf).or_else(default_config_path) else {
            return Ok(Self::default());
        };

        match fs::read_to_string(&path) {
            Ok(content) => json5::from_str(&content)
                .map_err(|error| format!("Invalid config {}: {error}", path.display())),

            Err(error) if error.kind() == io::ErrorKind::NotFound && explicit.is_none() => {
                Ok(Self::default())
            }

            Err(error) => Err(format!("Cannot read config {}: {error}", path.display())),
        }
    }

    pub fn logo_path(&self) -> Option<PathBuf> {
        let source = self.logo.source.as_deref()?;

        if source == "~" {
            return env::var_os("HOME").map(PathBuf::from);
        }

        if let Some(relative) = source.strip_prefix("~/") {
            return Some(
                env::var_os("HOME")
                    .map(PathBuf::from)
                    .unwrap_or_default()
                    .join(relative),
            );
        }

        Some(PathBuf::from(source))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_jsonc_with_comments() {
        let content = r#"
        {
            // Logo configuration
            "logo": {
                "enabled": true,
                "align": "bottom",
            },
            "theme": {
                "color": false,
                "logo_color": "magenta",
            },
        }
        "#;

        let config: Config = json5::from_str(content).unwrap();

        assert!(config.logo.enabled);
        assert!(matches!(config.logo.align, LogoAlign::Bottom));
        assert!(!config.theme.color);
        assert!(matches!(config.theme.logo_color, NamedColor::Magenta));
    }

    #[test]
    fn defaults_for_missing_fields() {
        let config: Config = json5::from_str("{}").unwrap();

        assert!(config.logo.enabled);
        assert!(matches!(config.logo.align, LogoAlign::Center));
        assert!(config.theme.color);
    }

    #[test]
    fn rejects_invalid_alignment() {
        let result = json5::from_str::<Config>(r#"{"logo": {"align": "middle"}}"#);

        assert!(result.is_err());
    }
}
