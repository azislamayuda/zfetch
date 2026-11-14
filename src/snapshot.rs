//! v0.6.0 machine snapshots (`--snapshot`) and comparison (`--compare`).
//! A snapshot holds only values that do not change from run to run; see
//! docs/design/SNAPSHOT-DESIGN-v0.6.0.md. Never called on a normal run.
use crate::{
    config::{Config, HARDWARE_MODULES, SYSTEM_MODULES},
    report::{self, Sensitive},
    system::{self, SystemInfo, packages},
};
use serde_json::{Map, Value, json};
use std::{
    collections::BTreeMap,
    env, fs,
    io::{IsTerminal, Read},
    os::unix::fs::OpenOptionsExt,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

const SCHEMA: u64 = 1;
const MAX_BYTES: u64 = 1024 * 1024;
const GIB: f64 = 1024.0 * 1024.0 * 1024.0;
/// Changes every run (uptime) or is personal and network-dependent (local_ip).
const VOLATILE: [&str; 2] = ["uptime", "local_ip"];
/// Fixed paths this version writes; snapshot paths outside them come from a newer zfetch.
const KNOWN: [&str; 17] = [
    "identity.username",
    "identity.hostname",
    "system.os",
    "system.kernel",
    "system.arch",
    "system.shell",
    "system.de",
    "system.session",
    "system.wm",
    "system.terminal",
    "hardware.host",
    "hardware.bios",
    "hardware.cpu",
    "hardware.gpu",
    "hardware.memory.total_bytes",
    "hardware.swap.total_bytes",
    "hardware.battery.present",
];
const KNOWN_FAMILIES: [&str; 3] = ["system.packages.", "hardware.display[", "hardware.disk["];

fn modules() -> Vec<&'static str> {
    SYSTEM_MODULES
        .iter()
        .chain(HARDWARE_MODULES)
        .copied()
        .filter(|name| !VOLATILE.contains(name))
        .collect()
}

/// Detector strings use "Unknown" (or empty) for "not detected"; snapshots store null.
fn text(value: &str) -> Value {
    if value.is_empty() || value == "Unknown" {
        Value::Null
    } else {
        json!(value)
    }
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// Pure builder: identity, system and hardware are redacted with the shared privacy path.
fn build(
    mut info: SystemInfo,
    counts: &[(&str, usize)],
    privacy: bool,
) -> (Value, Option<Sensitive>) {
    let sensitive = privacy.then(|| report::protect_info(&mut info));
    let packages: Map<String, Value> = counts
        .iter()
        .map(|(name, n)| ((*name).to_string(), json!(n)))
        .collect();
    let disks: Vec<Value> = info
        .disks
        .iter()
        .map(|d| {
            json!({"mount_point": d.mount_point, "filesystem": d.filesystem,
                   "total_bytes": d.usage.total})
        })
        .collect();
    let mut doc = json!({
        "snapshot_schema_version": SCHEMA,
        "kind": "zfetch-snapshot",
        "zfetch_version": env!("CARGO_PKG_VERSION"),
        "created_unix": now_unix(),
        "privacy": privacy,
        "modules": modules(),
        "identity": {"username": info.username, "hostname": info.hostname},
        "system": {
            "os": text(&info.os), "kernel": text(&info.kernel), "arch": text(&info.arch),
            "shell": text(&info.shell), "de": text(&info.desktop),
            "session": text(&info.session), "wm": text(&info.wm),
            "terminal": text(&info.terminal), "packages": packages,
        },
        "hardware": {
            "host": text(&info.host), "bios": text(&info.bios), "cpu": text(&info.cpu),
            "gpu": text(&info.gpu), "display": info.displays,
            "memory": {"total_bytes": info.memory.map(|u| u.total)},
            "swap": {"total_bytes": info.swap.map(|u| u.total)},
            "disk": disks,
            "battery": {"present": info.battery.is_some()},
        },
    });
    if let Some(sensitive) = &sensitive {
        redact_sections(&mut doc, sensitive);
    }
    (doc, sensitive)
}

/// Metadata (kind, module names) is left alone so a hostname cannot rewrite it.
fn redact_sections(doc: &mut Value, sensitive: &Sensitive) {
    for section in ["identity", "system", "hardware"] {
        if let Some(value) = doc.get_mut(section) {
            report::protect_json(value, sensitive);
        }
    }
}

/// Collect the fixed module set, ignoring the user's config and module filters.
fn current(privacy: bool) -> (Value, Option<Sensitive>) {
    let mut config = Config::default();
    // Package counts are collected per manager below instead of as one string.
    config.modules.system.retain(|m| m != "packages");
    config
        .modules
        .hardware
        .retain(|m| !VOLATILE.contains(&m.as_str()));
    let info = system::collect(&config, false);
    build(info, &packages::counts(), privacy)
}

pub fn save(path: &Path, privacy: bool) -> Result<(), String> {
    let (doc, _) = current(privacy);
    let text = serde_json::to_string_pretty(&doc).map_err(|e| e.to_string())? + "\n";
    report::write_export(path, text.as_bytes())?;
    println!("Snapshot saved: {}", path.display());
    Ok(())
}

/// Read and validate a snapshot. Symlinks, non-regular files, large files and
/// documents that are not snapshot schema <= 1 are refused.
fn read(path: &Path) -> Result<Value, String> {
    let shown = path.display();
    let file = fs::OpenOptions::new()
        .read(true)
        // NONBLOCK: opening a FIFO must not hang before the regular-file check.
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(path)
        .map_err(|error| match error.raw_os_error() {
            Some(libc::ELOOP) => format!("{shown} is a symbolic link; refusing to read it"),
            _ => format!("cannot read {shown}: {error}"),
        })?;
    let meta = file
        .metadata()
        .map_err(|e| format!("cannot read {shown}: {e}"))?;
    if !meta.is_file() {
        return Err(format!("{shown} is not a regular file"));
    }
    let too_large = || format!("{shown} is larger than 1 MiB; not a zfetch snapshot");
    if meta.len() > MAX_BYTES {
        return Err(too_large());
    }
    let mut bytes = Vec::new();
    file.take(MAX_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| format!("cannot read {shown}: {e}"))?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err(too_large());
    }
    let doc: Value =
        serde_json::from_slice(&bytes).map_err(|e| format!("{shown} is not valid JSON: {e}"))?;
    match doc.get("snapshot_schema_version") {
        None if doc.get("schema_version").is_some() => Err(format!(
            "{shown} is a --format json report, not a snapshot; create one with --snapshot"
        )),
        Some(Value::Number(n)) => match n.as_u64() {
            Some(v) if v > SCHEMA => Err(format!(
                "{shown} was created by a newer zfetch (snapshot schema {v}); \
                 this version reads up to {SCHEMA}"
            )),
            Some(_) => Ok(doc),
            None => Err(format!("{shown} is not a zfetch snapshot")),
        },
        _ => Err(format!("{shown} is not a zfetch snapshot")),
    }
}

