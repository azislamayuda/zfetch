use super::desktop::{Desktop, classify};
use std::env;

/// Compositors that announce themselves through their IPC socket variable.
const SOCKET_WMS: [(&str, &str); 4] = [
    ("HYPRLAND_INSTANCE_SIGNATURE", "Hyprland"),
    ("SWAYSOCK", "Sway"),
    ("NIRI_SOCKET", "niri"),
    ("I3SOCK", "i3"),
];

fn detect(var: impl Fn(&str) -> Option<String>) -> Option<&'static str> {
    let from_socket = SOCKET_WMS
        .iter()
        .find(|(key, _)| var(key).is_some_and(|value| !value.is_empty()))
        .map(|(_, name)| *name);
    from_socket.or_else(|| match classify(&var("XDG_CURRENT_DESKTOP")?)? {
        Desktop::Gnome => Some("Mutter"),
        Desktop::Plasma => Some("KWin"),
        Desktop::Xfce => Some("Xfwm4"),
        Desktop::Cinnamon | Desktop::Mate => None,
    })
}

pub fn name() -> String {
    let Some(manager) = detect(|key| env::var(key).ok()) else {
        return "Unknown".to_string();
    };

    let session = env::var("XDG_SESSION_TYPE")
        .unwrap_or_default()
        .to_ascii_lowercase();

    match session.as_str() {
        "wayland" => format!("{manager} (Wayland)"),
        "x11" => format!("{manager} (X11)"),
        _ => manager.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn with(pairs: &[(&str, &str)]) -> Option<&'static str> {
        detect(|key| {
            pairs
                .iter()
                .find(|(k, _)| *k == key)
                .map(|(_, v)| (*v).to_string())
        })
    }

    #[test]
    fn socket_variables_win() {
        assert_eq!(
            with(&[("HYPRLAND_INSTANCE_SIGNATURE", "abc")]),
            Some("Hyprland")
        );
        assert_eq!(
            with(&[("SWAYSOCK", "/run/user/1000/sway.sock")]),
            Some("Sway")
        );
        assert_eq!(with(&[("NIRI_SOCKET", "/run/niri.sock")]), Some("niri"));
        assert_eq!(with(&[("I3SOCK", "/run/i3.sock")]), Some("i3"));
        assert_eq!(
            with(&[("SWAYSOCK", "/s"), ("XDG_CURRENT_DESKTOP", "GNOME")]),
            Some("Sway")
        );
        assert_eq!(with(&[("SWAYSOCK", "")]), None);
    }

    #[test]
    fn desktops_use_exact_tokens() {
        assert_eq!(
            with(&[("XDG_CURRENT_DESKTOP", "ubuntu:GNOME")]),
            Some("Mutter")
        );
        assert_eq!(with(&[("XDG_CURRENT_DESKTOP", "KDE")]), Some("KWin"));
        assert_eq!(with(&[("XDG_CURRENT_DESKTOP", "XFCE")]), Some("Xfwm4"));
        assert_eq!(with(&[("XDG_CURRENT_DESKTOP", "notgnome")]), None);
        assert_eq!(with(&[]), None);
    }
}
