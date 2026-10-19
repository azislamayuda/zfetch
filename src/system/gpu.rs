use std::{fs, process::Command};

pub fn name() -> String {
    // Primary detection using lspci.
    if let Ok(output) = Command::new("lspci").arg("-nn").output()
        && output.status.success()
    {
        let result = String::from_utf8_lossy(&output.stdout);

        for line in result.lines() {
            let is_gpu = line.contains("VGA compatible controller")
                || line.contains("3D controller")
                || line.contains("Display controller");

            if is_gpu && let Some((_, description)) = line.split_once(": ") {
                return description.trim().to_string();
            }
        }
    }

    // Fallback using Linux sysfs.
    if let Ok(entries) = fs::read_dir("/sys/class/drm") {
        for entry in entries.flatten() {
            let filename = entry.file_name();
            let filename = filename.to_string_lossy();

            let Some(number) = filename.strip_prefix("card") else {
                continue;
            };

            if number.is_empty() || !number.chars().all(|c| c.is_ascii_digit()) {
                continue;
            }

            let path = entry.path().join("device");

            if let Ok(vendor) = fs::read_to_string(path.join("vendor")) {
                let vendor = vendor.trim();

                let name = match vendor {
                    "0x8086" => "Intel",
                    "0x1002" => "AMD",
                    "0x10de" => "NVIDIA",
                    "0x1af4" => "Virtio",
                    _ => "Unknown",
                };

                return format!("{name} GPU ({vendor})");
            }
        }
    }

    "Unknown".to_string()
}
