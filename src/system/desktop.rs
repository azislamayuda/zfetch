use std::{env, process::Command};

use zbus::blocking::{Connection, Proxy};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Desktop {
    Gnome,
    Plasma,
    Xfce,
    Cinnamon,
    Mate,
}

impl Desktop {
    fn label(self) -> &'static str {
        match self {
            Self::Gnome => "GNOME",
            Self::Plasma => "KDE Plasma",
            Self::Xfce => "Xfce",
            Self::Cinnamon => "Cinnamon",
            Self::Mate => "MATE",
        }
    }
}

// Recognize only exact desktop tokens. Avoid matching unrelated substrings.
fn classify(value: &str) -> Option<Desktop> {
    value
        .split([':', ';', ','])
        .map(str::trim)
        .find_map(|token| match token.to_ascii_lowercase().as_str() {
            "gnome" | "gnome-classic" | "ubuntu" | "ubuntu-wayland" => Some(Desktop::Gnome),
            "kde" | "plasma" | "plasmawayland" | "plasmax11" => Some(Desktop::Plasma),
            "xfce" | "xfce4" | "xubuntu" => Some(Desktop::Xfce),
            "cinnamon" | "x-cinnamon" => Some(Desktop::Cinnamon),
            "mate" | "mate-session" => Some(Desktop::Mate),
            _ => None,
        })
}

fn desktop_from_values<'a>(values: impl IntoIterator<Item = &'a str>) -> Option<Desktop> {
    values.into_iter().find_map(classify)
}

// Existing GNOME D-Bus path remains preferred for version detection.
fn gnome_dbus_version() -> Option<String> {
    let connection = Connection::session().ok()?;
    let proxy = Proxy::new(
        &connection,
        "org.gnome.Shell",
        "/org/gnome/Shell",
        "org.gnome.Shell",
    )
    .ok()?;
    let version: String = proxy.get_property("ShellVersion").ok()?;
    clean_version(&version)
}

fn clean_version(raw: &str) -> Option<String> {
    let token = raw
        .split_whitespace()
        .find(|part| part.chars().next().is_some_and(|c| c.is_ascii_digit()))?;
    let token = token.trim_end_matches(|c: char| !c.is_ascii_alphanumeric());
    if token.is_empty() {
        None
    } else {
        Some(token.to_string())
    }
}

fn parse_command_version(output: &str) -> Option<String> {
    output.lines().find_map(clean_version)
}

fn command_version(command: &str) -> Option<String> {
    let output = Command::new(command).arg("--version").output().ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    parse_command_version(&stdout).or_else(|| parse_command_version(&stderr))
}

fn desktop_version(desktop: Desktop) -> Option<String> {
    match desktop {
        Desktop::Gnome => gnome_dbus_version().or_else(|| command_version("gnome-shell")),
        Desktop::Plasma => command_version("plasmashell"),
        Desktop::Xfce => command_version("xfce4-session"),
        Desktop::Cinnamon => command_version("cinnamon"),
        Desktop::Mate => command_version("mate-session"),
    }
}

pub fn name() -> String {
    let values = [
        env::var("XDG_CURRENT_DESKTOP").ok(),
        env::var("XDG_SESSION_DESKTOP").ok(),
        env::var("DESKTOP_SESSION").ok(),
    ];
    let desktop = desktop_from_values(values.iter().filter_map(Option::as_deref));

    if let Some(desktop) = desktop {
        return match desktop_version(desktop) {
            Some(version) => format!("{} {version}", desktop.label()),
            None => desktop.label().to_string(),
        };
    }

    values
        .into_iter()
        .flatten()
        .find(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "Unknown".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_gnome() {
        assert_eq!(classify("GNOME"), Some(Desktop::Gnome));
        assert_eq!(classify("gnome"), Some(Desktop::Gnome));
        assert_eq!(classify("ubuntu:GNOME"), Some(Desktop::Gnome));
        assert_eq!(classify("GNOME:ubuntu"), Some(Desktop::Gnome));
    }

    #[test]
    fn rejects_other_desktops_as_gnome() {
        assert_ne!(classify("KDE"), Some(Desktop::Gnome));
        assert_ne!(classify("XFCE"), Some(Desktop::Gnome));
        assert_eq!(classify("Unknown"), None);
    }

    #[test]
    fn recognizes_common_desktops() {
        let cases = [
            ("KDE", Desktop::Plasma),
            ("plasma", Desktop::Plasma),
            ("XFCE", Desktop::Xfce),
            ("X-Cinnamon", Desktop::Cinnamon),
            ("MATE", Desktop::Mate),
            ("Xubuntu", Desktop::Xfce),
        ];
        for (input, expected) in cases {
            assert_eq!(classify(input), Some(expected), "{input}");
        }
    }

    #[test]
    fn exact_token_matching() {
        assert_eq!(classify("notgnome"), None);
        assert_eq!(classify("mykde"), None);
        assert_eq!(classify("ubuntu:GNOME"), Some(Desktop::Gnome));
    }

    #[test]
    fn fallbacks_preserve_priority() {
        assert_eq!(
            desktop_from_values(["Unknown", "KDE", "XFCE"]),
            Some(Desktop::Plasma)
        );
        assert_eq!(desktop_from_values(["", "MATE"]), Some(Desktop::Mate));
        assert_eq!(desktop_from_values(["", "Unknown"]), None);
    }

    #[test]
    fn version_parsing_handles_common_outputs() {
        assert_eq!(
            parse_command_version("GNOME Shell 50.1\n"),
            Some("50.1".into())
        );
        assert_eq!(
            parse_command_version("plasmashell 6.3.4\n"),
            Some("6.3.4".into())
        );
        assert_eq!(
            parse_command_version("xfce4-session 4.20.0 (Xfce 4.20)\n"),
            Some("4.20.0".into())
        );
        assert_eq!(
            parse_command_version("cinnamon 6.4.2\n"),
            Some("6.4.2".into())
        );
        assert_eq!(
            parse_command_version("mate-session 1.28.1\n"),
            Some("1.28.1".into())
        );
        assert_eq!(parse_command_version("unavailable"), None);
    }
}
