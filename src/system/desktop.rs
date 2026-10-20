use std::{env, process::Command};

use zbus::blocking::{Connection, Proxy};

// ============================================
// NATIVE GNOME SHELL D-BUS DETECTION
// ============================================

fn gnome_dbus_version() -> Option<String> {
    // Independent GNOME D-Bus connection.
    let connection = Connection::session().ok()?;

    let proxy = Proxy::new(
        &connection,
        "org.gnome.Shell",
        "/org/gnome/Shell",
        "org.gnome.Shell",
    )
    .ok()?;

    let version: String = proxy.get_property("ShellVersion").ok()?;

    let version = version.trim();

    if version.is_empty() {
        None
    } else {
        Some(version.to_string())
    }
}

// ============================================
// FALLBACK - GNOME SHELL COMMAND
// ============================================

fn gnome_command_version() -> Option<String> {
    let output = Command::new("gnome-shell").arg("--version").output().ok()?;

    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8_lossy(&output.stdout);

    text.split_whitespace().last().map(str::to_string)
}

// ============================================
// GNOME DESKTOP DETECTION
// ============================================

fn is_gnome(desktop: &str) -> bool {
    desktop
        .split(':')
        .any(|component| component.eq_ignore_ascii_case("GNOME"))
}

// ============================================
// PUBLIC DESKTOP DETECTOR
// ============================================

pub fn name() -> String {
    let desktop = env::var("XDG_CURRENT_DESKTOP").unwrap_or_else(|_| "Unknown".to_string());

    if is_gnome(&desktop) {
        let version = gnome_dbus_version().or_else(gnome_command_version);

        return match version {
            Some(version) => {
                format!("GNOME {version}")
            }
            None => "GNOME".to_string(),
        };
    }

    desktop
}

// ============================================
// UNIT TESTS
// ============================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_gnome() {
        assert!(is_gnome("GNOME"));
        assert!(is_gnome("gnome"));
        assert!(is_gnome("ubuntu:GNOME"));
        assert!(is_gnome("GNOME:ubuntu"));
    }

    #[test]
    fn rejects_other_desktops() {
        assert!(!is_gnome("KDE"));
        assert!(!is_gnome("XFCE"));
        assert!(!is_gnome("Unknown"));
    }
}
