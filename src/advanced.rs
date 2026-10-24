//! v0.3.0 advanced CLI operations. Everything here is offline on normal runs.
use crate::{
    cli::{Options, Preset},
    config::{Config, NamedColor},
    system::SystemInfo,
};
use serde_json::{Map, Value, json};
use std::{collections::HashSet, env, fs, io::Write, path::PathBuf};

pub const SYSTEM_MODULES: &[&str] = &[
    "os", "kernel", "arch", "shell", "de", "session", "wm", "terminal", "packages",
];
pub const HARDWARE_MODULES: &[&str] = &[
    "host", "bios", "display", "cpu", "gpu", "memory", "swap", "disk", "local_ip", "battery",
    "uptime",
];
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
    println!("Available themes: {}", THEMES.join(", "));
    println!("Note: limited to the 8 ANSI named colors supported by zfetch.");
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
/// Selection precedes system::collect(), so unselected modules are never detected.
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

#[derive(Debug, PartialEq, Eq)]
pub struct CpuTopology {
    pub cores: usize,
    pub threads: usize,
}

fn parse_cpu_topology(text: &str) -> Option<CpuTopology> {
    let mut threads = 0;
    let mut pairs = HashSet::new();
    let mut fallback_cores = None;
    for section in text.split("\n\n") {
        let mut processor = false;
        let mut physical = "0";
        let mut core = None;
        for line in section.lines() {
            let Some((key, value)) = line.split_once(':') else {
                continue;
            };
            match key.trim() {
                "processor" => processor = true,
                "physical id" => physical = value.trim(),
                "core id" => core = Some(value.trim()),
                "cpu cores" => fallback_cores = value.trim().parse::<usize>().ok(),
                _ => {}
            }
        }
        if processor {
            threads += 1;
            if let Some(core) = core {
                pairs.insert((physical.to_owned(), core.to_owned()));
            }
        }
    }
    if threads == 0 {
        return None;
    }
    let cores = if pairs.is_empty() {
        fallback_cores.unwrap_or(threads)
    } else {
        pairs.len()
    };
    Some(CpuTopology { cores, threads })
}

pub fn cpu_topology() -> Option<CpuTopology> {
    fs::read_to_string("/proc/cpuinfo")
        .ok()
        .and_then(|s| parse_cpu_topology(&s))
}

pub fn enrich_hardware(info: &mut SystemInfo, config: &Config, options: &Options) {
    if options.hardware_details
        && config.modules.hardware.iter().any(|key| key == "cpu")
        && let Some(topology) = cpu_topology()
    {
        info.cpu.push_str(&format!(
            " | {} cores / {} threads",
            topology.cores, topology.threads
        ));
    }
}

fn selected(modules: &[String], name: &str) -> bool {
    modules.iter().any(|v| v == name)
}

fn insert(map: &mut Map<String, Value>, modules: &[String], key: &str, value: &str) {
    if selected(modules, key) {
        map.insert(key.to_string(), json!(value));
    }
}

/// JSON is a machine-readable contract: strings retain the v0.2 detector values;
/// disk usage is exported separately as numeric byte counters.
pub fn json_document(info: &SystemInfo, config: &Config) -> Value {
    let sys = &config.modules.system;
    let hw = &config.modules.hardware;
    let mut system = Map::new();
    let mut hardware = Map::new();
    insert(&mut system, sys, "os", &info.os);
    insert(&mut system, sys, "kernel", &info.kernel);
    insert(&mut system, sys, "arch", &info.arch);
    insert(&mut system, sys, "shell", &info.shell);
    insert(&mut system, sys, "de", &info.desktop);
    insert(&mut system, sys, "session", &info.session);
    insert(&mut system, sys, "wm", &info.wm);
    insert(&mut system, sys, "terminal", &info.terminal);
    insert(&mut system, sys, "packages", &info.packages);
    insert(&mut hardware, hw, "host", &info.host);
    insert(&mut hardware, hw, "bios", &info.bios);
    if selected(hw, "display") {
        hardware.insert("display".into(), json!(&info.displays));
    }
    insert(&mut hardware, hw, "cpu", &info.cpu);
    insert(&mut hardware, hw, "gpu", &info.gpu);
    insert(&mut hardware, hw, "memory", &info.memory);
    insert(&mut hardware, hw, "swap", &info.swap);
    if selected(hw, "disk") {
        let disks: Vec<Value> = info
            .disks
            .iter()
            .filter(|d| config.display.show_efi || d.mount_point != "/boot/efi")
            .map(|d| {
                json!({
                    "mount_point": d.mount_point, "filesystem": d.filesystem,
                    "used_bytes": d.used, "total_bytes": d.total
                })
            })
            .collect();
        hardware.insert("disk".into(), json!(disks));
    }
    insert(&mut hardware, hw, "local_ip", &info.local_ip);
    if selected(hw, "battery") {
        hardware.insert("battery".into(), json!(&info.battery));
    }
    insert(&mut hardware, hw, "uptime", &info.uptime);
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

fn default_config_path() -> Result<PathBuf, String> {
    if let Some(root) = env::var_os("XDG_CONFIG_HOME") {
        return Ok(PathBuf::from(root).join("zfetch/config.jsonc"));
    }
    let home = env::var_os("HOME").ok_or("HOME and XDG_CONFIG_HOME are unset")?;
    Ok(PathBuf::from(home).join(".config/zfetch/config.jsonc"))
}

pub fn init_config() -> Result<(), String> {
    let path = default_config_path()?;
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
        None => default_config_path()?,
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
    fn cpu_topology_parser() {
        let sample = "processor : 0\nphysical id : 0\ncore id : 0\n\nprocessor : 1\nphysical id : 0\ncore id : 0\n\nprocessor : 2\nphysical id : 0\ncore id : 1\n";
        assert_eq!(
            parse_cpu_topology(sample),
            Some(CpuTopology {
                cores: 2,
                threads: 3
            })
        );
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
