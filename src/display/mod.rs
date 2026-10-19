use crate::system::SystemInfo;

pub fn render(info: &SystemInfo) {
    println!();

    println!("zfetch v{}", env!("CARGO_PKG_VERSION"));
    println!("{}@{}", info.username, info.hostname);
    println!("────────────────────────────────");

    println!("{:<10} {}", "OS", info.os);
    println!("{:<10} {}", "Kernel", info.kernel);
    println!("{:<10} {}", "Arch", info.arch);
    println!("{:<10} {}", "Shell", info.shell);
    println!("{:<10} {}", "DE", info.desktop);
    println!("{:<10} {}", "Session", info.session);
    println!("{:<10} {}", "WM", info.wm);
    println!("{:<10} {}", "Terminal", info.terminal);
    println!("{:<10} {}", "Packages", info.packages);

    println!();

    println!("{:<10} {}", "Host", info.host);
    println!("{:<10} {}", "BIOS", info.bios);
    if info.displays.is_empty() {
        println!("{:<10} Unknown", "Display");
    } else {
        for (index, display) in info.displays.iter().enumerate() {
            let label = if index == 0 { "Display" } else { "" };

            println!("{label:<10} {display}");
        }
    }
    println!("{:<10} {}", "CPU", info.cpu);
    println!("{:<10} {}", "GPU", info.gpu);
    println!("{:<10} {}", "Memory", info.memory);
    println!("{:<10} {}", "Swap", info.swap);
    if info.disks.is_empty() {
        println!("{:<10} Unknown", "Disk");
    } else {
        for (index, disk) in info.disks.iter().enumerate() {
            let label = if index == 0 { "Disk" } else { "" };

            println!("{label:<10} {}", disk.formatted());
        }
    }
    println!("{:<10} {}", "Local IP", info.local_ip);

    if let Some(battery) = &info.battery {
        println!("{:<10} {}", "Battery", battery);
    }

    println!("{:<10} {}", "Uptime", info.uptime);

    println!();
}
