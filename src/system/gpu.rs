use std::{fs, path::Path, process::Command};

fn vendor_name(id: &str) -> Option<&'static str> {
    match id.trim() {
        "0x8086" => Some("Intel Corporation"),
        "0x1002" => Some("Advanced Micro Devices"),
        "0x10de" => Some("NVIDIA Corporation"),
        _ => None,
    }
}

fn model_from_udev(device: &Path) -> Option<String> {
    let address = device.file_name()?.to_str()?;

    let database = format!("/run/udev/data/+pci:{address}");

    let content = fs::read_to_string(database).ok()?;

    let model = content
        .lines()
        .find_map(|line| line.strip_prefix("E:ID_MODEL_FROM_DATABASE="))?
        .trim();

    if model.is_empty() {
        return None;
    }

    let vendor_id = fs::read_to_string(device.join("vendor")).ok()?;

    let Some(vendor) = vendor_name(&vendor_id) else {
        return Some(model.to_string());
    };

    if model
        .to_ascii_lowercase()
        .starts_with(&vendor.to_ascii_lowercase())
    {
        Some(model.to_string())
    } else {
        Some(format!("{vendor} {model}"))
    }
}

fn native_gpu() -> Option<String> {
    let entries = fs::read_dir("/sys/bus/pci/devices").ok()?;

    let mut devices: Vec<_> = entries.flatten().map(|entry| entry.path()).collect();

    devices.sort();

    for device in devices {
        let Ok(class) = fs::read_to_string(device.join("class")) else {
            continue;
        };

        let is_gpu = matches!(class.trim(), "0x030000" | "0x030200" | "0x038000");

        if !is_gpu {
            continue;
        }

        if let Some(name) = model_from_udev(&device) {
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

    for line in text.lines() {
        let is_gpu = line.contains("VGA compatible controller")
            || line.contains("3D controller")
            || line.contains("Display controller");

        if is_gpu && let Some((_, description)) = line.split_once(": ") {
            return Some(description.trim().to_string());
        }
    }

    None
}

pub fn name() -> String {
    native_gpu()
        .or_else(lspci_gpu)
        .unwrap_or_else(|| "Unknown".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vendor_mapping() {
        assert_eq!(vendor_name("0x8086"), Some("Intel Corporation"));

        assert_eq!(vendor_name("0x10de"), Some("NVIDIA Corporation"));
    }
}
