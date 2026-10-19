pub mod battery;
pub mod cpu;
pub mod disk;
pub mod gpu;
pub mod memory;
pub mod os;
pub mod uptime;

use std::{env, fs};

pub struct SystemInfo {
    pub username: String,
    pub hostname: String,
    pub os: String,
    pub kernel: String,
    pub arch: String,
    pub cpu: String,
    pub gpu: String,
    pub memory: String,
    pub disk: String,
    pub battery: Option<String>,
    pub uptime: String,
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
        cpu: cpu::name(),
        gpu: gpu::name(),
        memory: memory::usage(),
        disk: disk::usage(),
        battery: battery::status(),
        uptime: uptime::formatted(),
    }
}
