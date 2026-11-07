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
pub mod util;
pub mod wm;

// Internal helper
mod package_db;

use crate::config::Config;

use std::{
    env, fs,
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Instant,
};

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

    pub memory: Option<util::Usage>,
    pub swap: Option<util::Usage>,
    pub disks: Vec<disk::DiskInfo>,

    pub local_ip: String,
    pub battery: Option<battery::Battery>,
    pub uptime: String,
}

impl SystemInfo {
    pub fn memory_text(&self, with_percent: bool) -> String {
        self.memory
            .map_or_else(|| "Unknown".into(), |u| u.formatted(with_percent))
    }

    pub fn swap_text(&self, with_percent: bool) -> String {
        match self.swap {
            None => "Unknown".into(),
            Some(u) if u.total == 0 => "Disabled".into(),
            Some(u) => u.formatted(with_percent),
        }
    }
}

// ============================================
// PERFORMANCE PROFILER
// ============================================

static BENCHMARK: AtomicBool = AtomicBool::new(false);
static TIMINGS: Mutex<Vec<(&'static str, f64)>> = Mutex::new(Vec::new());

/// Record per-module timings for `--benchmark` instead of printing them.
pub fn start_benchmark() {
    BENCHMARK.store(true, Ordering::Relaxed);
}

pub fn benchmark_timings() -> Vec<(&'static str, f64)> {
    TIMINGS.lock().map(|t| t.clone()).unwrap_or_default()
}

fn profile<T>(name: &'static str, operation: impl FnOnce() -> T) -> T {
    let benchmark = BENCHMARK.load(Ordering::Relaxed);
    if !benchmark && env::var_os("ZFETCH_PROFILE").is_none() {
        return operation();
    }

    let start = Instant::now();
    let result = operation();
    let ms = start.elapsed().as_secs_f64() * 1000.0;

    if benchmark {
        if let Ok(mut timings) = TIMINGS.lock() {
            timings.push((name, ms));
        }
    } else {
        eprintln!("[profile] {name:<12} {ms:>8.3} ms");
    }

    result
}

// ============================================
// LAZY DETECTION HELPERS
// ============================================

fn detect_if<T: Default>(active: bool, name: &'static str, operation: impl FnOnce() -> T) -> T {
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

/// `hardware_details` adds CPU topology from the same /proc/cpuinfo read.
pub fn collect(config: &Config, hardware_details: bool) -> SystemInfo {
    let enabled = |key| config.modules.enabled(key);

    thread::scope(|scope| {
        // ------------------------------------
        // Slow modules: spawn only if enabled
        // ------------------------------------

        let desktop_task = enabled("de").then(|| scope.spawn(|| profile("desktop", desktop::name)));

        let terminal_task =
            enabled("terminal").then(|| scope.spawn(|| profile("terminal", terminal::name)));

        let packages_task =
            enabled("packages").then(|| scope.spawn(|| profile("packages", packages::summary)));

        let displays_task =
            enabled("display").then(|| scope.spawn(|| profile("displays", display::active)));

        let battery_task =
            enabled("battery").then(|| scope.spawn(|| profile("battery", battery::status)));

        // ------------------------------------
        // User identity is always required
        // for the title in the renderer
        // ------------------------------------

        let username = profile("username", || {
            env::var("USER").unwrap_or_else(|_| "Unknown".into())
        });

        let hostname = profile("hostname", || {
            fs::read_to_string("/proc/sys/kernel/hostname")
                .map_or_else(|_| "Unknown".into(), |value| value.trim().to_string())
        });

        // ------------------------------------
        // Conditional system detection
        // ------------------------------------

        let os = detect_if(enabled("os"), "os", os::name);

        let kernel = detect_if(enabled("kernel"), "kernel", os::kernel);

        let arch = if enabled("arch") {
            env::consts::ARCH.to_string()
        } else {
            String::new()
        };

        let shell = detect_if(enabled("shell"), "shell", shell::name);

        let session = detect_if(enabled("session"), "session", session::name);

        let wm = detect_if(enabled("wm"), "wm", wm::name);

        // ------------------------------------
        // Conditional hardware detection
        // ------------------------------------

        let host = detect_if(enabled("host"), "host", host::name);

        let bios = detect_if(enabled("bios"), "bios", bios::info);

        let cpu = detect_if(enabled("cpu"), "cpu", || cpu::name(hardware_details));

        let gpu = detect_if(enabled("gpu"), "gpu", gpu::name);

        // One /proc/meminfo read serves both memory and swap.
        let meminfo = (enabled("memory") || enabled("swap"))
            .then(|| fs::read_to_string("/proc/meminfo").ok())
            .flatten()
            .unwrap_or_default();

        let memory = detect_if(enabled("memory"), "memory", || memory::usage(&meminfo));

        let swap = detect_if(enabled("swap"), "swap", || swap::usage(&meminfo));

        let disks = detect_if(enabled("disk"), "disks", disk::all);

        let local_ip = detect_if(enabled("local_ip"), "network", network::local_ip);

        let uptime = detect_if(enabled("uptime"), "uptime", uptime::formatted);

        // ------------------------------------
        // Collect parallel results
        // ------------------------------------

        let desktop = join_or_default(desktop_task);
        let terminal = join_or_default(terminal_task);
        let packages = join_or_default(packages_task);
        let displays = join_or_default(displays_task);
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
    fn test_disabled_detection_is_skipped() {
        let value: String = detect_if(false, "test", || panic!("Should not execute"));

        assert_eq!(value, "");
    }
}
