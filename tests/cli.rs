//! Integration tests against the built binary. Isolated XDG_CONFIG_HOME keeps the
//! user's real config and logos untouched.
use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
};

const USER: &str = "zfetchprivacyuser";

fn workspace(name: &str) -> PathBuf {
    let dir = std::env::temp_dir()
        .join(format!("zfetch-it-{name}-{}", std::process::id()))
        .join(USER);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn zfetch(config_home: &PathBuf, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_zfetch"))
        .args(args)
        .env("XDG_CONFIG_HOME", config_home)
        .env("USER", USER)
        .env("NO_COLOR", "1")
        .output()
        .unwrap()
}

fn hostname() -> String {
    fs::read_to_string("/proc/sys/kernel/hostname")
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn assert_private(text: &str) {
    assert!(!text.contains(USER), "username leaked:\n{text}");
    let host = hostname();
    if host.len() >= 3 {
        assert!(!text.contains(&host), "hostname leaked:\n{text}");
    }
}

#[test]
fn doctor_runs_read_only_and_redacts_with_privacy() {
    let home = workspace("doctor");
    let plain = zfetch(&home, &["--doctor"]);
    assert!(plain.status.success());
    let text = String::from_utf8_lossy(&plain.stdout);
    assert!(text.starts_with("zfetch doctor"));
    assert!(text.contains("not found, using defaults"));
    assert!(text.contains(USER), "path shown without --privacy");
    assert!(!home.join("zfetch").exists(), "doctor must not write files");

    let private = zfetch(&home, &["--doctor", "--privacy"]);
    assert!(private.status.success());
    assert_private(&String::from_utf8_lossy(&private.stdout));
    fs::remove_dir_all(home.parent().unwrap()).unwrap();
}

#[test]
fn privacy_applies_to_terminal_json_and_export() {
    let home = workspace("privacy");
    let terminal = zfetch(&home, &["--privacy", "--no-logo", "--no-color"]);
    assert!(terminal.status.success());
    let terminal = String::from_utf8_lossy(&terminal.stdout).into_owned();
    assert!(terminal.contains("hidden@hidden"));
    assert_private(&terminal);

    let json = zfetch(&home, &["--privacy", "--format", "json"]);
    assert!(json.status.success());
    let json = String::from_utf8_lossy(&json.stdout).into_owned();
    assert!(json.contains("\"username\": \"hidden\""));
    assert_private(&json);

    for (file, extra) in [("report.txt", None), ("report.json", Some("json"))] {
        let path = home.join(file);
        let path_text = path.to_str().unwrap();
        let mut args = vec!["--privacy", "--export", path_text];
        if let Some(format) = extra {
            args.extend(["--format", format]);
        }
        assert!(zfetch(&home, &args).status.success());
        assert_private(&fs::read_to_string(&path).unwrap());
    }
    fs::remove_dir_all(home.parent().unwrap()).unwrap();
}

#[test]
fn benchmark_prints_module_table() {
    let home = workspace("bench");
    let out = zfetch(&home, &["--benchmark"]);
    assert!(out.status.success());
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.starts_with("MODULE"));
    assert!(text.contains("memory") && text.contains("total"));
    fs::remove_dir_all(home.parent().unwrap()).unwrap();
}

#[test]
fn snapshot_then_compare_reports_no_differences() {
    use std::os::unix::fs::PermissionsExt;
    let home = workspace("snapshot");
    let file = home.join("machine.json");
    let file_arg = file.to_str().unwrap();

    let saved = zfetch(&home, &["--snapshot", file_arg]);
    assert!(saved.status.success(), "{saved:?}");
    let mode = fs::metadata(&file).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o600);

    let again = zfetch(&home, &["--snapshot", file_arg]);
    assert_eq!(
        again.status.code(),
        Some(1),
        "existing snapshot must not be overwritten"
    );

    let compared = zfetch(&home, &["--compare", file_arg]);
    assert!(compared.status.success(), "{compared:?}");
    let text = String::from_utf8_lossy(&compared.stdout);
    assert!(text.contains("\n0 differences, "), "{text}");

    let report = home.join("report.json");
    fs::write(&report, br#"{"schema_version": 1}"#).unwrap();
    let rejected = zfetch(&home, &["--compare", report.to_str().unwrap()]);
    assert_eq!(rejected.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("not a snapshot"));
    fs::remove_dir_all(home.parent().unwrap()).unwrap();
}
