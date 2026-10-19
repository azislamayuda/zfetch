use std::{
    env, fs,
    path::{Path, PathBuf},
};

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

pub fn load(explicit_path: Option<&Path>, no_logo: bool) -> Vec<String> {
    if no_logo {
        return Vec::new();
    }

    let path = explicit_path.map(Path::to_path_buf).or_else(config_path);

    if let Some(path) = path
        && let Ok(content) = fs::read_to_string(path)
    {
        let lines: Vec<String> = content.lines().map(str::to_string).collect();

        if !lines.is_empty() {
            return lines;
        }
    }

    default_logo()
}
