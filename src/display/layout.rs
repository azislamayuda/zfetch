use crate::{config::Config, system::SystemInfo};

pub enum Line {
    Title(String),
    Subtitle(String),
    Heading(&'static str),
    Item {
        label: &'static str,
        value: String,
        percentage: Option<u8>,
        battery: bool,
    },
    Spacer,
}

fn compact_gpu(value: &str) -> String {
    let vendor = if value.contains("Intel") {
        "Intel"
    } else if value.contains("NVIDIA") {
        "NVIDIA"
    } else if value.contains("AMD") || value.contains("Advanced Micro Devices") {
        "AMD"
    } else {
        return value.to_string();
    };

    let model = value
        .split('[')
        .skip(1)
        .filter_map(|part| part.split_once(']').map(|(name, _)| name.trim()))
        .find(|name| !name.contains(':') && name.contains(' '));

    match model {
        Some(name) => format!("{vendor} {name}"),
        None => value.to_string(),
    }
}

fn memory_with_percent(value: &str) -> String {
    let Some((used, total)) = value.split_once(" GiB / ") else {
        return value.to_string();
    };

    let parsed = used.parse::<f64>().ok().zip(
        total
            .strip_suffix(" GiB")
            .and_then(|s| s.parse::<f64>().ok()),
    );

    if let Some((used, total)) = parsed
        && total > 0.0
    {
        let percent = (used / total * 100.0).round();
        return format!("{value} ({percent:.0}%)");
    }

    value.to_string()
}

fn extract_percent(value: &str) -> Option<u8> {
    let before = value.split_once('%')?.0;

    let digits = before.rsplit(|c: char| !c.is_ascii_digit()).next()?;

    digits.parse::<u8>().ok()
}

fn hide_percent(value: &str) -> String {
    // Handle "(48%)" and "(80%) - ext4".
    if let Some(start) = value.find(" (") {
        let rest = &value[start + 2..];

        if let Some(end) = rest.find("%)")
            && !rest[..end].is_empty()
            && rest[..end].chars().all(|c| c.is_ascii_digit())
        {
            return format!("{}{}", &value[..start], &rest[end + 2..])
                .trim()
                .to_string();
        }
    }

    // Handle battery values such as "100% (Full)".
    if let Some((digits, rest)) = value.split_once('%')
        && !digits.is_empty()
        && digits.chars().all(|c| c.is_ascii_digit())
    {
        let rest = rest.trim();

        if rest.starts_with('(') && rest.ends_with(')') {
            return rest[1..rest.len() - 1].to_string();
        }

        if !rest.is_empty() {
            return rest.to_string();
        }
    }

    value.to_string()
}

fn push_item(
    lines: &mut Vec<Line>,
    config: &Config,
    key: &str,
    label: &'static str,
    raw_value: String,
) {
    let resource = matches!(key, "memory" | "swap" | "disk" | "battery");

    let percentage = if resource {
        extract_percent(&raw_value)
    } else {
        None
    };

    let value = if resource && !config.display.show_percentage {
        hide_percent(&raw_value)
    } else {
        raw_value
    };

    lines.push(Line::Item {
        label,
        value,
        percentage,
        battery: key == "battery",
    });
}

fn add_system(lines: &mut Vec<Line>, info: &SystemInfo, config: &Config) {
    for key in &config.modules.system {
        let (label, value) = match key.as_str() {
            "os" => ("OS", info.os.clone()),
            "kernel" => ("Kernel", info.kernel.clone()),
            "arch" => ("Arch", info.arch.clone()),
            "shell" => ("Shell", info.shell.clone()),
            "de" => ("DE", info.desktop.clone()),
            "session" => ("Session", info.session.clone()),
            "wm" => ("WM", info.wm.clone()),
            "terminal" => ("Terminal", info.terminal.clone()),
            "packages" => ("Packages", info.packages.clone()),
            _ => continue,
        };

        push_item(lines, config, key, label, value);
    }
}

fn add_hardware(lines: &mut Vec<Line>, info: &SystemInfo, config: &Config) {
    for key in &config.modules.hardware {
        match key.as_str() {
            "display" => {
                if info.displays.is_empty() {
                    push_item(lines, config, key, "Display", "Unknown".into());
                } else {
                    for (index, value) in info.displays.iter().enumerate() {
                        let label = if index == 0 { "Display" } else { "" };

                        push_item(lines, config, key, label, value.clone());
                    }
                }
            }

            "disk" => {
                let disks: Vec<_> = info
                    .disks
                    .iter()
                    .filter(|disk| config.display.show_efi || disk.mount_point != "/boot/efi")
                    .collect();

                if disks.is_empty() {
                    push_item(lines, config, key, "Disk", "Unknown".into());
                } else {
                    for (index, disk) in disks.iter().enumerate() {
                        let label = if index == 0 { "Disk" } else { "" };

                        push_item(lines, config, key, label, disk.formatted());
                    }
                }
            }

            "battery" => {
                if let Some(value) = &info.battery {
                    push_item(lines, config, key, "Battery", value.clone());
                }
            }

            _ => {
                let (label, value) = match key.as_str() {
                    "host" => ("Host", info.host.clone()),
                    "bios" => ("BIOS", info.bios.clone()),
                    "cpu" => ("CPU", info.cpu.clone()),
                    "gpu" => ("GPU", compact_gpu(&info.gpu)),
                    "memory" => ("Memory", memory_with_percent(&info.memory)),
                    "swap" => ("Swap", info.swap.clone()),
                    "local_ip" => ("Local IP", info.local_ip.clone()),
                    "uptime" => ("Uptime", info.uptime.clone()),
                    _ => continue,
                };

                push_item(lines, config, key, label, value);
            }
        }
    }
}

pub fn build(info: &SystemInfo, config: &Config) -> Vec<Line> {
    let mut lines = vec![
        Line::Title(format!("{}@{}", info.username, info.hostname)),
        Line::Subtitle(format!("zfetch v{}", env!("CARGO_PKG_VERSION"))),
    ];

    if !config.modules.system.is_empty() {
        lines.push(Line::Spacer);
        lines.push(Line::Heading("SYSTEM INFO"));
        add_system(&mut lines, info, config);
    }

    if !config.modules.hardware.is_empty() {
        lines.push(Line::Spacer);
        lines.push(Line::Heading("HARDWARE INFO"));
        add_hardware(&mut lines, info, config);
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_percentage() {
        assert_eq!(
            memory_with_percent("7.00 GiB / 14.00 GiB"),
            "7.00 GiB / 14.00 GiB (50%)"
        );
    }

    #[test]
    fn test_percentage_extraction() {
        assert_eq!(extract_percent("80%"), Some(80));
        assert_eq!(extract_percent("100% (Full)"), Some(100));
        assert_eq!(extract_percent("Unknown"), None);
    }

    #[test]
    fn test_hide_percentage() {
        assert_eq!(
            hide_percent("7.12 GiB / 14.86 GiB (48%)"),
            "7.12 GiB / 14.86 GiB"
        );

        assert_eq!(
            hide_percent("229.08 GiB / 285.11 GiB (80%) - ext4 [/]"),
            "229.08 GiB / 285.11 GiB - ext4 [/]"
        );

        assert_eq!(hide_percent("100% (Full)"), "Full");
    }

    #[test]
    fn test_compact_gpu() {
        assert_eq!(
            compact_gpu("Intel Corporation TigerLake-LP GT2 [Iris Xe Graphics] [8086:9a49]"),
            "Intel Iris Xe Graphics"
        );
    }
}
