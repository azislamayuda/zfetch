//! `--doctor` read-only diagnostics and the `--benchmark` table.
//! No network, no subprocesses, no writes.
use crate::{
    config::{Config, default_config_path},
    display::logo,
    report::Sensitive,
    system::{
        packages::rpm_database_present,
        util::{in_path, os_release, os_release_value, readable_dir},
    },
};
use std::{
    env, fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Ok,
    Warn,
    Info,
}

pub struct Check {
    pub level: Level,
    pub text: String,
}

fn check(level: Level, text: impl Into<String>) -> Check {
    Check {
        level,
        text: text.into(),
    }
}

/// Package databases zfetch can read, as (label, path that marks it present).
const PACKAGE_DATABASES: [(&str, &str); 7] = [
    ("dpkg", "/var/lib/dpkg/status"),
    ("pacman", "/var/lib/pacman/local"),
    ("apk", "/lib/apk/db/installed"),
    ("portage", "/var/db/pkg"),
    ("xbps", "/var/db/xbps"),
    ("flatpak", "/var/lib/flatpak"),
    ("snap", "/var/lib/snapd/snaps"),
];

fn config_check(explicit: Option<&Path>) -> (Check, Config) {
    let Some(path) = explicit.map(Path::to_path_buf).or_else(default_config_path) else {
        return (
            check(Level::Warn, "Config: HOME and XDG_CONFIG_HOME are unset"),
            Config::default(),
        );
    };
    if !path.exists() {
        let text = format!("Config: {} not found, using defaults", path.display());
        return (check(Level::Info, text), Config::default());
    }
    match Config::load(Some(&path)) {
        Ok(config) => (
            check(Level::Ok, format!("Config: {} is valid", path.display())),
            config,
        ),
        Err(error) => (
            check(Level::Warn, format!("Config: {error}")),
            Config::default(),
        ),
    }
}

fn logo_check(config: &Config) -> Check {
    let readable = |path: &PathBuf| fs::read_to_string(path).is_ok_and(|t| !t.trim().is_empty());
    if let Some(path) = config.logo_path() {
        return if readable(&path) {
            check(Level::Ok, format!("Logo: config source {}", path.display()))
        } else {
            let text = format!("Logo: config source {} is missing or empty", path.display());
            check(Level::Warn, text)
        };
    }
    if let Some(path) = logo::user_logo_path().filter(readable) {
        return check(Level::Ok, format!("Logo: custom file {}", path.display()));
    }
    let id = os_release()
        .as_deref()
        .and_then(|text| os_release_value(text, "ID").map(str::to_owned))
        .unwrap_or_else(|| "linux".into());
    check(
        Level::Ok,
        format!("Logo: built-in, selected from os-release ID '{id}'"),
    )
}

fn os_release_check() -> Check {
    match os_release() {
        Some(text) if os_release_value(&text, "ID").is_some() => {
            check(Level::Ok, "os-release: present and parseable")
        }
        Some(_) => check(Level::Warn, "os-release: present but has no ID"),
        None => check(Level::Warn, "os-release: missing (/etc and /usr/lib)"),
    }
}

pub fn checks(explicit_config: Option<&Path>) -> Vec<Check> {
    let (config_result, config) = config_check(explicit_config);
    let mut out = vec![config_result, logo_check(&config), os_release_check()];

    for dir in ["/proc", "/sys/class/power_supply", "/sys/class/drm"] {
        out.push(if readable_dir(Path::new(dir)) {
            check(Level::Ok, format!("{dir}: accessible"))
        } else {
            check(Level::Warn, format!("{dir}: not accessible"))
        });
    }

    let mut found: Vec<&str> = PACKAGE_DATABASES
        .iter()
        .filter(|(_, path)| Path::new(path).exists())
        .map(|(name, _)| *name)
        .collect();
    if rpm_database_present() {
        found.push("rpm");
    }
    out.push(if found.is_empty() {
        check(Level::Warn, "Packages: no supported package database found")
    } else {
        check(Level::Ok, format!("Packages: {}", found.join(", ")))
    });

    for var in ["XDG_CURRENT_DESKTOP", "XDG_SESSION_TYPE"] {
        out.push(match env::var(var) {
            Ok(value) if !value.is_empty() => check(Level::Ok, format!("{var}={value}")),
            _ => check(
                Level::Info,
                format!("{var} is not set (DE/session show Unknown)"),
            ),
        });
    }

    for (command, purpose) in [
        ("gdctl", "display fallback when EDID is unreadable"),
        ("curl", "--check-update / --update"),
    ] {
        out.push(if in_path(command) {
            check(Level::Ok, format!("{command}: found ({purpose})"))
        } else {
            check(
                Level::Info,
                format!("{command}: not found (only for {purpose})"),
            )
        });
    }
    out
}

pub fn run(privacy: bool, explicit_config: Option<&Path>) {
    let sensitive = privacy.then(|| {
        let user = env::var("USER").unwrap_or_default();
        let host = fs::read_to_string("/proc/sys/kernel/hostname").unwrap_or_default();
        Sensitive::new(&user, host.trim(), "")
    });
    let home = env::var("HOME").ok().filter(|home| home.len() > 1);
    println!("zfetch doctor (read-only)");
    for item in checks(explicit_config) {
        let tag = match item.level {
            Level::Ok => "[OK]  ",
            Level::Warn => "[WARN]",
            Level::Info => "[INFO]",
        };
        let text = match (&sensitive, &home) {
            // Home paths reveal the username even when it differs from $USER.
            (Some(s), Some(home)) => s.scrub(&item.text.replace(home.as_str(), "~")),
            (Some(s), None) => s.scrub(&item.text),
            (None, _) => item.text,
        };
        println!("{tag} {text}");
    }
}

pub fn print_benchmark(timings: &[(&str, f64)], total_ms: f64) {
    let mut rows = timings.to_vec();
    rows.sort_by(|a, b| b.1.total_cmp(&a.1));
    println!("{:<12} {:>10}", "MODULE", "TIME (ms)");
    for (name, ms) in rows {
        println!("{name:<12} {ms:>10.3}");
    }
    println!("{:<12} {total_ms:>10.3}", "total");
    println!("Threaded modules (desktop, terminal, packages, displays, battery) overlap.");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doctor_reports_every_area_and_invalid_config() {
        let dir = env::temp_dir().join(format!("zfetch-doctor-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let bad = dir.join("bad.jsonc");
        fs::write(&bad, "{ \"modules\": { \"system\": [\"nope\"] } }").unwrap();
        let result = checks(Some(&bad));
        assert_eq!(result[0].level, Level::Warn);
        let all: String = result.iter().map(|c| c.text.as_str()).collect();
        for area in [
            "Logo:",
            "os-release",
            "/proc",
            "/sys/class/drm",
            "Packages",
            "XDG_SESSION_TYPE",
            "gdctl",
            "curl",
        ] {
            assert!(all.contains(area), "missing {area}");
        }
        let missing = checks(Some(&dir.join("absent.jsonc")));
        assert_eq!(missing[0].level, Level::Info);
        fs::remove_dir_all(dir).unwrap();
    }
}