/// Path -> value for every compared field; matching is by path, never by position.
fn flatten(doc: &Value) -> BTreeMap<String, Value> {
    let mut out = BTreeMap::new();
    for section in ["identity", "system", "hardware"] {
        let Some(map) = doc.get(section).and_then(Value::as_object) else {
            continue;
        };
        for (key, value) in map {
            let path = format!("{section}.{key}");
            match (path.as_str(), value) {
                ("hardware.display", Value::Array(items)) => {
                    for item in items {
                        let name = item.as_str().unwrap_or_default();
                        out.insert(format!("{path}[{name}]"), Value::Bool(true));
                    }
                }
                ("hardware.disk", Value::Array(items)) => {
                    // Sorted so equal redacted mounts ("[private mount]") pair up the same way.
                    let mut items = items.clone();
                    items.sort_by_key(Value::to_string);
                    for mut item in items {
                        let mount = item
                            .get("mount_point")
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_string();
                        if let Some(object) = item.as_object_mut() {
                            object.remove("mount_point");
                        }
                        let mut key = format!("{path}[{mount}]");
                        let mut n = 1;
                        while out.contains_key(&key) {
                            n += 1;
                            key = format!("{path}[{mount}] #{n}");
                        }
                        out.insert(key, item);
                    }
                }
                (_, Value::Object(inner)) => {
                    for (k, v) in inner {
                        out.insert(format!("{path}.{k}"), v.clone());
                    }
                }
                _ => {
                    out.insert(path, value.clone());
                }
            }
        }
    }
    out
}

fn known(path: &str) -> bool {
    KNOWN.contains(&path) || KNOWN_FAMILIES.iter().any(|p| path.starts_with(p))
}

#[derive(Debug, Default)]
struct Diff {
    changed: Vec<(String, Value, Value)>,
    added: Vec<(String, Value)>,
    removed: Vec<(String, Value)>,
    skipped: Vec<String>,
    unchanged: usize,
    /// Fields from a newer zfetch that this version does not know.
    not_compared: usize,
}

