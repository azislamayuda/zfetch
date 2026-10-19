use crate::cli::LogoAlign;
use serde::Deserialize;
use std::{
    collections::HashSet,
    env, fs, io,
    path::{Path, PathBuf},
};

#[derive(Debug, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub logo: LogoConfig,
    pub theme: ThemeConfig,
    pub display: DisplayConfig,
    pub modules: ModulesConfig,
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
    pub label_color: NamedColor,
    pub value_color: NamedColor,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            color: true,
            logo_color: NamedColor::Red,
            heading_color: NamedColor::Cyan,
            label_color: NamedColor::White,
            value_color: NamedColor::Gray,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct DisplayConfig {
    pub show_efi: bool,
    pub show_percentage: bool,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            show_efi: false,
            show_percentage: true,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ModulesConfig {
    pub system: Vec<String>,
    pub hardware: Vec<String>,
}

impl Default for ModulesConfig {
    fn default() -> Self {
        Self {
            system: vec![
                "os", "kernel", "arch", "shell", "de", "session", "wm", "terminal", "packages",
            ]
            .into_iter()
            .map(str::to_string)
            .collect(),

            hardware: vec![
                "host", "bios", "display", "cpu", "gpu", "memory", "swap", "disk", "local_ip",
                "battery", "uptime",
            ]
            .into_iter()
            .map(str::to_string)
            .collect(),
        }
    }
}

const SYSTEM_MODULES: &[&str] = &[
    "os", "kernel", "arch", "shell", "de", "session", "wm", "terminal", "packages",
];

const HARDWARE_MODULES: &[&str] = &[
    "host", "bios", "display", "cpu", "gpu", "memory", "swap", "disk", "local_ip", "battery",
    "uptime",
];

fn validate_modules(group: &str, modules: &[String], supported: &[&str]) -> Result<(), String> {
    let mut seen = HashSet::new();

    for module in modules {
        if !supported.contains(&module.as_str()) {
            return Err(format!("Unknown module '{module}' in modules.{group}"));
        }

        if !seen.insert(module) {
            return Err(format!("Duplicate module '{module}' in modules.{group}"));
        }
    }

    Ok(())
}

fn default_config_path() -> Option<PathBuf> {
    if let Some(xdg) = env::var_os("XDG_CONFIG_HOME") {
        return Some(PathBuf::from(xdg).join("zfetch/config.jsonc"));
    }

    env::var_os("HOME").map(|home| PathBuf::from(home).join(".config/zfetch/config.jsonc"))
}

impl Config {
    pub fn validate(&self) -> Result<(), String> {
        validate_modules("system", &self.modules.system, SYSTEM_MODULES)?;

        validate_modules("hardware", &self.modules.hardware, HARDWARE_MODULES)?;

        Ok(())
    }

    pub fn load(explicit: Option<&Path>) -> Result<Self, String> {
        let Some(path) = explicit.map(Path::to_path_buf).or_else(default_config_path) else {
            return Ok(Self::default());
        };

        let config: Self = match fs::read_to_string(&path) {
            Ok(content) => json5::from_str(&content)
                .map_err(|error| format!("Invalid config {}: {error}", path.display()))?,

            Err(error) if error.kind() == io::ErrorKind::NotFound && explicit.is_none() => {
                Self::default()
            }

            Err(error) => {
                return Err(format!("Cannot read config {}: {error}", path.display()));
            }
        };

        config.validate()?;
        Ok(config)
    }

    pub fn logo_path(&self) -> Option<PathBuf> {
        let source = self.logo.source.as_deref()?;

        if source == "~" {
            return env::var_os("HOME").map(PathBuf::from);
        }

        if let Some(relative) = source.strip_prefix("~/") {
            return env::var_os("HOME").map(|home| PathBuf::from(home).join(relative));
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
            // Custom logo
            "logo": {
                "align": "bottom",
            },
            "theme": {
                "logo_color": "magenta",
                "value_color": "gray",
            },
            "display": {
                "show_efi": true,
            },
            "modules": {
                "system": ["os", "kernel"],
                "hardware": ["cpu", "gpu"],
            },
        }
        "#;

        let config: Config = json5::from_str(content).unwrap();

        config.validate().unwrap();
        assert!(matches!(config.logo.align, LogoAlign::Bottom));
        assert!(config.display.show_efi);
        assert_eq!(config.modules.system[0], "os");
    }

    #[test]
    fn defaults_for_missing_fields() {
        let config: Config = json5::from_str("{}").unwrap();

        assert!(config.logo.enabled);
        assert!(matches!(config.logo.align, LogoAlign::Center));
        assert!(config.theme.color);
        assert!(!config.display.show_efi);
        assert!(config.display.show_percentage);
        assert!(!config.modules.hardware.is_empty());
    }

    #[test]
    fn rejects_invalid_alignment() {
        let result = json5::from_str::<Config>(r#"{"logo": {"align": "middle"}}"#);

        assert!(result.is_err());
    }

    #[test]
    fn rejects_unknown_module() {
        let mut config = Config::default();
        config.modules.system.push("invalid".into());

        assert!(config.validate().is_err());
    }

    #[test]
    fn rejects_duplicate_module() {
        let mut config = Config::default();
        config.modules.hardware.push("cpu".into());

        assert!(config.validate().is_err());
    }
}
