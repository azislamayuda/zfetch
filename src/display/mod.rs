use crate::system::SystemInfo;

pub fn render(info: &SystemInfo) {
    println!();
    println!("zfetch v{}", env!("CARGO_PKG_VERSION"));
    println!("{}@{}", info.username, info.hostname);
    println!("────────────────────────────────");

    println!("{:<10} {}", "OS", info.os);
    println!("{:<10} {}", "Kernel", info.kernel);
    println!("{:<10} {}", "Arch", info.arch);
    println!("{:<10} {}", "CPU", info.cpu);
    println!("{:<10} {}", "GPU", info.gpu);
    println!("{:<10} {}", "Memory", info.memory);
    println!("{:<10} {}", "Disk", info.disk);

    if let Some(battery) = &info.battery {
        println!("{:<10} {}", "Battery", battery);
    }

    println!("{:<10} {}", "Uptime", info.uptime);
    println!();
}