fn diff(old: &BTreeMap<String, Value>, new: &BTreeMap<String, Value>) -> Diff {
    let hidden = |v: &Value| v.as_str() == Some("hidden");
    let mut result = Diff::default();
    for (path, before) in old {
        match new.get(path) {
            None if !known(path) => result.not_compared += 1,
            None => result.removed.push((path.clone(), before.clone())),
            Some(after) if hidden(before) || hidden(after) => result.skipped.push(path.clone()),
            Some(after) if after == before => result.unchanged += 1,
            Some(after) => result
                .changed
                .push((path.clone(), before.clone(), after.clone())),
        }
    }
    for (path, after) in new {
        if !old.contains_key(path) {
            result.added.push((path.clone(), after.clone()));
        }
    }
    result
}

fn gib(bytes: u64) -> String {
    format!("{:.2} GiB", bytes as f64 / GIB)
}

/// Human form of one value; the path already names display entries.
fn show(path: &str, value: &Value) -> String {
    match value {
        Value::Null => "not detected".into(),
        Value::String(s) => s.clone(),
        Value::Bool(true) if path.starts_with("hardware.display[") => String::new(),
        Value::Number(n) if path.ends_with("_bytes") => n.as_u64().map_or(n.to_string(), gib),
        Value::Object(o) if path.starts_with("hardware.disk[") => {
            let fs = o.get("filesystem").and_then(Value::as_str).unwrap_or("?");
            let total = o.get("total_bytes").and_then(Value::as_u64).unwrap_or(0);
            format!("{fs}, {}", gib(total))
        }
        other => other.to_string(),
    }
}

fn age(created: u64, now: u64) -> String {
    let secs = now.saturating_sub(created);
    match secs {
        0..3600 => "less than an hour ago".into(),
        3600..86400 => format!("{} hours ago", secs / 3600),
        _ => format!("{} days ago", secs / 86400),
    }
}

fn render_terminal(file: &str, meta: &Value, result: &Diff, color: bool) -> String {
    let paint = |code: &str, word: &str| {
        if color {
            format!("\x1b[{code}m{word:<8}\x1b[0m")
        } else {
            format!("{word:<8}")
        }
    };
    let version = meta["zfetch_version"].as_str().unwrap_or("unknown");
    let created = meta["created_unix"].as_u64().unwrap_or(0);
    let mut lines = vec![
        format!(
            "Comparing with {file} (zfetch {version}, created {})",
            age(created, now_unix())
        ),
        String::new(),
    ];
    for (path, old, new) in &result.changed {
        let mut line = format!(
            "{} {path:<34} {} -> {}",
            paint("33", "changed"),
            show(path, old),
            show(path, new)
        );
        if let (true, Some(a), Some(b)) = (
            path.starts_with("system.packages."),
            old.as_i64(),
            new.as_i64(),
        ) {
            line += &format!(" ({:+})", b - a);
        }
        lines.push(line);
    }
    for (path, new) in &result.added {
        lines.push(
            format!("{} {path:<34} {}", paint("32", "added"), show(path, new))
                .trim_end()
                .into(),
        );
    }
    for (path, old) in &result.removed {
        lines.push(
            format!("{} {path:<34} {}", paint("31", "removed"), show(path, old))
                .trim_end()
                .into(),
        );
    }
    for path in &result.skipped {
        lines.push(format!("{} {path:<34} redacted", paint("2", "skipped")));
    }
    let differences = result.changed.len() + result.added.len() + result.removed.len();
    if differences + result.skipped.len() > 0 {
        lines.push(String::new());
    }
    lines.push(format!(
        "{differences} differences, {} fields unchanged, {} skipped.",
        result.unchanged,
        result.skipped.len()
    ));
    if result.not_compared > 0 {
        lines.push(format!(
            "{} fields from a newer zfetch were not compared.",
            result.not_compared
        ));
    }
    lines.join("\n")
}

fn render_json(meta: &Value, result: &Diff) -> Value {
    json!({
        "compare_schema_version": 1,
        "snapshot": {
            "zfetch_version": meta["zfetch_version"],
            "created_unix": meta["created_unix"],
            "privacy": meta["privacy"],
        },
        "changed": result.changed.iter()
            .map(|(p, o, n)| json!({"path": p, "old": o, "new": n})).collect::<Vec<_>>(),
        "added": result.added.iter()
            .map(|(p, n)| json!({"path": p, "new": n})).collect::<Vec<_>>(),
        "removed": result.removed.iter()
            .map(|(p, o)| json!({"path": p, "old": o})).collect::<Vec<_>>(),
        "skipped": result.skipped.iter()
            .map(|p| json!({"path": p, "reason": "redacted"})).collect::<Vec<_>>(),
        "unchanged_count": result.unchanged,
        "not_compared_count": result.not_compared,
    })
}

