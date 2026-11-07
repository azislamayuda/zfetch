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
        Some(name) => {
            // Keep telemetry suffixes, but discard irrelevant PCI-id brackets.
            let suffix = value.find(" | ").map_or("", |start| &value[start..]);
            format!("{vendor} {name}{suffix}")
        }
        None => value.to_string(),
    }
}

fn item(label: &'static str, value: String, percentage: Option<u8>) -> Line {
    Line::Item {
        label,
        value,
        percentage,
        battery: false,
    }
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

        lines.push(item(label, value, None));
    }
}

fn add_hardware(lines: &mut Vec<Line>, info: &SystemInfo, config: &Config) {
    let show = config.display.show_percentage;
    for key in &config.modules.hardware {
        match key.as_str() {
            "display" => {
                if info.displays.is_empty() {
                    lines.push(item("Display", "Unknown".into(), None));
                }
                for (index, value) in info.displays.iter().enumerate() {
                    let label = if index == 0 { "Display" } else { "" };
                    lines.push(item(label, value.clone(), None));
                }
            }

            "disk" => {
                let disks: Vec<_> = info
                    .disks
                    .iter()
                    .filter(|disk| config.display.show_efi || disk.mount_point != "/boot/efi")
                    .collect();

                if disks.is_empty() {
                    lines.push(item("Disk", "Unknown".into(), None));
                }
                for (index, disk) in disks.iter().enumerate() {
                    let label = if index == 0 { "Disk" } else { "" };
                    let percent = Some(disk.usage.percent());
                    lines.push(item(label, disk.formatted(show), percent));
                }
            }

            "memory" => {
                let percent = info.memory.map(crate::system::util::Usage::percent);
                lines.push(item("Memory", info.memory_text(show), percent));
            }

            "swap" => {
                let percent = info
                    .swap
                    .filter(|u| u.total > 0)
                    .map(crate::system::util::Usage::percent);
                lines.push(item("Swap", info.swap_text(show), percent));
            }

            "battery" => {
                if let Some(battery) = &info.battery {
                    lines.push(Line::Item {
                        label: "Battery",
                        value: battery.formatted(show),
                        percentage: battery.percent,
                        battery: true,
                    });
                }
            }

            _ => {
                let (label, value) = match key.as_str() {
                    "host" => ("Host", info.host.clone()),
                    "bios" => ("BIOS", info.bios.clone()),
                    "cpu" => ("CPU", info.cpu.clone()),
                    "gpu" => ("GPU", compact_gpu(&info.gpu)),
                    "local_ip" => ("Local IP", info.local_ip.clone()),
                    "uptime" => ("Uptime", info.uptime.clone()),
                    _ => continue,
                };

                lines.push(item(label, value, None));
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
    fn sprint03_preserve_gpu_telemetry_after_model_compaction() {
        let raw = "Intel Corporation TigerLake-LP GT2 [Iris Xe Graphics] [8086:9a49] | integrated | Current 600 MHz | Max 1300 MHz";
        assert_eq!(
            compact_gpu(raw),
            "Intel Iris Xe Graphics | integrated | Current 600 MHz | Max 1300 MHz"
        );
        assert_eq!(
            compact_gpu("Intel Corporation TigerLake-LP GT2 [Iris Xe Graphics] [8086:9a49]"),
            "Intel Iris Xe Graphics"
        );
    }

    #[test]
    fn test_compact_gpu() {
        assert_eq!(
            compact_gpu("Intel Corporation TigerLake-LP GT2 [Iris Xe Graphics] [8086:9a49]"),
            "Intel Iris Xe Graphics"
        );
    }
}
