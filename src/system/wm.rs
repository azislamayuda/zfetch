use std::env;

pub fn name() -> String {
    let desktop = env::var("XDG_CURRENT_DESKTOP")
        .unwrap_or_default()
        .to_ascii_lowercase();

    let manager = if desktop.contains("gnome") {
        "Mutter"
    } else if desktop.contains("kde") {
        "KWin"
    } else if desktop.contains("xfce") {
        "Xfwm4"
    } else {
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
