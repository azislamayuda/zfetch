//! Optional Linux hardware telemetry for zfetch v0.3.0 Sprint 02.
//!
//! No subprocesses or network. These paths are accessed only when the user
//! explicitly requests --hardware-details and enables the corresponding module.
//! Missing or unreliable sysfs values are omitted rather than guessed.
use crate::{cli::Options, config::Config, system::SystemInfo};
use serde_json::{Value, json};
use std::{fs, path::Path};

#[derive(Debug, Default)]
pub struct HardwareTelemetry {
    cpu: Option<CpuFrequencies>,
    gpu: Option<GpuDetails>,
    battery: Option<BatteryEstimate>,
    displays: Vec<DisplayDetails>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CpuFrequencies {
    base_mhz: Option<u32>,
    max_mhz: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GpuDetails {
    kind: Option<&'static str>,
    current_mhz: Option<u32>,
    max_mhz: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BatteryEstimate {
    minutes: u32,
    to_full: bool,
}

#[derive(Debug, Clone, PartialEq)]
struct DisplayDetails {
    connector: String,
    width_mm: u16,
    height_mm: u16,
    diagonal_inches: f64,
}

fn enabled(config: &Config, name: &str) -> bool {
    config.modules.hardware.iter().any(|item| item == name)
}

fn read_u64(path: &Path) -> Option<u64> {
    fs::read_to_string(path).ok()?.trim().parse::<u64>().ok()
}

fn read_string(path: &Path) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .map(|value| value.trim().to_owned())
}

fn khz_to_mhz(khz: u64) -> Option<u32> {
    // Refuse bogus zero or implausible frequencies.
    if !(100_000..=12_000_000).contains(&khz) {
        return None;
    }
    u32::try_from(khz / 1_000).ok()
}

fn cpu_frequencies(root: &Path) -> Option<CpuFrequencies> {
    let base_mhz = read_u64(&root.join("base_frequency")).and_then(khz_to_mhz);
    let max_mhz = read_u64(&root.join("cpuinfo_max_freq")).and_then(khz_to_mhz);
    if base_mhz.is_none() && max_mhz.is_none() {
        return None;
    }
    // Firmware may expose inconsistent base/max metadata; avoid showing nonsense.
    let base_mhz = match (base_mhz, max_mhz) {
        (Some(base), Some(max)) if base > max => None,
        (value, _) => value,
    };
    Some(CpuFrequencies { base_mhz, max_mhz })
}

fn gpu_kind(label: &str) -> Option<&'static str> {
    let normalized = label.to_ascii_lowercase();
    if normalized.contains("intel iris xe")
        || (normalized.contains("intel") && normalized.contains("iris xe graphics"))
        || normalized.contains("intel iris pro")
        || normalized.contains("intel uhd graphics")
        || normalized.contains("intel hd graphics")
        || normalized.contains("radeon 780m")
        || normalized.contains("radeon 890m")
    {
        Some("integrated")
    } else if normalized.contains("geforce")
        || normalized.contains("radeon rx ")
        || normalized.contains("nvidia quadro")
    {
        Some("discrete")
    } else {
        None
    }
}

fn drm_card_clock(root: &Path, gpu_label: &str) -> (Option<u32>, Option<u32>) {
    // Do not attach one card's clock to another card in multi-GPU computers.
    // Currently only Intel DRM frequency interfaces are supported.
    if !gpu_label.to_ascii_lowercase().contains("intel") {
        return (None, None);
    }
    let Ok(entries) = fs::read_dir(root) else {
        return (None, None);
    };
    let mut cards = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if !name.starts_with("card")
            || !name[4..].chars().all(|ch| ch.is_ascii_digit())
            || name.len() == 4
        {
            continue;
        }
        let path = entry.path();
        if read_string(&path.join("device/vendor")).as_deref() == Some("0x8086") {
            cards.push(path);
        }
    }
    if cards.len() != 1 {
        return (None, None);
    }
    let card = &cards[0];
    let current = [
        "gt/gt0/rps_cur_freq_mhz",
        "device/gt/gt0/rps_cur_freq_mhz",
        "gt_cur_freq_mhz",
        "device/gt_cur_freq_mhz",
    ]
    .iter()
    .find_map(|relative| read_u64(&card.join(relative)).and_then(valid_gpu_mhz));
    let max = [
        "gt/gt0/rps_max_freq_mhz",
        "device/gt/gt0/rps_max_freq_mhz",
        "gt_max_freq_mhz",
        "device/gt_max_freq_mhz",
    ]
    .iter()
    .find_map(|relative| read_u64(&card.join(relative)).and_then(valid_gpu_mhz));
    (current, max)
}

