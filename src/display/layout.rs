use crate::system::SystemInfo;

pub enum Line {
    Title(String),
    Subtitle(String),
    Heading(&'static str),
    Item(&'static str, String),
    Spacer,
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

pub fn build(info: &SystemInfo) -> Vec<Line> {
    let mut lines = vec![
        Line::Title(format!("{}@{}", info.username, info.hostname)),
        Line::Subtitle(format!("zfetch v{}", env!("CARGO_PKG_VERSION"))),
        Line::Spacer,
        Line::Heading("SYSTEM INFO"),
        Line::Item("OS", info.os.clone()),
        Line::Item("Kernel", info.kernel.clone()),
        Line::Item("Arch", info.arch.clone()),
        Line::Item("Shell", info.shell.clone()),
        Line::Item("DE", info.desktop.clone()),
        Line::Item("Session", info.session.clone()),
        Line::Item("WM", info.wm.clone()),
        Line::Item("Terminal", info.terminal.clone()),
        Line::Item("Packages", info.packages.clone()),
        Line::Spacer,
        Line::Heading("HARDWARE INFO"),
        Line::Item("Host", info.host.clone()),
        Line::Item("BIOS", info.bios.clone()),
    ];

    if info.displays.is_empty() {
        lines.push(Line::Item("Display", "Unknown".into()));
    } else {
        for (index, display) in info.displays.iter().enumerate() {
            let label = if index == 0 { "Display" } else { "" };
            lines.push(Line::Item(label, display.clone()));
        }
    }

    lines.extend([
        Line::Item("CPU", info.cpu.clone()),
        Line::Item("GPU", compact_gpu(&info.gpu)),
        Line::Item("Memory", memory_with_percent(&info.memory)),
        Line::Item("Swap", info.swap.clone()),
    ]);

    if info.disks.is_empty() {
        lines.push(Line::Item("Disk", "Unknown".into()));
    } else {
        for (index, disk) in info
            .disks
            .iter()
            .filter(|disk| disk.mount_point != "/boot/efi")
            .enumerate()
        {
            let label = if index == 0 { "Disk" } else { "" };
            lines.push(Line::Item(label, disk.formatted()));
        }
    }

    lines.push(Line::Item("Local IP", info.local_ip.clone()));

    if let Some(battery) = &info.battery {
        lines.push(Line::Item("Battery", battery.clone()));
    }

    lines.push(Line::Item("Uptime", info.uptime.clone()));

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

        assert_eq!(memory_with_percent("Unknown"), "Unknown");
    }
}
