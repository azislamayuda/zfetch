use std::{env, fs, path::PathBuf};

fn default_logo() -> Vec<String> {
    [
        "  _______   ",
        " |__  __/   ",
        "   / /      ",
        "  / /___    ",
        " /_____/    ",
    ]
    .iter()
    .map(|line| line.to_string())
    .collect()
}

fn config_path() -> Option<PathBuf> {
    if let Some(path) = env::var_os("ZFETCH_LOGO") {
        return Some(PathBuf::from(path));
    }

    if let Some(xdg) = env::var_os("XDG_CONFIG_HOME") {
        return Some(PathBuf::from(xdg).join("zfetch/logo.txt"));
    }

    env::var_os("HOME").map(|home| PathBuf::from(home).join(".config/zfetch/logo.txt"))
}

pub fn load() -> Vec<String> {
    if let Some(path) = config_path()
        && let Ok(content) = fs::read_to_string(path)
    {
        let lines: Vec<String> = content.lines().map(|line| line.to_string()).collect();

        if !lines.is_empty() {
            return lines;
        }
    }

    default_logo()
}
