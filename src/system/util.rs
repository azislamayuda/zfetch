//! Small shared readers for /proc, /sys and /etc files.
use std::{env, fs, path::Path};

/// Raw used/total byte counters for memory, swap and disks.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Usage {
    pub used: u64,
    pub total: u64,
}

impl Usage {
    pub fn percent(self) -> u8 {
        if self.total == 0 {
            return 0;
        }
        (self.used as f64 / self.total as f64 * 100.0)
            .round()
            .clamp(0.0, 100.0) as u8
    }

    /// "8.07 GiB / 14.86 GiB", plus " (54%)" when `with_percent`.
    pub fn formatted(self, with_percent: bool) -> String {
        const GIB: f64 = 1024.0 * 1024.0 * 1024.0;
        let (used, total) = (self.used as f64 / GIB, self.total as f64 / GIB);
        if with_percent {
            format!("{used:.2} GiB / {total:.2} GiB ({}%)", self.percent())
        } else {
            format!("{used:.2} GiB / {total:.2} GiB")
        }
    }
}

/// Value of a `/proc/meminfo` key, converted from KiB to bytes.
pub fn meminfo(content: &str, key: &str) -> Option<u64> {
    content.lines().find_map(|line| {
        let (name, value) = line.split_once(':')?;
        if name.trim() != key {
            return None;
        }
        Some(value.split_whitespace().next()?.parse::<u64>().ok()? * 1024)
    })
}

pub fn read_dmi(field: &str) -> Option<String> {
    fs::read_to_string(format!("/sys/class/dmi/id/{field}"))
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

pub fn os_release() -> Option<String> {
    fs::read_to_string("/etc/os-release")
        .or_else(|_| fs::read_to_string("/usr/lib/os-release"))
        .ok()
}

pub fn os_release_value<'a>(content: &'a str, wanted: &str) -> Option<&'a str> {
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

/// PATH lookup without spawning a process.
pub fn in_path(command: &str) -> bool {
    use std::os::unix::fs::PermissionsExt;
    env::var_os("PATH").is_some_and(|paths| {
        env::split_paths(&paths).any(|dir| {
            fs::metadata(dir.join(command))
                .is_ok_and(|meta| meta.is_file() && meta.permissions().mode() & 0o111 != 0)
        })
    })
}

pub fn readable_dir(path: &Path) -> bool {
    fs::read_dir(path).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usage_formats_and_rounds() {
        let gib = 1024 * 1024 * 1024;
        let usage = Usage {
            used: 7 * gib,
            total: 14 * gib,
        };
        assert_eq!(usage.percent(), 50);
        assert_eq!(usage.formatted(true), "7.00 GiB / 14.00 GiB (50%)");
        assert_eq!(usage.formatted(false), "7.00 GiB / 14.00 GiB");
        assert_eq!(Usage::default().percent(), 0);
    }

    #[test]
    fn meminfo_reads_kib_as_bytes() {
        let text = "MemTotal:       16000 kB\nMemAvailable:    8000 kB\n";
        assert_eq!(meminfo(text, "MemTotal"), Some(16_000 * 1024));
        assert_eq!(meminfo(text, "SwapTotal"), None);
    }

    #[test]
    fn os_release_quotes() {
        let text = "NAME=\"Ubuntu\"\nID=ubuntu\nPRETTY_NAME=\"Ubuntu 26.04 LTS\"\n";
        assert_eq!(os_release_value(text, "ID"), Some("ubuntu"));
        assert_eq!(
            os_release_value(text, "PRETTY_NAME"),
            Some("Ubuntu 26.04 LTS")
        );
        assert_eq!(os_release_value(text, "ID_LIKE"), None);
    }
}
