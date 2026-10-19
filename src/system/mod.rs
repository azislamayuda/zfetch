pub mod battery;
pub mod bios;
pub mod cpu;
pub mod desktop;
pub mod disk;
pub mod gpu;
pub mod host;
pub mod memory;
pub mod network;
pub mod os;
pub mod packages;
pub mod session;
pub mod shell;
pub mod swap;
pub mod terminal;
pub mod uptime;
pub mod wm;

use std::{env, fs};

pub struct SystemInfo {
    pub username: String,
    pub hostname: String,
    pub os: String,
    pub kernel: String,
    pub arch: String,
    pub shell: String,
    pub desktop: String,
    pub session: String,
    pub wm: String,
    pub terminal: String,
    pub packages: String,
    pub cpu: String,
    pub gpu: String,
    pub memory: String,
    pub disk: String,
    pub battery: Option<String>,
    pub uptime: String,
    pub bios: String,
    pub host: String,
    pub local_ip: String,
    pub swap: String,
}

pub fn collect() -> SystemInfo {
    let hostname = fs::read_to_string("/proc/sys/kernel/hostname")
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "Unknown".to_string());

    SystemInfo {
        username: env::var("USER").unwrap_or_else(|_| "Unknown".to_string()),
        hostname,
        os: os::name(),
        kernel: os::kernel(),
        arch: env::consts::ARCH.to_string(),
        shell: shell::name(),
        desktop: desktop::name(),
        session: session::name(),
        wm: wm::name(),
        terminal: terminal::name(),
        packages: packages::summary(),
        cpu: cpu::name(),
        gpu: gpu::name(),
        memory: memory::usage(),
        disk: disk::usage(),
        battery: battery::status(),
        uptime: uptime::formatted(),
        bios: bios::info(),
        host: host::name(),
        local_ip: network::local_ip(),
        swap: swap::usage(),
    }
}