fn valid_gpu_mhz(value: u64) -> Option<u32> {
    if !(1..=8_000).contains(&value) {
        return None;
    }
    u32::try_from(value).ok()
}

fn estimate_minutes(
    status: &str,
    now: u64,
    full: Option<u64>,
    rate: u64,
) -> Option<BatteryEstimate> {
    if rate == 0 {
        return None;
    }
    let (remaining, to_full) = match status {
        "Discharging" => (now, false),
        "Charging" => (full?.checked_sub(now)?, true),
        _ => return None,
    };
    if remaining == 0 {
        return None;
    }
    let rounded = (u128::from(remaining) * 60 + u128::from(rate) / 2) / u128::from(rate);
    if rounded == 0 || rounded > 72 * 60 {
        return None;
    }
    Some(BatteryEstimate {
        minutes: u32::try_from(rounded).ok()?,
        to_full,
    })
}

fn battery_estimate(root: &Path) -> Option<BatteryEstimate> {
    let entries = fs::read_dir(root).ok()?;
    let mut batteries: Vec<_> = entries.flatten().map(|entry| entry.path()).collect();
    batteries.sort();
    for path in batteries {
        if read_string(&path.join("type")).as_deref() != Some("Battery") {
            continue;
        }
        let Some(status) = read_string(&path.join("status")) else {
            continue;
        };
        // uWh / uW and uAh / uA both yield hours. Never mix units.
        for (now_name, full_name, rate_name) in [
            ("energy_now", "energy_full", "power_now"),
            ("charge_now", "charge_full", "current_now"),
        ] {
            if let (Some(now), Some(rate)) = (
                read_u64(&path.join(now_name)),
                read_u64(&path.join(rate_name)),
            ) {
                let full = read_u64(&path.join(full_name));
                if let Some(estimate) = estimate_minutes(&status, now, full, rate) {
                    return Some(estimate);
                }
            }
        }
    }
    None
}

fn plausible_dimensions(width_mm: u16, height_mm: u16) -> bool {
    (50..=2_000).contains(&width_mm) && (50..=2_000).contains(&height_mm)
}

fn edid_dimensions_mm(edid: &[u8]) -> Option<(u16, u16)> {
    const HEADER: &[u8; 8] = &[0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00];
    if edid.len() < 128 || &edid[..8] != HEADER {
        return None;
    }
    // Prefer detailed timing descriptors (millimeters) to integer centimeters.
    for offset in [54, 72, 90, 108] {
        let descriptor = &edid[offset..offset + 18];
        if descriptor[0] == 0 && descriptor[1] == 0 {
            continue;
        }
        let width = u16::from(descriptor[12]) | (u16::from(descriptor[14] >> 4) << 8);
        let height = u16::from(descriptor[13]) | (u16::from(descriptor[14] & 0x0f) << 8);
        if plausible_dimensions(width, height) {
            return Some((width, height));
        }
    }
    let width = u16::from(edid[21]) * 10;
    let height = u16::from(edid[22]) * 10;
    plausible_dimensions(width, height).then_some((width, height))
}

fn detect_displays(root: &Path, known_displays: &[String]) -> Vec<DisplayDetails> {
    let Ok(entries) = fs::read_dir(root) else {
        return Vec::new();
    };
    let mut devices: Vec<_> = entries.flatten().map(|entry| entry.path()).collect();
    devices.sort();
    let mut found = Vec::new();
    for device in devices {
        let Some(dirname) = device.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let Some((card, connector)) = dirname.split_once('-') else {
            continue;
        };
        if !card.starts_with("card")
            || card.len() <= 4
            || !card[4..].chars().all(|ch| ch.is_ascii_digit())
        {
            continue;
        }
        if !known_displays
            .iter()
            .any(|line| line.contains(&format!("({connector})")))
        {
            continue;
        }
        if let Some(status) = read_string(&device.join("status"))
            && status != "connected"
        {
            continue;
        }
        let Ok(bytes) = fs::read(device.join("edid")) else {
            continue;
        };
        let Some((width_mm, height_mm)) = edid_dimensions_mm(&bytes) else {
            continue;
        };
        let diagonal_inches = f64::from(width_mm).hypot(f64::from(height_mm)) / 25.4;
        found.push(DisplayDetails {
            connector: connector.into(),
            width_mm,
            height_mm,
            diagonal_inches,
        });
    }
    // Multiple GPUs may expose identically named connectors. Avoid ambiguous matches.
    let mut duplicates = std::collections::HashSet::new();
    for item in &found {
        if found
            .iter()
            .filter(|other| other.connector == item.connector)
            .count()
            > 1
        {
            duplicates.insert(item.connector.clone());
        }
    }
    found.retain(|item| !duplicates.contains(&item.connector));
    found
}

