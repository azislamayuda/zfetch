use std::{
    env, fs,
    path::{Path, PathBuf},
};

// ============================================
// DPKG DETECTION
// ============================================

fn count_dpkg() -> Option<usize> {
    use std::io::BufRead;

    let file = fs::File::open("/var/lib/dpkg/status").ok()?;

    let mut reader = std::io::BufReader::with_capacity(64 * 1024, file);

    let mut line = Vec::with_capacity(256);
    let mut count = 0usize;

    loop {
        line.clear();

        match reader.read_until(b'\n', &mut line) {
            Ok(0) => break,

            Ok(_) => {
                if line.last() == Some(&b'\n') {
                    line.pop();
                }

                if line.last() == Some(&b'\r') {
                    line.pop();
                }

                if line.as_slice() == b"Status: install ok installed" {
                    count += 1;
                }
            }

            Err(_) => return None,
        }
    }

    Some(count)
}

// ============================================
// FILESYSTEM HELPERS
// ============================================

fn directories(path: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(path) else {
        return Vec::new();
    };

    entries
        .flatten()
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
        .map(|entry| entry.path())
        .collect()
}

// ============================================
// FLATPAK DETECTION
// ============================================

// Count installed apps and runtimes directly
// from Flatpak installation directories.

fn count_flatpak_at(root: &Path) -> usize {
    // Applications with an active installation.
    let apps = directories(&root.join("app"))
        .into_iter()
        .filter(|app| app.join("current").exists())
        .count();

    // Installed runtime branches.
    let runtimes = directories(&root.join("runtime"))
        .into_iter()
        .filter(|runtime| {
            let name = runtime
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");

            let suffix = name.rsplit('.').next().unwrap_or("");

            suffix != "Locale" && suffix != "Debug"
        })
        .map(|runtime| {
            directories(&runtime)
                .into_iter()
                .map(|arch| directories(&arch).len())
                .sum::<usize>()
        })
        .sum::<usize>();

    apps + runtimes
}

fn count_flatpak() -> Option<usize> {
    let system_path = Path::new("/var/lib/flatpak");

    let user_path =
        env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share/flatpak"));

    let system_exists = system_path.is_dir();

    let user_exists = user_path.as_ref().is_some_and(|path| path.is_dir());

    if !system_exists && !user_exists {
        return None;
    }

    let mut total = count_flatpak_at(system_path);

    if let Some(path) = user_path {
        total += count_flatpak_at(&path);
    }

    Some(total)
}

// ============================================
// SNAP DETECTION
// ============================================

fn count_snap() -> Option<usize> {
    let snap_path = Path::new("/snap");
    let snapd_path = Path::new("/var/lib/snapd/snap");

    if !snap_path.is_dir() && !snapd_path.is_dir() {
        return None;
    }

    let mut count = directories(snap_path).len();

    if count == 0 {
        count = directories(snapd_path).len();
    }

    // Match Fastfetch counting, excluding
    // the Snap bin directory.
    Some(count.saturating_sub(1))
}

// ============================================
// APPIMAGE DETECTION
// ============================================

fn count_appimages_at(path: &Path) -> usize {
    let Ok(entries) = fs::read_dir(path) else {
        return 0;
    };

    entries
        .flatten()
        .filter(|entry| {
            let path = entry.path();

            path.is_file()
                && path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("appimage"))
        })
        .count()
}

fn count_appimage() -> Option<usize> {
    let home = env::var_os("HOME")?;
    let home = PathBuf::from(home);

    let locations = [home.join("Applications"), home.join("AppImages")];

    let found = locations.iter().any(|path| path.is_dir());

    if !found {
        return None;
    }

    Some(locations.iter().map(|path| count_appimages_at(path)).sum())
}

// ============================================
// PACKAGE SUMMARY
// ============================================

pub fn summary() -> String {
    let mut result = Vec::new();

    if let Some(count) = count_appimage()
        && count > 0
    {
        result.push(format!("{count} (appimage)"));
    }

    if let Some(count) = count_dpkg() {
        result.push(format!("{count} (dpkg)"));
    }

    if let Some(count) = count_flatpak()
        && count > 0
    {
        result.push(format!("{count} (flatpak)"));
    }

    if let Some(count) = count_snap()
        && count > 0
    {
        result.push(format!("{count} (snap)"));
    }

    if result.is_empty() {
        "Unknown".to_string()
    } else {
        result.join(", ")
    }
}

// ============================================
// UNIT TESTS
// ============================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nonexistent_directory_is_empty() {
        let path = Path::new("/nonexistent-zfetch-package-test-directory");

        assert!(directories(path).is_empty());
    }

    #[test]
    fn missing_flatpak_directory_is_zero() {
        let path = Path::new("/nonexistent-zfetch-flatpak-test-directory");

        assert_eq!(count_flatpak_at(path), 0);
    }
}
