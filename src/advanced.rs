//! v0.3.0 advanced CLI operations. Everything here is offline on normal runs.
use crate::{
    cli::{Options, Preset},
    config::{Config, HARDWARE_MODULES, NamedColor, SYSTEM_MODULES, default_config_path},
    system::SystemInfo,
};
use serde_json::{Map, Value, json};
use std::{
    collections::HashSet,
    env, fs,
    io::{IsTerminal, Write},
    path::PathBuf,
};

pub const THEMES: &[&str] = &[
    "default",
    "monochrome",
    "nord",
    "gruvbox",
    "rust",
    "high-contrast",
];

pub fn list_modules() {
    println!("SYSTEM:  {}", SYSTEM_MODULES.join(", "));
    println!("HARDWARE: {}", HARDWARE_MODULES.join(", "));
}

pub fn list_themes() {
    let use_color = std::io::stdout().is_terminal() && env::var_os("NO_COLOR").is_none();
    println!("Available ANSI themes (logo / heading / label / value):");
    for name in THEMES {
        let mut config = Config::default();
        // Every listed name is a supported theme by construction.
        if apply_theme(&mut config, name).is_err() {
            continue;
        }
        let colors = [
            config.theme.logo_color,
            config.theme.heading_color,
            config.theme.label_color,
            config.theme.value_color,
        ];
        print!("  {name:<14} ");
        for color in colors {
            if use_color {
                print!("\x1b[{}m██\x1b[0m ", color.ansi());
            } else {
                print!("[{}] ", color.ansi());
            }
        }
        println!();
    }
    println!("Try: zfetch --theme nord --layout classic");
    println!("Themes use the existing eight ANSI named colors, not truecolor palettes.");
}

fn validate_names(names: &[String]) -> Result<(), String> {
    for name in names {
        if !SYSTEM_MODULES.contains(&name.as_str()) && !HARDWARE_MODULES.contains(&name.as_str()) {
            return Err(format!(
                "Unknown module: '{name}'. Run zfetch --list-modules."
            ));
        }
    }
    Ok(())
}

fn apply_theme(config: &mut Config, name: &str) -> Result<(), String> {
    use NamedColor::{Blue, Cyan, Gray, Green, Red, White, Yellow};
    let (logo, heading, label, value) = match name {
        "default" => (Red, Cyan, White, Gray),
        "monochrome" => (White, White, White, Gray),
        "nord" => (Cyan, Blue, White, Gray),
        "gruvbox" => (Yellow, Green, White, Gray),
        "rust" => (Red, Yellow, White, Gray),
        "high-contrast" => (Yellow, Cyan, White, White),
        _ => {
            return Err(format!(
                "Unknown theme: '{name}'. Run zfetch --list-themes."
            ));
        }
    };
    config.theme.logo_color = logo;
    config.theme.heading_color = heading;
    config.theme.label_color = label;
    config.theme.value_color = value;
    Ok(())
}

/// CLI settings apply to an in-memory config and never overwrite the user's JSONC.
/// Selection precedes `system::collect()`, so unselected modules are never detected.
pub fn configure(config: &mut Config, options: &Options) -> Result<(), String> {
    if let Some(preset) = options.preset {
        match preset {
            Preset::Minimal => {
                config.modules.system = ["os", "kernel"].iter().map(|x| (*x).into()).collect();
                config.modules.hardware = ["cpu", "memory"].iter().map(|x| (*x).into()).collect();
                config.logo.enabled = false;
            }
            Preset::Default => {
                config.modules = Config::default().modules;
            }
            Preset::Full => {
                config.modules = Config::default().modules;
                config.display.show_efi = true;
            }
        }
    }
    if let Some(names) = &options.only {
        validate_names(names)?;
        config.modules.system = names
            .iter()
            .filter(|n| SYSTEM_MODULES.contains(&n.as_str()))
            .cloned()
            .collect();
        config.modules.hardware = names
            .iter()
            .filter(|n| HARDWARE_MODULES.contains(&n.as_str()))
            .cloned()
            .collect();
    }
    if let Some(names) = &options.exclude {
        validate_names(names)?;
        let rejected: HashSet<&str> = names.iter().map(String::as_str).collect();
        config
            .modules
            .system
            .retain(|name| !rejected.contains(name.as_str()));
        config
            .modules
            .hardware
            .retain(|name| !rejected.contains(name.as_str()));
    }
    if let Some(theme) = &options.theme {
        apply_theme(config, theme)?;
    }
    config.validate()
}

