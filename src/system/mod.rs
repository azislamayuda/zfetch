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
// PARALLEL SYSTEM COLLECTOR
// ============================================

pub fn collect() -> SystemInfo {
    thread::scope(|scope| {
        // ------------------------------------
        // Spawn slow detection modules
        // ------------------------------------

        let desktop_task = scope.spawn(|| profile("desktop", desktop::name));

        let terminal_task = scope.spawn(|| profile("terminal", terminal::name));

        let packages_task = scope.spawn(|| profile("packages", packages::summary));

        let displays_task = scope.spawn(|| profile("displays", display::active));

        let gpu_task = scope.spawn(|| profile("gpu", gpu::name));

        let battery_task = scope.spawn(|| profile("battery", battery::status));

        // ------------------------------------
        // Run lightweight detection
        // on the current thread
        // ------------------------------------

        let username = profile("username", || {
            env::var("USER").unwrap_or_else(|_| "Unknown".into())
        });

        let hostname = profile("hostname", || {
            fs::read_to_string("/proc/sys/kernel/hostname")
                .map(|s| s.trim().to_string())
                .unwrap_or_else(|_| "Unknown".into())
        });

        let os = profile("os", os::name);
        let kernel = profile("kernel", os::kernel);

        let arch = env::consts::ARCH.to_string();

        let shell = profile("shell", shell::name);
        let session = profile("session", session::name);
        let wm = profile("wm", wm::name);

        let host = profile("host", host::name);
        let bios = profile("bios", bios::info);

        let cpu = profile("cpu", cpu::name);
        let memory = profile("memory", memory::usage);
        let swap = profile("swap", swap::usage);

        let disks = profile("disks", disk::all);

        let local_ip = profile("network", network::local_ip);

        let uptime = profile("uptime", uptime::formatted);

        // ------------------------------------
        // Collect parallel results
        // ------------------------------------

        let desktop = desktop_task
            .join()
            .expect("Desktop detection thread panicked");

        let terminal = terminal_task
            .join()
            .expect("Terminal detection thread panicked");

        let packages = packages_task
            .join()
            .expect("Packages detection thread panicked");

        let displays = displays_task
            .join()
            .expect("Display detection thread panicked");

        let gpu = gpu_task.join().expect("GPU detection thread panicked");

        let battery = battery_task
            .join()
            .expect("Battery detection thread panicked");

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
