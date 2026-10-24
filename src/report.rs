//! Sprint 04: privacy-aware output and safe local report export.
//! No network access and no work on normal runs unless the options are requested.
use crate::system::SystemInfo;
use serde_json::Value;
use std::{
    fs::{self, OpenOptions},
    io::Write,
    os::unix::fs::OpenOptionsExt,
    path::Path,
};

const REDACTED: &str = "hidden";

/// Keep the original values only for last-mile redaction of optional JSON telemetry.
pub struct Sensitive {
    username: String,
    hostname: String,
    ip_address: String,
}

fn valid_token(value: &str) -> bool {
    value.len() >= 3 && !matches!(value, "Unknown" | "unknown" | "hidden")
}

fn personal_mount(mount: &str) -> bool {
    ["/home/", "/media/", "/run/media/", "/mnt/"]
        .iter()
        .any(|prefix| mount.starts_with(prefix))
}

/// Redact before rendering, so terminal, text export, and v0.3 JSON use the same source.
pub fn protect_info(info: &mut SystemInfo) -> Sensitive {
    let ip_address = info
        .local_ip
        .split(['/', ' ', '('])
        .next()
        .unwrap_or("")
        .to_owned();
    let old = Sensitive {
        username: info.username.clone(),
        hostname: info.hostname.clone(),
        ip_address,
    };
    info.username = REDACTED.into();
    info.hostname = REDACTED.into();
    if !info.local_ip.is_empty() {
        info.local_ip = REDACTED.into();
    }
    for disk in &mut info.disks {
        if personal_mount(&disk.mount_point) {
            disk.mount_point = "[private mount]".into();
        }
    }
    old
}

fn protected_string(value: &str, sensitive: &Sensitive) -> String {
    let mut text = value.to_owned();
    for old in [
        sensitive.username.as_str(),
        sensitive.hostname.as_str(),
        sensitive.ip_address.as_str(),
    ] {
        if valid_token(old) {
            text = text.replace(old, REDACTED);
        }
    }
    // Avoid exposing user-mounted paths in optional telemetry string values.
    if text.starts_with("/home/")
        || text.starts_with("/media/")
        || text.starts_with("/run/media/")
        || text.starts_with("/mnt/")
    {
        "[private mount]".into()
    } else {
        text
    }
}

fn sensitive_key(key: &str) -> bool {
    matches!(
        key,
        "username"
            | "hostname"
            | "local_ip"
            | "ip_address"
            | "ipv4"
            | "ipv6"
            | "mac_address"
            | "ssid"
            | "serial_number"
    )
}

/// A second pass handles privacy-sensitive keys in optional JSON metadata/telemetry.
pub fn protect_json(value: &mut Value, sensitive: &Sensitive) {
    match value {
        Value::Object(map) => {
            for (key, field) in map {
                if sensitive_key(key) {
                    *field = Value::String(REDACTED.into());
                } else {
                    protect_json(field, sensitive);
                }
            }
        }
        Value::Array(items) => {
            for item in items {
                protect_json(item, sensitive);
            }
        }
        Value::String(text) => *text = protected_string(text, sensitive),
        _ => {}
    }
}

/// Create only a new regular file. Existing files (including symlinks) are never overwritten.
/// Reports are 0600 even when the process has a permissive umask.
pub fn write_export(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
        .map_err(|error| format!("Cannot create new report {}: {error}", path.display()))?;

    let outcome = file.write_all(bytes).and_then(|()| file.sync_all());
    drop(file);
    if let Err(error) = outcome {
        let _ = fs::remove_file(path);
        return Err(format!("Cannot write report {}: {error}", path.display()));
    }
    Ok(())
}