fn append_frequency(info: &mut String, frequencies: CpuFrequencies) {
    if let Some(base) = frequencies.base_mhz {
        info.push_str(&format!(
            " | Kernel Base {:.2} GHz",
            f64::from(base) / 1_000.0
        ));
    }
    if let Some(max) = frequencies.max_mhz {
        info.push_str(&format!(" | Max {:.2} GHz", f64::from(max) / 1_000.0));
    }
}

fn format_estimate(estimate: BatteryEstimate) -> String {
    let suffix = if estimate.to_full {
        "to full"
    } else {
        "remaining"
    };
    let hours = estimate.minutes / 60;
    let minutes = estimate.minutes % 60;
    if hours == 0 {
        format!("~{minutes}m {suffix}")
    } else {
        format!("~{hours}h {minutes:02}m {suffix}")
    }
}

/// Enrich only modules selected by CLI/JSONC, and only when explicitly requested.
/// Caller should run advanced::enrich_hardware first for CPU cores/threads.
pub fn enrich(info: &mut SystemInfo, config: &Config, options: &Options) -> HardwareTelemetry {
    let mut telemetry = HardwareTelemetry::default();
    if !options.hardware_details {
        return telemetry;
    }
    if enabled(config, "cpu") {
        telemetry.cpu = cpu_frequencies(Path::new("/sys/devices/system/cpu/cpu0/cpufreq"));
        if let Some(frequencies) = telemetry.cpu {
            append_frequency(&mut info.cpu, frequencies);
        }
    }
    if enabled(config, "gpu") {
        let kind = gpu_kind(&info.gpu);
        let (current_mhz, max_mhz) = drm_card_clock(Path::new("/sys/class/drm"), &info.gpu);
        if kind.is_some() || current_mhz.is_some() || max_mhz.is_some() {
            if let Some(kind) = kind {
                info.gpu.push_str(&format!(" | {kind}"));
            }
            if let Some(current) = current_mhz {
                info.gpu.push_str(&format!(" | Current {current} MHz"));
            }
            if let Some(max) = max_mhz {
                info.gpu.push_str(&format!(" | Max {max} MHz"));
            }
            telemetry.gpu = Some(GpuDetails {
                kind,
                current_mhz,
                max_mhz,
            });
        }
    }
    if enabled(config, "battery") {
        telemetry.battery = battery_estimate(Path::new("/sys/class/power_supply"));
        if let (Some(display), Some(estimate)) = (info.battery.as_mut(), telemetry.battery) {
            display.push_str(&format!(" | {}", format_estimate(estimate)));
        }
    }
    if enabled(config, "display") {
        telemetry.displays = detect_displays(Path::new("/sys/class/drm"), &info.displays);
        for detected in &telemetry.displays {
            if let Some(line) = info
                .displays
                .iter_mut()
                .find(|line| line.contains(&format!("({})", detected.connector)))
            {
                line.push_str(&format!(" | {:.1}\"", detected.diagonal_inches));
            }
        }
    }
    telemetry
}

