use std::env;

pub fn name() -> String {
    env::var("XDG_SESSION_TYPE")
        .ok()
        .filter(|value| !value.is_empty())
        .map(|value| match value.to_ascii_lowercase().as_str() {
            "wayland" => "Wayland".to_string(),
            "x11" => "X11".to_string(),
            _ => value,
        })
        .unwrap_or_else(|| "Unknown".to_string())
}
