use std::{env, fs, process::Command};

fn identify(process: &str) -> Option<&'static str> {
    let name = process.to_ascii_lowercase();

    if name.contains("ptyxis") {
        Some("Ptyxis")
    } else if name.contains("gnome-terminal") {
        Some("GNOME Terminal")
    } else if name.contains("kitty") {
        Some("Kitty")
    } else if name.contains("alacritty") {
        Some("Alacritty")
    } else if name.contains("wezterm") {
        Some("WezTerm")
    } else if name.contains("konsole") {
        Some("Konsole")
    } else if name.contains("xfce4-terminal") {
        Some("XFCE Terminal")
    } else if name == "foot" {
        Some("Foot")
    } else if name == "tilix" {
        Some("Tilix")
    } else if name == "kgx" {
        Some("GNOME Console")
    } else {
        None
    }
}

fn detect_parent() -> Option<&'static str> {
    let mut pid = std::process::id();

    for _ in 0..20 {
        let status = fs::read_to_string(format!("/proc/{pid}/status")).ok()?;

        let parent = status
            .lines()
            .find_map(|line| line.strip_prefix("PPid:")?.trim().parse::<u32>().ok())?;

        if parent <= 1 {
            break;
        }

        let process = fs::read_to_string(format!("/proc/{parent}/comm")).ok()?;

        if let Some(terminal) = identify(process.trim()) {
            return Some(terminal);
        }

        pid = parent;
    }

    None
}

pub fn name() -> String {
    let detected = detect_parent().or_else(|| {
        env::var("TERM_PROGRAM")
            .ok()
            .and_then(|value| identify(&value))
    });

    let Some(terminal) = detected else {
        return "Unknown".to_string();
    };

    if terminal == "Ptyxis"
        && let Ok(output) = Command::new("ptyxis").arg("--version").output()
        && output.status.success()
    {
        let bytes = if output.stdout.is_empty() {
            &output.stderr
        } else {
            &output.stdout
        };

        let text = String::from_utf8_lossy(bytes);

        let version = text.lines().find_map(|line| {
            line.trim()
                .strip_prefix("Ptyxis ")
                .and_then(|value| value.split_whitespace().next())
        });

        if let Some(version) = version
            && version.chars().next().is_some_and(|c| c.is_ascii_digit())
        {
            return format!("Ptyxis {version}");
        }
    }

    terminal.to_string()
}