/// Normal terminal and JSON paths print to stdout; export sends content only to file.
pub fn emit(text: &str, export: Option<&Path>) -> Result<(), String> {
    if let Some(path) = export {
        write_export(path, text.as_bytes())?;
        println!("Report exported successfully.");
    } else {
        println!("{text}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::system::disk::DiskInfo;
    use serde_json::json;
    use std::os::unix::fs::PermissionsExt;

    fn example_info() -> SystemInfo {
        SystemInfo {
            username: "sampleuser".into(),
            hostname: "privatehost".into(),
            os: "Ubuntu".into(),
            kernel: "7.0".into(),
            arch: "x86_64".into(),
            shell: "bash".into(),
            desktop: "GNOME".into(),
            session: "Wayland".into(),
            wm: "Mutter".into(),
            terminal: "Ptyxis".into(),
            packages: "100 (dpkg)".into(),
            host: "Laptop".into(),
            bios: "Firmware".into(),
            displays: vec!["1920x1080".into()],
            cpu: "Intel".into(),
            gpu: "Iris Xe".into(),
            memory: "8 GiB / 16 GiB".into(),
            swap: "0 GiB / 16 GiB".into(),
            disks: vec![
                DiskInfo {
                    mount_point: "/".into(),
                    filesystem: "ext4".into(),
                    used: 10,
                    total: 20,
                },
                DiskInfo {
                    mount_point: "/home/sampleuser/MYDATA".into(),
                    filesystem: "ext4".into(),
                    used: 10,
                    total: 20,
                },
            ],
            local_ip: "192.168.1.67/24 (wlp0s20f3)".into(),
            battery: Some("80%".into()),
            uptime: "1h".into(),
        }
    }

    #[test]
    fn privacy_masks_user_hostname_ip_and_private_mounts() {
        let mut info = example_info();
        let sensitive = protect_info(&mut info);
        assert_eq!(info.username, "hidden");
        assert_eq!(info.hostname, "hidden");
        assert_eq!(info.local_ip, "hidden");
        assert_eq!(info.disks[0].mount_point, "/");
        assert_eq!(info.disks[1].mount_point, "[private mount]");
        let mut output = json!({
            "identity":{"username":"sampleuser", "hostname":"privatehost"},
            "hardware":{"local_ip":"192.168.1.67/24 (wlp0s20f3)",
                "disk":[{"mount_point":"/home/sampleuser/data"}]},
            "metadata":{"ssid":"OfficeWiFi", "note":"privatehost / 192.168.1.67"}
        });
        protect_json(&mut output, &sensitive);
        let rendered = output.to_string();
        for private in ["sampleuser", "privatehost", "192.168.1.67", "OfficeWiFi"] {
            assert!(!rendered.contains(private));
        }
    }

    #[test]
    fn export_writes_private_file_and_rejects_overwrite() {
        let directory = std::env::temp_dir().join(format!(
            "zfetch-sprint04-{}-{}",
            std::process::id(),
            std::thread::current()
                .name()
                .unwrap_or("test")
                .replace(':', "_")
        ));
        fs::create_dir_all(&directory).unwrap();
        let file = directory.join("report.txt");
        let _ = fs::remove_file(&file);
        write_export(&file, b"example\n").unwrap();
        assert_eq!(fs::read(&file).unwrap(), b"example\n");
        assert_eq!(
            fs::metadata(&file).unwrap().permissions().mode() & 0o777,
            0o600
        );
        assert!(write_export(&file, b"overwrite").is_err());
        assert_eq!(fs::read(&file).unwrap(), b"example\n");
        fs::remove_file(&file).unwrap();
        fs::remove_dir(&directory).unwrap();
    }

    #[test]
    fn export_does_not_follow_existing_symlink() {
        use std::os::unix::fs::symlink;
        let directory = std::env::temp_dir().join(format!("zfetch-link-{}", std::process::id()));
        fs::create_dir_all(&directory).unwrap();
        let victim = directory.join("victim");
        let link = directory.join("report.json");
        let _ = fs::remove_file(&link);
        let _ = fs::remove_file(&victim);
        fs::write(&victim, b"safe").unwrap();
        symlink(&victim, &link).unwrap();
        assert!(write_export(&link, b"changed").is_err());
        assert_eq!(fs::read(&victim).unwrap(), b"safe");
        fs::remove_file(&link).unwrap();
        fs::remove_file(&victim).unwrap();
        fs::remove_dir(&directory).unwrap();
    }
}
