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
const PRIVATE_MOUNT: &str = "[private mount]";
const PERSONAL_MOUNT_PREFIXES: [&str; 4] = ["/home/", "/media/", "/run/media/", "/mnt/"];

/// Identity values captured before redaction; `scrub` removes them from any text.
pub struct Sensitive {
    tokens: [String; 3],
}

fn valid_token(value: &str) -> bool {
    value.len() >= 3 && !matches!(value, "Unknown" | "unknown" | "hidden")
}

fn personal_mount(text: &str) -> bool {
    PERSONAL_MOUNT_PREFIXES
        .iter()
        .any(|prefix| text.starts_with(prefix))
}

fn is_mac(token: &str) -> bool {
    let parts: Vec<_> = token.split(':').collect();
    parts.len() == 6
        && parts
            .iter()
            .all(|p| p.len() == 2 && p.bytes().all(|b| b.is_ascii_hexdigit()))
}

fn is_ipv6(token: &str) -> bool {
    token.matches(':').count() >= 2 && token.parse::<std::net::Ipv6Addr>().is_ok()
}

/// Replace MAC and IPv6 address tokens; tokens are runs of [A-Za-z0-9:.].
fn scrub_addresses(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut token = String::new();
    for c in text.chars().map(Some).chain([None]) {
        if let Some(c) = c
            && (c.is_ascii_alphanumeric() || c == ':' || c == '.')
        {
            token.push(c);
            continue;
        }
        // A sentence-ending '.' is not part of the address.
        let core = token.trim_end_matches('.');
        if is_mac(core) || is_ipv6(core) {
            out.push_str(REDACTED);
            out.push_str(&token[core.len()..]);
        } else {
            out.push_str(&token);
        }
        token.clear();
        if let Some(c) = c {
            out.push(c);
        }
    }
    out
}

impl Sensitive {
    pub fn new(username: &str, hostname: &str, ip_address: &str) -> Self {
        Self {
            tokens: [username.into(), hostname.into(), ip_address.into()],
        }
    }

    /// The one redaction path shared by terminal, text export, JSON and --doctor.
    pub fn scrub(&self, value: &str) -> String {
        if personal_mount(value) {
            return PRIVATE_MOUNT.into();
        }
        let mut text = scrub_addresses(value);
        for old in &self.tokens {
            if valid_token(old) {
                text = text.replace(old.as_str(), REDACTED);
            }
        }
        text
    }
}

/// Redact before rendering, so terminal, text export, and JSON use the same source.
pub fn protect_info(info: &mut SystemInfo) -> Sensitive {
    let ip_address = info.local_ip.split(['/', ' ', '(']).next().unwrap_or("");
    let sensitive = Sensitive::new(&info.username, &info.hostname, ip_address);
    info.username = REDACTED.into();
    info.hostname = REDACTED.into();
    if !info.local_ip.is_empty() {
        info.local_ip = REDACTED.into();
    }
    for field in [
        &mut info.os,
        &mut info.kernel,
        &mut info.arch,
        &mut info.shell,
        &mut info.desktop,
        &mut info.session,
        &mut info.wm,
        &mut info.terminal,
        &mut info.packages,
        &mut info.host,
        &mut info.bios,
        &mut info.cpu,
        &mut info.gpu,
        &mut info.uptime,
    ]
    .into_iter()
    .chain(&mut info.displays)
    .chain(info.disks.iter_mut().map(|disk| &mut disk.mount_point))
    .chain(
        info.battery
            .iter_mut()
            .flat_map(|b| b.state.iter_mut().chain([&mut b.extra])),
    ) {
        *field = sensitive.scrub(field);
    }
    sensitive
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
            | "serial"
            | "serial_number"
            | "product_serial"
            | "board_serial"
            | "chassis_serial"
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
        Value::String(text) => *text = sensitive.scrub(text),
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
    use crate::system::{battery::Battery, disk::DiskInfo, util::Usage};
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
            memory: Some(Usage { used: 8, total: 16 }),
            swap: Some(Usage { used: 0, total: 16 }),
            disks: vec![
                DiskInfo {
                    mount_point: "/".into(),
                    filesystem: "ext4".into(),
                    usage: Usage {
                        used: 10,
                        total: 20,
                    },
                },
                DiskInfo {
                    mount_point: "/home/sampleuser/MYDATA".into(),
                    filesystem: "ext4".into(),
                    usage: Usage {
                        used: 10,
                        total: 20,
                    },
                },
            ],
            local_ip: "192.168.1.67/24 (wlp0s20f3)".into(),
            battery: Some(Battery {
                percent: Some(80),
                state: Some("Discharging".into()),
                extra: String::new(),
            }),
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
    fn privacy_scrubs_mac_ipv6_and_identity_in_every_format() {
        let mut info = example_info();
        info.host = "Laptop of privatehost".into();
        info.displays = vec!["1920x1080 (eDP-1) aa:bb:cc:dd:ee:ff".into()];
        info.bios = "fw fe80::1c2b:3aff:fe4d:5e6f/64".into();
        let sensitive = protect_info(&mut info);
        let config = crate::config::Config::default();

        let text = crate::display::plain_report(&info, &config);
        let mut json = crate::advanced::json_document(&info, &config);
        json["hardware"]["serial_number"] = "SN-123456".into();
        json["metadata"] = json!({"mac_address": "11:22:33:44:55:66", "ipv6": "2001:db8::1"});
        protect_json(&mut json, &sensitive);
        let json = json.to_string();

        for output in [&text, &json] {
            for private in [
                "sampleuser",
                "privatehost",
                "192.168.1.67",
                "aa:bb:cc:dd:ee:ff",
                "fe80::1c2b",
                "SN-123456",
                "11:22:33:44:55:66",
                "2001:db8::1",
                "/home/",
            ] {
                assert!(!output.contains(private), "{private} leaked: {output}");
            }
        }
        assert!(text.contains("1920x1080 (eDP-1) hidden"));
    }

    #[test]
    fn address_scrubber_keeps_ordinary_text() {
        for keep in [
            "0h 27m",
            "12:30:45",
            "std::fs",
            "1920x1080 @ 60 Hz",
            "GNOME 50.1",
        ] {
            assert_eq!(scrub_addresses(keep), keep);
        }
        assert_eq!(scrub_addresses("mac AA:BB:CC:DD:EE:FF."), "mac hidden.");
        assert_eq!(scrub_addresses("(::1)"), "(hidden)");
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
