use std::{
    collections::{HashMap, HashSet},
    process::Command,
};

use zbus::{
    blocking::{Connection, Proxy},
    zvariant::{OwnedValue, Value},
};

// ============================================
// MUTTER D-BUS DATA TYPES
// ============================================

type Properties = HashMap<String, OwnedValue>;

type MonitorSpec = (
    String, // Connector
    String, // Vendor
    String, // Product
    String, // Serial
);

type MonitorMode = (
    String,     // Mode ID
    i32,        // Width
    i32,        // Height
    f64,        // Refresh rate
    f64,        // Preferred scale
    Vec<f64>,   // Supported scales
    Properties, // Mode properties
);

type PhysicalMonitor = (MonitorSpec, Vec<MonitorMode>, Properties);

type LogicalMonitor = (
    i32,              // X
    i32,              // Y
    f64,              // Scale
    u32,              // Transform
    bool,             // Primary
    Vec<MonitorSpec>, // Monitor specifications
    Properties,
);

type CurrentState = (u32, Vec<PhysicalMonitor>, Vec<LogicalMonitor>, Properties);

// ============================================
// PROPERTY HELPERS
// ============================================

fn property_bool(properties: &Properties, key: &str) -> bool {
    matches!(
        properties.get(key).map(|value| &**value),
        Some(&Value::Bool(true))
    )
}

fn is_builtin(connector: &str) -> bool {
    connector.starts_with("eDP") || connector.starts_with("LVDS") || connector.starts_with("DSI")
}

fn format_display(width: i32, height: i32, refresh: f64, connector: &str, builtin: bool) -> String {
    let hz = if (refresh - refresh.round()).abs() < 0.01 {
        format!("{refresh:.0}")
    } else {
        format!("{refresh:.2}")
    };

    let label = if builtin { " [Built-in]" } else { "" };

    format!("{width}x{height} @ {hz} Hz{label} ({connector})")
}

// ============================================
// NATIVE GNOME MUTTER D-BUS DETECTION
// ============================================

fn mutter_dbus() -> Option<Vec<String>> {
    let connection = Connection::session().ok()?;

    let proxy = Proxy::new(
        &connection,
        "org.gnome.Mutter.DisplayConfig",
        "/org/gnome/Mutter/DisplayConfig",
        "org.gnome.Mutter.DisplayConfig",
    )
    .ok()?;

    let state: CurrentState = proxy.call("GetCurrentState", &()).ok()?;

    let (_, physical_monitors, logical_monitors, _) = state;

    let mut displays = Vec::new();
    let mut seen = HashSet::new();

    // Logical monitors identify active displays.
    for logical in &logical_monitors {
        for spec in &logical.5 {
            let connector = &spec.0;

            if !seen.insert(connector.clone()) {
                continue;
            }

            let Some(monitor) = physical_monitors
                .iter()
                .find(|monitor| monitor.0.0 == *connector)
            else {
                continue;
            };

            // Select only the currently active mode.
            let Some(mode) = monitor
                .1
                .iter()
                .find(|mode| property_bool(&mode.6, "is-current"))
            else {
                continue;
            };

            let width = mode.1;
            let height = mode.2;
            let refresh = mode.3;

            let builtin = property_bool(&monitor.2, "is-builtin") || is_builtin(connector);

            displays.push(format_display(width, height, refresh, connector, builtin));
        }
    }

    Some(displays)
}

// ============================================
// GDCTL FALLBACK
// ============================================

fn parse_mode(line: &str) -> Option<(i32, i32, f64)> {
    let mode = line
        .trim_start_matches(|c: char| !c.is_ascii_digit())
        .split_whitespace()
        .next()?;

    let (resolution, refresh) = mode.split_once('@')?;
    let (width, height) = resolution.split_once('x')?;

    Some((
        width.parse().ok()?,
        height.parse().ok()?,
        refresh.parse().ok()?,
    ))
}

fn gdctl_fallback() -> Vec<String> {
    let Ok(output) = Command::new("gdctl").arg("show").output() else {
        return Vec::new();
    };

    if !output.status.success() {
        return Vec::new();
    }

    let text = String::from_utf8_lossy(&output.stdout);

    let mut displays = Vec::new();
    let mut connector = String::new();
    let mut expect_mode = false;

    for line in text.lines() {
        if let Some((_, name)) = line.split_once("Monitor ") {
            connector = name
                .split_whitespace()
                .next()
                .unwrap_or("Unknown")
                .to_string();

            expect_mode = false;
            continue;
        }

        if line.contains("Current mode") {
            expect_mode = true;
            continue;
        }

        if expect_mode {
            if let Some((width, height, refresh)) = parse_mode(line) {
                displays.push(format_display(
                    width,
                    height,
                    refresh,
                    &connector,
                    is_builtin(&connector),
                ));
            }

            expect_mode = false;
        }
    }

    displays
}

// ============================================
// PUBLIC DISPLAY DETECTOR
// ============================================

pub fn active() -> Vec<String> {
    // Primary: direct D-Bus communication.
    if let Some(displays) = mutter_dbus()
        && !displays.is_empty()
    {
        return displays;
    }

    // Fallback: external gdctl command.
    gdctl_fallback()
}

// ============================================
// UNIT TESTS
// ============================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_current_mode() {
        assert_eq!(
            parse_mode("│   └──1920x1080@60.000"),
            Some((1920, 1080, 60.0))
        );
    }

    #[test]
    fn format_builtin_display() {
        assert_eq!(
            format_display(1920, 1080, 60.0, "eDP-1", true),
            "1920x1080 @ 60 Hz [Built-in] (eDP-1)"
        );
    }

    #[test]
    fn format_external_display() {
        assert_eq!(
            format_display(2560, 1440, 59.94, "HDMI-1", false),
            "2560x1440 @ 59.94 Hz (HDMI-1)"
        );
    }

    #[test]
    fn builtin_connector_detection() {
        assert!(is_builtin("eDP-1"));
        assert!(is_builtin("LVDS-1"));
        assert!(!is_builtin("HDMI-1"));
        assert!(!is_builtin("DP-1"));
    }
}