pub fn compare(
    path: &Path,
    json_output: bool,
    no_color: bool,
    privacy: bool,
) -> Result<(), String> {
    let mut snapshot = read(path)?;
    let snapshot_private = snapshot["privacy"].as_bool().unwrap_or(false);
    // A private snapshot forces redaction of the current side, so real values never
    // print next to "hidden"; --privacy also redacts the snapshot's old values.
    let (now, sensitive) = current(privacy || snapshot_private);
    if let (true, Some(sensitive)) = (privacy, &sensitive) {
        redact_sections(&mut snapshot, sensitive);
    }
    let result = diff(&flatten(&snapshot), &flatten(&now));
    if json_output {
        let text = serde_json::to_string_pretty(&render_json(&snapshot, &result))
            .map_err(|e| e.to_string())?;
        println!("{text}");
    } else {
        let color =
            !no_color && std::io::stdout().is_terminal() && env::var_os("NO_COLOR").is_none();
        // Only the file name: the directory can contain the username.
        let file = path.file_name().map_or_else(
            || path.display().to_string(),
            |n| n.to_string_lossy().into(),
        );
        println!("{}", render_terminal(&file, &snapshot, &result, color));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::system::{battery::Battery, disk::DiskInfo, util::Usage};

    fn info() -> SystemInfo {
        let disk = |mount: &str, total| DiskInfo {
            mount_point: mount.into(),
            filesystem: "ext4".into(),
            usage: Usage { used: 1, total },
        };
        SystemInfo {
            username: "sampleuser".into(),
            hostname: "privatehost".into(),
            os: "Ubuntu 26.04".into(),
            kernel: "7.0.0-43-generic".into(),
            arch: "x86_64".into(),
            shell: "bash".into(),
            desktop: "GNOME".into(),
            session: "Wayland".into(),
            wm: "Unknown".into(),
            terminal: "Ptyxis".into(),
            packages: String::new(),
            host: "Laptop of privatehost".into(),
            bios: "Firmware".into(),
            displays: vec![
                "1920x1080 @ 60 Hz (eDP-1)".into(),
                "2560x1440 (HDMI-A-1)".into(),
            ],
            cpu: "Intel".into(),
            gpu: "Iris Xe".into(),
            memory: Some(Usage { used: 8, total: 16 }),
            swap: Some(Usage { used: 0, total: 0 }),
            disks: vec![disk("/", 100), disk("/home/sampleuser/data", 200)],
            local_ip: "192.168.1.67/24 (wlp0s20f3)".into(),
            battery: Some(Battery {
                percent: Some(80),
                state: Some("Discharging".into()),
                extra: String::new(),
            }),
            uptime: "1h 2m".into(),
        }
    }

    fn doc(privacy: bool) -> Value {
        build(info(), &[("dpkg", 2134), ("flatpak", 12)], privacy).0
    }

    fn temp(name: &str) -> std::path::PathBuf {
        let dir = env::temp_dir().join(format!(
            "zfetch-test-snapshot-{name}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn snapshot_has_own_schema_and_no_volatile_values() {
        let d = doc(false);
        assert_eq!(d["snapshot_schema_version"], 1);
        assert!(d.get("schema_version").is_none());
        assert_eq!(d["system"]["wm"], Value::Null, "Unknown becomes null");
        assert_eq!(d["system"]["packages"]["dpkg"], 2134);
        assert_eq!(d["hardware"]["memory"], json!({"total_bytes": 16}));
        let text = d.to_string();
        for volatile in [
            "uptime",
            "local_ip",
            "192.168",
            "used",
            "Discharging",
            "\"80\"",
        ] {
            assert!(!text.contains(volatile), "{volatile} stored: {text}");
        }
    }

    #[test]
    fn privacy_snapshot_redacts_identity_and_mounts() {
        let d = doc(true);
        assert_eq!(d["privacy"], true);
        assert_eq!(d["kind"], "zfetch-snapshot");
        let text = d.to_string();
        for private in ["sampleuser", "privatehost", "/home/"] {
            assert!(!text.contains(private), "{private} leaked: {text}");
        }
    }

    #[test]
    fn identical_documents_have_no_differences() {
        let mut reordered = doc(false);
        reordered["hardware"]["display"]
            .as_array_mut()
            .unwrap()
            .reverse();
        reordered["hardware"]["disk"]
            .as_array_mut()
            .unwrap()
            .reverse();
        let result = diff(&flatten(&doc(false)), &flatten(&reordered));
        assert_eq!(
            result.changed.len() + result.added.len() + result.removed.len(),
            0
        );
        assert_eq!(result.unchanged, 23);
    }

    #[test]
    fn compare_reports_changed_added_removed_and_unknown() {
        let old = doc(false);
        let mut new = doc(false);
        new["system"]["kernel"] = json!("7.0.0-45-generic");
        new["system"]["packages"] = json!({"dpkg": 2140});
        new["hardware"]["disk"][0]["total_bytes"] = json!(150);
        new["hardware"]["display"] = json!(["1920x1080 @ 60 Hz (eDP-1)"]);
        new["hardware"]["gpu"] = Value::Null;
        let mut old_newer = old.clone();
        old_newer["hardware"]["future_field"] = json!("x");
        let result = diff(&flatten(&old_newer), &flatten(&new));
        let changed: Vec<&str> = result.changed.iter().map(|c| c.0.as_str()).collect();
        assert_eq!(
            changed,
            [
                "hardware.disk[/]",
                "hardware.gpu",
                "system.kernel",
                "system.packages.dpkg"
            ]
        );
        let removed: Vec<&str> = result.removed.iter().map(|c| c.0.as_str()).collect();
        assert_eq!(
            removed,
            [
                "hardware.display[2560x1440 (HDMI-A-1)]",
                "system.packages.flatpak"
            ]
        );
        assert!(result.added.is_empty());
        assert_eq!(result.not_compared, 1);

        let meta = json!({"zfetch_version": "0.6.0", "created_unix": now_unix()});
        let text = render_terminal("s.json", &meta, &result, false);
        assert!(text.contains("changed  system.kernel"));
        assert!(text.contains("7.0.0-43-generic -> 7.0.0-45-generic"));
        assert!(text.contains("2134 -> 2140 (+6)"));
        assert!(text.contains("Iris Xe -> not detected"));
        assert!(text.contains("removed  system.packages.flatpak"));
        assert!(text.contains("6 differences, 17 fields unchanged, 0 skipped."));
        assert!(text.contains("1 fields from a newer zfetch were not compared."));
        assert!(!text.contains('\x1b'));

        let json = render_json(&meta, &result);
        assert_eq!(json["compare_schema_version"], 1);
        assert_eq!(json["changed"].as_array().unwrap().len(), 4);
        assert_eq!(json["not_compared_count"], 1);
    }

    #[test]
    fn redacted_fields_are_skipped_not_changed() {
        let result = diff(&flatten(&doc(true)), &flatten(&doc(true)));
        assert_eq!(result.skipped, ["identity.hostname", "identity.username"]);
        assert!(result.changed.is_empty());
    }

    #[test]
    fn reader_refuses_bad_files() {
        let dir = temp("read");
        let write = |name: &str, body: &[u8]| {
            let path = dir.join(name);
            fs::write(&path, body).unwrap();
            path
        };
        let cases = [
            (write("bad.json", b"{not json"), "is not valid JSON"),
            (
                write("report.json", br#"{"schema_version":1}"#),
                "--format json report",
            ),
            (
                write("newer.json", br#"{"snapshot_schema_version":2}"#),
                "newer zfetch (snapshot schema 2)",
            ),
            (
                write("other.json", br#"{"a":1}"#),
                "is not a zfetch snapshot",
            ),
            (
                write("string.json", br#"{"snapshot_schema_version":"1"}"#),
                "is not a zfetch snapshot",
            ),
            (
                write("big.json", &vec![b' '; MAX_BYTES as usize + 1]),
                "larger than 1 MiB",
            ),
            (dir.join("missing.json"), "cannot read"),
            (dir.clone(), "not a regular file"),
        ];
        for (path, expected) in cases {
            let error = read(&path).unwrap_err();
            assert!(error.contains(expected), "{expected}: {error}");
        }
        let good = write("good.json", br#"{"snapshot_schema_version":1}"#);
        let link = dir.join("link.json");
        std::os::unix::fs::symlink(&good, &link).unwrap();
        assert!(read(&link).unwrap_err().contains("symbolic link"));
        assert!(read(&good).is_ok());
        let fifo = dir.join("fifo");
        let c = std::ffi::CString::new(fifo.as_os_str().as_encoded_bytes()).unwrap();
        assert_eq!(unsafe { libc::mkfifo(c.as_ptr(), 0o600) }, 0);
        assert!(read(&fifo).unwrap_err().contains("not a regular file"));
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn age_is_readable() {
        assert_eq!(age(1000, 1000), "less than an hour ago");
        assert_eq!(age(0, 7200), "2 hours ago");
        assert_eq!(age(0, 3 * 86400), "3 days ago");
    }
}