/// Stable optional structured additions for JSON clients; existing schema stays v1.
/// These fields are emitted only for requested modules with verified data.
pub fn augment_json(document: &mut Value, telemetry: &HardwareTelemetry) {
    let Some(hardware) = document.get_mut("hardware").and_then(Value::as_object_mut) else {
        return;
    };
    if let Some(cpu) = telemetry.cpu {
        hardware.insert(
            "cpu_details".into(),
            json!({
                "base_frequency_mhz": cpu.base_mhz,
                "max_frequency_mhz": cpu.max_mhz,
            }),
        );
    }
    if let Some(gpu) = &telemetry.gpu {
        hardware.insert(
            "gpu_details".into(),
            json!({
                "type": gpu.kind,
                "current_frequency_mhz": gpu.current_mhz,
                "max_frequency_mhz": gpu.max_mhz,
            }),
        );
    }
    if let Some(battery) = telemetry.battery {
        hardware.insert(
            "battery_details".into(),
            json!({
                "estimated_minutes": battery.minutes,
                "estimate_type": if battery.to_full { "until_full" } else { "remaining" },
            }),
        );
    }
    if !telemetry.displays.is_empty() {
        hardware.insert(
            "display_details".into(),
            json!(
                telemetry
                    .displays
                    .iter()
                    .map(|d| json!({
                        "connector": d.connector,
                        "width_mm": d.width_mm,
                        "height_mm": d.height_mm,
                        "diagonal_inches": (d.diagonal_inches * 10.0).round() / 10.0,
                    }))
                    .collect::<Vec<_>>()
            ),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frequencies_validate_units() {
        assert_eq!(khz_to_mhz(4_400_000), Some(4_400));
        assert_eq!(khz_to_mhz(0), None);
        assert_eq!(khz_to_mhz(50_000_000), None);
        assert_eq!(valid_gpu_mhz(1_300), Some(1_300));
        assert_eq!(valid_gpu_mhz(0), None);
    }

    #[test]
    fn gpu_type_is_conservative() {
        assert_eq!(gpu_kind("Intel Iris Xe Graphics"), Some("integrated"));
        assert_eq!(gpu_kind("NVIDIA GeForce RTX 4060"), Some("discrete"));
        assert_eq!(gpu_kind("Unrecognized vendor 999"), None);
    }

    #[test]
    fn sprint021_real_world_iris_xe_and_kernel_base_label() {
        // Actual lspci description for Intel Tiger Lake (PCI ID 8086:9a49).
        assert_eq!(
            gpu_kind("Intel Corporation TigerLake-LP GT2 [Iris Xe Graphics]"),
            Some("integrated")
        );
        // Avoid treating an unknown vendor as an Intel iGPU by name alone.
        assert_eq!(gpu_kind("Unknown Iris Xe Graphics"), None);

        let mut formatted = String::from("Intel CPU");
        append_frequency(
            &mut formatted,
            CpuFrequencies {
                base_mhz: Some(1500),
                max_mhz: Some(4400),
            },
        );
        assert!(formatted.contains("Kernel Base 1.50 GHz"));
        assert!(formatted.contains("Max 4.40 GHz"));
    }

    #[test]
    fn battery_discharge_and_charge_estimates() {
        assert_eq!(
            estimate_minutes("Discharging", 25_000, Some(40_000), 10_000),
            Some(BatteryEstimate {
                minutes: 150,
                to_full: false
            })
        );
        assert_eq!(
            estimate_minutes("Charging", 30_000, Some(40_000), 20_000),
            Some(BatteryEstimate {
                minutes: 30,
                to_full: true
            })
        );
        assert_eq!(estimate_minutes("Full", 40_000, Some(40_000), 10_000), None);
        assert_eq!(estimate_minutes("Discharging", 1_000, None, 0), None);
        assert_eq!(
            estimate_minutes("Charging", 50_000, Some(40_000), 10_000),
            None
        );
    }

    fn sample_edid(width_mm: u16, height_mm: u16) -> [u8; 128] {
        let mut bytes = [0u8; 128];
        bytes[..8].copy_from_slice(&[0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00]);
        bytes[54] = 1; // nonzero pixel clock means detailed timing descriptor
        bytes[66] = width_mm as u8;
        bytes[67] = height_mm as u8;
        bytes[68] = (((width_mm >> 8) as u8) << 4) | ((height_mm >> 8) as u8 & 0x0f);
        bytes
    }

    #[test]
    fn edid_detailed_timing_size() {
        let edid = sample_edid(293, 165);
        assert_eq!(edid_dimensions_mm(&edid), Some((293, 165)));
        let inches = f64::from(293_u16).hypot(165.0) / 25.4;
        assert!((inches - 13.25).abs() < 0.1);
    }

    #[test]
    fn edid_cm_fallback_and_invalid_header() {
        let mut bytes = sample_edid(0, 0);
        bytes[21] = 30;
        bytes[22] = 17;
        assert_eq!(edid_dimensions_mm(&bytes), Some((300, 170)));
        bytes[0] = 42;
        assert_eq!(edid_dimensions_mm(&bytes), None);
    }

    #[test]
    fn json_metadata_opt_in() {
        let mut doc = json!({ "hardware": { "cpu": "model" } });
        let telemetry = HardwareTelemetry {
            cpu: Some(CpuFrequencies {
                base_mhz: Some(2600),
                max_mhz: Some(4400),
            }),
            battery: Some(BatteryEstimate {
                minutes: 95,
                to_full: false,
            }),
            ..HardwareTelemetry::default()
        };
        augment_json(&mut doc, &telemetry);
        assert_eq!(doc["hardware"]["cpu_details"]["max_frequency_mhz"], 4400);
        assert_eq!(doc["hardware"]["battery_details"]["estimated_minutes"], 95);
    }
}
