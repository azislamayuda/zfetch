use std::{fs, path::Path, process::Command};

fn vendor_name(id: &str) -> Option<&'static str> {
    match id.trim() {
        "0x8086" => Some("Intel Corporation"),
        "0x1002" => Some("Advanced Micro Devices"),
        "0x10de" => Some("NVIDIA Corporation"),
        "0x13b5" => Some("Arm"),
        "0x14e4" => Some("Broadcom"),
        _ => None,
    }
}

fn model_from_udev(device: &Path, subsystem: &str) -> Option<String> {
    let address = device.file_name()?.to_str()?;
    let database = format!("/run/udev/data/+{subsystem}:{address}");
    let content = fs::read_to_string(database).ok()?;
    let model = content
        .lines()
        .find_map(|line| line.strip_prefix("E:ID_MODEL_FROM_DATABASE="))?
        .trim();
    if model.is_empty() {
        return None;
    }
    let vendor_id = fs::read_to_string(device.join("vendor")).ok();
    let vendor = vendor_id.as_deref().and_then(vendor_name);
    match vendor {
        Some(v)
            if !model
                .to_ascii_lowercase()
                .starts_with(&v.to_ascii_lowercase()) =>
        {
            Some(format!("{v} {model}"))
        }
        _ => Some(model.to_owned()),
    }
}

fn native_pci_gpu() -> Option<String> {
    let entries = fs::read_dir("/sys/bus/pci/devices").ok()?;
    let mut devices: Vec<_> = entries.flatten().map(|entry| entry.path()).collect();
    devices.sort();
    for device in devices {
        let Ok(class) = fs::read_to_string(device.join("class")) else {
            continue;
        };
        if !matches!(class.trim(), "0x030000" | "0x030200" | "0x038000") {
            continue;
        }
        if let Some(name) = model_from_udev(&device, "pci") {
            return Some(name);
        }
    }
    None
}

fn lspci_gpu() -> Option<String> {
    let output = Command::new("lspci").arg("-nn").output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    text.lines().find_map(|line| {
        let is_gpu = line.contains("VGA compatible controller")
            || line.contains("3D controller")
            || line.contains("Display controller");
        if is_gpu {
            line.split_once(": ")
                .map(|(_, description)| description.trim().to_owned())
        } else {
            None
        }
    })
}

fn drm_driver_name(raw: &str) -> Option<&'static str> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "panfrost" | "panthor" | "mali" => Some("Arm Mali GPU"),
        "v3d" | "vc4" => Some("Broadcom VideoCore GPU"),
        "msm" => Some("Qualcomm Adreno GPU"),
        "etnaviv" => Some("Vivante GPU"),
        "lima" => Some("Arm Mali (Lima) GPU"),
        "nouveau" => Some("NVIDIA GPU"),
        "amdgpu" => Some("AMD GPU"),
        "i915" | "xe" => Some("Intel GPU"),
        _ => None,
    }
}

fn native_drm_gpu() -> Option<String> {
    let mut cards: Vec<_> = fs::read_dir("/sys/class/drm")
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name().and_then(|s| s.to_str()).is_some_and(|s| {
                s.starts_with("card") && s[4..].chars().all(|c| c.is_ascii_digit())
            })
        })
        .collect();
    cards.sort();
    for card in cards {
        // driver symlink is accessible via sysfs, including platform DRM devices.
        let Ok(driver) = fs::read_link(card.join("device/driver")) else {
            continue;
        };
        let Some(name) = driver
            .file_name()
            .and_then(|s| s.to_str())
            .and_then(drm_driver_name)
        else {
            continue;
        };
        return Some(name.to_owned());
    }
    None
}

pub fn name() -> String {
    native_pci_gpu()
        .or_else(lspci_gpu)
        .or_else(native_drm_gpu)
        .unwrap_or_else(|| "Unknown".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn vendor_mapping() {
        assert_eq!(vendor_name("0x8086"), Some("Intel Corporation"));
        assert_eq!(vendor_name("0x10de"), Some("NVIDIA Corporation"));
    }
    #[test]
    fn arm_drm_drivers() {
        assert_eq!(drm_driver_name("panfrost"), Some("Arm Mali GPU"));
        assert_eq!(drm_driver_name("v3d"), Some("Broadcom VideoCore GPU"));
        assert_eq!(drm_driver_name("msm"), Some("Qualcomm Adreno GPU"));
        assert_eq!(drm_driver_name("not-a-gpu"), None);
    }
}
