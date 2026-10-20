use super::package_db;

use std::{env, fs, process::Command};

// ============================================
// TERMINAL IDENTIFICATION
// ============================================

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

// ============================================
// NATIVE /PROC PROCESS PARSER
// ============================================

// /proc/<pid>/stat:
// pid (comm) state ppid ...
//
// Process names may contain spaces or ')'.
// Find the last ") " before the numeric fields.

fn parse_stat(content: &str) -> Option<(&str, u32)> {
    let start = content.find('(')?;
    let end = content.rfind(") ")?;

    if end <= start {
        return None;
    }

    let process = &content[start + 1..end];
    let mut fields = content[end + 2..].split_whitespace();

    // Process state: R, S, D, etc.
    let _state = fields.next()?;

    // Parent PID.
    let parent = fields.next()?.parse::<u32>().ok()?;

    Some((process, parent))
}

// ============================================
// TERMINAL PARENT DETECTION
// ============================================

fn detect_parent() -> Option<&'static str> {
    let mut pid = std::process::id();

    for _ in 0..20 {
        let stat = fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;

        let (process, parent) = parse_stat(&stat)?;

        if let Some(terminal) = identify(process) {
            return Some(terminal);
        }

        if parent <= 1 || parent == pid {
            break;
        }

        pid = parent;
    }

    None
}

// ============================================
// PTYXIS FALLBACK VERSION
// ============================================

fn ptyxis_command_version() -> Option<String> {
    let output = Command::new("ptyxis").arg("--version").output().ok()?;

    if !output.status.success() {
        return None;
    }

    let bytes = if output.stdout.is_empty() {
        &output.stderr
    } else {
        &output.stdout
    };

    let text = String::from_utf8_lossy(bytes);

    text.lines().find_map(|line| {
        line.trim()
            .strip_prefix("Ptyxis ")
            .and_then(|version| version.split_whitespace().next())
            .map(str::to_string)
    })
}

// ============================================
// PUBLIC TERMINAL DETECTOR
// ============================================

pub fn name() -> String {
    let detected = detect_parent().or_else(|| {
        env::var("TERM_PROGRAM")
            .ok()
            .and_then(|value| identify(&value))
    });

    let Some(terminal) = detected else {
        return "Unknown".to_string();
    };

    if terminal == "Ptyxis" {
        let version = package_db::installed_version("ptyxis").or_else(ptyxis_command_version);

        if let Some(version) = version {
            return format!("Ptyxis {version}");
        }
    }

    terminal.to_string()
}

// ============================================
// UNIT TESTS
// ============================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_bash_process() {
        let stat = "1234 (bash) S 5678 1000 1000";

        assert_eq!(parse_stat(stat), Some(("bash", 5678)));
    }

    #[test]
    fn parses_ptyxis_process() {
        let stat = "5678 (ptyxis-agent) S 1 1000 1000";

        assert_eq!(parse_stat(stat), Some(("ptyxis-agent", 1)));

        assert_eq!(identify("ptyxis-agent"), Some("Ptyxis"));
    }

    #[test]
    fn parses_process_with_parentheses() {
        let stat = "123 (my ) terminal) S 456 1000";

        assert_eq!(parse_stat(stat), Some(("my ) terminal", 456)));
    }

    #[test]
    fn rejects_invalid_stat() {
        assert_eq!(parse_stat("invalid"), None);
    }
}