/// JSON is a machine-readable contract: strings retain the v0.2 detector values;
/// disk usage is exported separately as numeric byte counters.
pub fn json_document(info: &SystemInfo, config: &Config) -> Value {
    let on = |key| config.modules.enabled(key);
    let mut system = Map::new();
    let mut hardware = Map::new();
    for (key, value) in [
        ("os", &info.os),
        ("kernel", &info.kernel),
        ("arch", &info.arch),
        ("shell", &info.shell),
        ("de", &info.desktop),
        ("session", &info.session),
        ("wm", &info.wm),
        ("terminal", &info.terminal),
        ("packages", &info.packages),
    ] {
        if on(key) {
            system.insert(key.into(), json!(value));
        }
    }
    // JSON v1 contract: memory has no percentage, swap keeps it.
    for (key, value) in [
        ("host", info.host.clone()),
        ("bios", info.bios.clone()),
        ("cpu", info.cpu.clone()),
        ("gpu", info.gpu.clone()),
        ("memory", info.memory_text(false)),
        ("swap", info.swap_text(true)),
        ("local_ip", info.local_ip.clone()),
        ("uptime", info.uptime.clone()),
    ] {
        if on(key) {
            hardware.insert(key.into(), json!(value));
        }
    }
    if on("display") {
        hardware.insert("display".into(), json!(&info.displays));
    }
    if on("disk") {
        let disks: Vec<Value> = info
            .disks
            .iter()
            .filter(|d| config.display.show_efi || d.mount_point != "/boot/efi")
            .map(|d| {
                json!({
                    "mount_point": d.mount_point, "filesystem": d.filesystem,
                    "used_bytes": d.usage.used, "total_bytes": d.usage.total
                })
            })
            .collect();
        hardware.insert("disk".into(), json!(disks));
    }
    if on("battery") {
        let battery = info.battery.as_ref().map(|b| b.formatted(true));
        hardware.insert("battery".into(), json!(battery));
    }
    json!({
        "schema_version": 1,
        "zfetch_version": env!("CARGO_PKG_VERSION"),
        "identity": {"username": info.username, "hostname": info.hostname},
        "system": system,
        "hardware": hardware
    })
}

const DEFAULT_JSONC: &str = r#"{
  // zfetch configuration; edit this file to customize your terminal.
  "logo": {"enabled": true, "align": "center"},
  "theme": {"color": true, "logo_color": "red", "heading_color": "cyan", "label_color": "white", "value_color": "gray"},
  "display": {"show_efi": false, "show_percentage": true},
  "modules": {
    "system": ["os", "kernel", "arch", "shell", "de", "session", "wm", "terminal", "packages"],
    "hardware": ["host", "bios", "display", "cpu", "gpu", "memory", "swap", "disk", "local_ip", "battery", "uptime"]
  }
}
"#;

fn config_path() -> Result<PathBuf, String> {
    default_config_path().ok_or_else(|| "HOME and XDG_CONFIG_HOME are unset".into())
}

pub fn init_config() -> Result<(), String> {
    let path = config_path()?;
    let parent = path.parent().ok_or("Invalid config path")?;
    fs::create_dir_all(parent)
        .map_err(|e| format!("Could not create {}: {e}", parent.display()))?;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|e| {
            format!(
                "Cannot create {} (existing files are never overwritten): {e}",
                path.display()
            )
        })?;
    file.write_all(DEFAULT_JSONC.as_bytes())
        .map_err(|e| format!("Cannot write config: {e}"))?;
    println!("Config created: {}", path.display());
    Ok(())
}

pub fn validate_config(path: Option<&std::path::Path>) -> Result<(), String> {
    let path = match path {
        Some(p) => p.to_path_buf(),
        None => config_path()?,
    };
    if !path.is_file() {
        return Err(format!("Config does not exist: {}", path.display()));
    }
    Config::load(Some(&path))?;
    println!("Configuration valid: {}", path.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn selective_modules_keep_order_and_lazy_collection() {
        let mut config = Config::default();
        let options = Options {
            only: Some(vec!["memory".into(), "os".into(), "cpu".into()]),
            ..Options::default()
        };
        configure(&mut config, &options).unwrap();
        assert_eq!(config.modules.system, vec!["os"]);
        assert_eq!(config.modules.hardware, vec!["memory", "cpu"]);
    }
    #[test]
    fn unknown_module_rejected() {
        let mut config = Config::default();
        let options = Options {
            exclude: Some(vec!["not_a_module".into()]),
            ..Options::default()
        };
        assert!(configure(&mut config, &options).is_err());
    }
    #[test]
    fn default_config_is_valid_json5() {
        let cfg: Config = json5::from_str(DEFAULT_JSONC).unwrap();
        cfg.validate().unwrap();
    }
    #[test]
    fn themes_reject_unknown() {
        let mut cfg = Config::default();
        assert!(apply_theme(&mut cfg, "no-such-theme").is_err());
    }
}
