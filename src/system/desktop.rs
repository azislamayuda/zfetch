use std::{env, process::Command};

pub fn name() -> String {
    let desktop = env::var("XDG_CURRENT_DESKTOP").unwrap_or_else(|_| "Unknown".to_string());

    if desktop.to_ascii_lowercase().contains("gnome") {
        if let Ok(output) = Command::new("gnome-shell").arg("--version").output()
            && output.status.success()
        {
            let text = String::from_utf8_lossy(&output.stdout);

            if let Some(version) = text.split_whitespace().last() {
                return format!("GNOME {version}");
            }
        }

        return "GNOME".to_string();
    }

    desktop
}
