pub mod battery;
pub mod bios;
pub mod cpu;
pub mod desktop;
pub mod disk;
pub mod display;
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

// Internal helper
mod package_db;

use crate::config::Config;

use std::{env, fs, thread, time::Instant};

// ============================================
// SYSTEM INFORMATION
// ============================================

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

    pub host: String,
    pub bios: String,
    pub displays: Vec<String>,
    pub cpu: String,
    pub gpu: String,

    pub memory: String,
    pub swap: String,
    pub disks: Vec<disk::DiskInfo>,

    pub local_ip: String,
    pub battery: Option<String>,
    pub uptime: String,
}

// ============================================
// PERFORMANCE PROFILER
// ============================================

fn profile<T>(name: &str, operation: impl FnOnce() -> T) -> T {
    if env::var_os("ZFETCH_PROFILE").is_none() {
        return operation();
    }

    let start = Instant::now();
    let result = operation();

    eprintln!(
        "[profile] {:<12} {:>8.3} ms",
        name,
        start.elapsed().as_secs_f64() * 1000.0
    );

    result
}

// ============================================
// LAZY DETECTION HELPERS
// ============================================

fn enabled(modules: &[String], key: &str) -> bool {
    modules.iter().any(|name| name == key)
}

fn detect_if<T: Default>(active: bool, name: &str, operation: impl FnOnce() -> T) -> T {
    if active {
        profile(name, operation)
    } else {
        T::default()
    }
}

fn join_or_default<T: Default>(task: Option<thread::ScopedJoinHandle<'_, T>>) -> T {
    match task {
        Some(handle) => handle.join().expect("System detection thread panicked"),
        None => T::default(),
    }
}

// ============================================
// CONDITIONAL PARALLEL COLLECTOR
// ============================================

pub fn collect(config: &Config) -> SystemInfo {
    let system = &config.modules.system;
    let hardware = &config.modules.hardware;

    thread::scope(|scope| {
        // ------------------------------------
        // Slow modules: spawn only if enabled
        // ------------------------------------

        let desktop_task =
            enabled(system, "de").then(|| scope.spawn(|| profile("desktop", desktop::name)));

        let terminal_task = enabled(system, "terminal")
            .then(|| scope.spawn(|| profile("terminal", terminal::name)));

        let packages_task = enabled(system, "packages")
            .then(|| scope.spawn(|| profile("packages", packages::summary)));

        let displays_task = enabled(hardware, "display")
            .then(|| scope.spawn(|| profile("displays", display::active)));

        let gpu_task = enabled(hardware, "gpu").then(|| scope.spawn(|| profile("gpu", gpu::name)));

        let battery_task = enabled(hardware, "battery")
            .then(|| scope.spawn(|| profile("battery", battery::status)));

        // ------------------------------------
        // User identity is always required
        // for the title in the renderer
        // ------------------------------------

        let username = profile("username", || {
            env::var("USER").unwrap_or_else(|_| "Unknown".into())
        });

        let hostname = profile("hostname", || {
            fs::read_to_string("/proc/sys/kernel/hostname")
                .map(|value| value.trim().to_string())
                .unwrap_or_else(|_| "Unknown".into())
        });

        // ------------------------------------
        // Conditional system detection
        // ------------------------------------

        let os = detect_if(enabled(system, "os"), "os", os::name);

        let kernel = detect_if(enabled(system, "kernel"), "kernel", os::kernel);

        let arch = if enabled(system, "arch") {
            env::consts::ARCH.to_string()
        } else {
            String::new()
        };

        let shell = detect_if(enabled(system, "shell"), "shell", shell::name);

        let session = detect_if(enabled(system, "session"), "session", session::name);

        let wm = detect_if(enabled(system, "wm"), "wm", wm::name);

        // ------------------------------------
        // Conditional hardware detection
        // ------------------------------------

        let host = detect_if(enabled(hardware, "host"), "host", host::name);

        let bios = detect_if(enabled(hardware, "bios"), "bios", bios::info);

        let cpu = detect_if(enabled(hardware, "cpu"), "cpu", cpu::name);

        let memory = detect_if(enabled(hardware, "memory"), "memory", memory::usage);

        let swap = detect_if(enabled(hardware, "swap"), "swap", swap::usage);

        let disks = detect_if(enabled(hardware, "disk"), "disks", disk::all);

        let local_ip = detect_if(enabled(hardware, "local_ip"), "network", network::local_ip);

        let uptime = detect_if(enabled(hardware, "uptime"), "uptime", uptime::formatted);

        // ------------------------------------
        // Collect parallel results
        // ------------------------------------

        let desktop = join_or_default(desktop_task);
        let terminal = join_or_default(terminal_task);
        let packages = join_or_default(packages_task);
        let displays = join_or_default(displays_task);
        let gpu = join_or_default(gpu_task);
        let battery = join_or_default(battery_task);

        // ------------------------------------
        // Assemble SystemInfo
        // ------------------------------------

        SystemInfo {
            username,
            hostname,
            os,
            kernel,
            arch,

            shell,
            desktop,
            session,
            wm,
            terminal,
            packages,

            host,
            bios,
            displays,
            cpu,
            gpu,

            memory,
            swap,
            disks,

            local_ip,
            battery,
            uptime,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enabled_modules() {
        let modules = vec!["os".to_string(), "cpu".to_string()];

        assert!(enabled(&modules, "os"));
        assert!(!enabled(&modules, "gpu"));
    }

    #[test]
    fn test_disabled_detection_is_skipped() {
        let value: String = detect_if(false, "test", || panic!("Should not execute"));

        assert!(value.is_empty());
    }
}
