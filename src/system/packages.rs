use std::{env, fs, path::Path, process::Command};

fn count_dpkg() -> Option<usize> {
    let content = fs::read_to_string("/var/lib/dpkg/status").ok()?;

    Some(
        content
            .lines()
            .filter(|line| *line == "Status: install ok installed")
            .count(),
    )
}

fn count_flatpak() -> Option<usize> {
    let output = Command::new("flatpak")
        .args(["list", "--columns=application"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8_lossy(&output.stdout);

    Some(text.lines().filter(|s| !s.trim().is_empty()).count())
}

fn count_snap() -> Option<usize> {
    let output = Command::new("snap").arg("list").output().ok()?;

    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8_lossy(&output.stdout);

    Some(
        text.lines()
            .skip(1)
            .filter(|s| !s.trim().is_empty())
            .count(),
    )
}

fn count_in_directory(path: &Path) -> Option<usize> {
    let entries = fs::read_dir(path).ok()?;

    let count = entries
        .flatten()
        .filter(|entry| {
            let path = entry.path();

            path.is_file()
                && path
                    .extension()
                    .and_then(|s| s.to_str())
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("appimage"))
        })
        .count();

    Some(count)
}

fn count_appimage() -> Option<usize> {
    let home = env::var_os("HOME")?;
    let home = Path::new(&home);

    let mut total = 0;
    let mut found = false;

    for folder in ["Applications", "AppImages"] {
        if let Some(count) = count_in_directory(&home.join(folder)) {
            total += count;
            found = true;
        }
    }

    if found { Some(total) } else { None }
}

pub fn summary() -> String {
    let mut packages = Vec::new();

    if let Some(count) = count_appimage()
        && count > 0
    {
        packages.push(format!("{count} (appimage)"));
    }

    if let Some(count) = count_dpkg() {
        packages.push(format!("{count} (dpkg)"));
    }

    if let Some(count) = count_flatpak()
        && count > 0
    {
        packages.push(format!("{count} (flatpak)"));
    }

    if let Some(count) = count_snap()
        && count > 0
    {
        packages.push(format!("{count} (snap)"));
    }

    if packages.is_empty() {
        "Unknown".to_string()
    } else {
        packages.join(", ")
    }
}
