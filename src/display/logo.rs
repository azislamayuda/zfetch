use std::{
    env, fs,
    path::{Path, PathBuf},
};

// All logos are embedded at build time: the normal zfetch invocation
// performs zero network requests and needs no external logo database.
fn builtin_logo(id: &str) -> Option<&'static str> {
    match id {
        "ubuntu" => Some(include_str!("../../assets/logos/ubuntu.txt")),
        "debian" => Some(include_str!("../../assets/logos/debian.txt")),
        "linuxmint" | "mint" => Some(include_str!("../../assets/logos/linuxmint.txt")),
        "pop" | "pop_os" => Some(include_str!("../../assets/logos/pop.txt")),
        "zorin" => Some(include_str!("../../assets/logos/zorin.txt")),
        "elementary" => Some(include_str!("../../assets/logos/elementary.txt")),
        "kubuntu" => Some(include_str!("../../assets/logos/kubuntu.txt")),
        "fedora" => Some(include_str!("../../assets/logos/fedora.txt")),
        "arch" | "archlinux" => Some(include_str!("../../assets/logos/arch.txt")),
        "manjaro" => Some(include_str!("../../assets/logos/manjaro.txt")),
        "opensuse" | "opensuse-tumbleweed" | "opensuse-leap" | "suse" => {
            Some(include_str!("../../assets/logos/opensuse.txt"))
        }
        "linux" => Some(include_str!("../../assets/logos/linux.txt")),
        _ => None,
    }
}

fn os_release_value<'a>(content: &'a str, wanted: &str) -> Option<&'a str> {
    content.lines().find_map(|line| {
        let (key, value) = line.split_once('=')?;
        if key.trim() != wanted {
            return None;
        }
        let value = value.trim();
        Some(
            value
                .strip_prefix('"')
                .and_then(|v| v.strip_suffix('"'))
                .unwrap_or(value),
        )
    })
}

fn select_logo(content: &str) -> &'static str {
    if let Some(id) = os_release_value(content, "ID")
        && let Some(logo) = builtin_logo(id)
    {
        return logo;
    }

    if let Some(like) = os_release_value(content, "ID_LIKE") {
        for id in like.split_whitespace() {
            if let Some(logo) = builtin_logo(id) {
                return logo;
            }
        }
    }

    builtin_logo("linux").expect("generic Linux logo is embedded")
}

fn default_logo() -> Vec<String> {
    let os_release = fs::read_to_string("/etc/os-release")
        .or_else(|_| fs::read_to_string("/usr/lib/os-release"))
        .unwrap_or_default();
    select_logo(&os_release)
        .lines()
        .map(str::to_string)
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

    // Precedence: CLI --logo, config logo.source, ZFETCH_LOGO,
    // ~/.config/zfetch/logo.txt, /etc/os-release, generic Linux logo.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ubuntu_by_id() {
        assert_eq!(
            select_logo("NAME=Ubuntu\nID=ubuntu\nID_LIKE=debian\n"),
            builtin_logo("ubuntu").unwrap()
        );
    }

    #[test]
    fn distro_wins_over_family() {
        assert_eq!(
            select_logo("ID=linuxmint\nID_LIKE=ubuntu debian\n"),
            builtin_logo("linuxmint").unwrap()
        );
    }

    #[test]
    fn quoted_family_and_fallback() {
        assert_eq!(
            select_logo("ID=custom\nID_LIKE=\"debian linux\""),
            builtin_logo("debian").unwrap()
        );
        assert_eq!(select_logo("ID=unknown\n"), builtin_logo("linux").unwrap());
    }

    #[test]
    fn all_named_logos_present() {
        for distro in [
            "ubuntu",
            "debian",
            "linuxmint",
            "pop",
            "zorin",
            "elementary",
            "kubuntu",
            "fedora",
            "arch",
            "manjaro",
            "opensuse",
            "linux",
        ] {
            assert!(builtin_logo(distro).is_some());
        }
    }
}
