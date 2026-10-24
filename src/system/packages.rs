//! Cross-distro package detection for zfetch.
//! Native filesystem/database reads are preferred; RPM uses the installed rpm tool
//! because direct rpmdb parsing depends on the database backend and version.
use std::{
    env, fs,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

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

fn count_dpkg_at(path: &Path) -> Option<usize> {
    let file = fs::File::open(path).ok()?;
    count_dpkg_reader(BufReader::with_capacity(64 * 1024, file)).ok()
}

fn count_dpkg_reader<R: BufRead>(mut reader: R) -> std::io::Result<usize> {
    let mut count = 0;
    let mut line = Vec::new();
    loop {
        line.clear();
        if reader.read_until(b'\n', &mut line)? == 0 {
            break;
        }
        while matches!(line.last(), Some(b'\n' | b'\r')) {
            line.pop();
        }
        if line == b"Status: install ok installed" {
            count += 1;
        }
    }
    Ok(count)
}

fn count_flatpak_at(root: &Path) -> usize {
    let apps = directories(&root.join("app"))
        .into_iter()
        .filter(|app| app.join("current").exists())
        .count();
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
    let system = Path::new("/var/lib/flatpak");
    let user = env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share/flatpak"));
    if !system.is_dir() && !user.as_ref().is_some_and(|path| path.is_dir()) {
        return None;
    }
    Some(count_flatpak_at(system) + user.as_ref().map_or(0, |path| count_flatpak_at(path)))
}

// pacman local database: each installed package has one directory such as nano-8.5-1.
fn count_pacman_at(path: &Path) -> Option<usize> {
    if !path.is_dir() {
        return None;
    }
    Some(
        directories(path)
            .into_iter()
            .filter(|p| p.join("desc").is_file())
            .count(),
    )
}

// Snap installations are listed under /var/lib/snapd/snaps as .snap files.
// Exclude snapd itself (not an application), and count only one revision per name.
fn count_snap_files_at(path: &Path) -> Option<usize> {
    let entries = fs::read_dir(path).ok()?;
    let mut names = std::collections::HashSet::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if !entry.file_type().is_ok_and(|ft| ft.is_file()) {
            continue;
        }
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if !name.ends_with(".snap") {
            continue;
        }
        let Some((package, _revision)) = name.trim_end_matches(".snap").rsplit_once('_') else {
            continue;
        };
        if package != "snapd" {
            names.insert(package.to_string());
        }
    }
    Some(names.len())
}

fn count_snap_dirs_at(path: &Path) -> Option<usize> {
    if !path.is_dir() {
        return None;
    }
    Some(
        directories(path)
            .into_iter()
            .filter(|p| {
                let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
                !matches!(name, "bin" | "snapd")
            })
            .count(),
    )
}

fn count_snap() -> Option<usize> {
    count_snap_files_at(Path::new("/var/lib/snapd/snaps"))
        .or_else(|| count_snap_dirs_at(Path::new("/snap")))
        .or_else(|| count_snap_dirs_at(Path::new("/var/lib/snapd/snap")))
}

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
    let home = PathBuf::from(env::var_os("HOME")?);
    let roots = [home.join("Applications"), home.join("AppImages")];
    if !roots.iter().any(|root| root.is_dir()) {
        return None;
    }
    Some(roots.iter().map(|root| count_appimages_at(root)).sum())
}

// RPM database can use SQLite, Berkeley DB, or NDB. Use rpm's own query to
// avoid fragile direct parsing. It runs only if an rpm database is present.
fn rpm_database_present() -> bool {
    ["/usr/lib/sysimage/rpm", "/var/lib/rpm"]
        .iter()
        .any(|path| {
            let root = Path::new(path);
            root.join("rpmdb.sqlite").exists()
                || root.join("Packages.db").exists()
                || root.join("Packages").exists()
        })
}
fn count_rpm() -> Option<usize> {
    if !rpm_database_present() {
        return None;
    }
    let output = Command::new("rpm")
        .args(["-qa", "--qf", "%{NAME}\\n"])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(output.stdout.iter().filter(|&&c| c == b'\n').count())
}

pub fn summary() -> String {
    let mut out = Vec::new();
    let mut add = |name: &str, count: Option<usize>, include_zero: bool| {
        if let Some(n) = count
            && (include_zero || n > 0)
        {
            out.push(format!("{n} ({name})"));
        }
    };
    add("appimage", count_appimage(), false);
    add(
        "dpkg",
        count_dpkg_at(Path::new("/var/lib/dpkg/status")),
        true,
    );
    add("rpm", count_rpm(), true);
    add(
        "pacman",
        count_pacman_at(Path::new("/var/lib/pacman/local")),
        true,
    );
    add("flatpak", count_flatpak(), false);
    add("snap", count_snap(), false);
    if out.is_empty() {
        "Unknown".into()
    } else {
        out.join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    #[test]
    fn dpkg_counts_only_installed_records() {
        let fixture = b"Package: one\nStatus: install ok installed\n\nPackage: two\nStatus: deinstall ok config-files\n\nPackage: three\nStatus: install ok installed\n";
        assert_eq!(count_dpkg_reader(Cursor::new(fixture)).unwrap(), 2);
    }
    #[test]
    fn missing_package_databases_are_ignored() {
        let missing = Path::new("/nonexistent-zfetch-fixture-4c89e3");
        assert_eq!(count_pacman_at(missing), None);
        assert_eq!(count_snap_files_at(missing), None);
        assert_eq!(count_flatpak_at(missing), 0);
    }
    #[test]
    fn snap_names_are_deduplicated() {
        let root = std::env::temp_dir().join(format!(
            "zfetch-snap-test-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        fs::create_dir_all(&root).unwrap();
        for name in [
            "hello_1.snap",
            "hello_2.snap",
            "core22_5.snap",
            "snapd_3.snap",
        ] {
            fs::write(root.join(name), b"").unwrap();
        }
        assert_eq!(count_snap_files_at(&root), Some(2));
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn pacman_uses_desc_files() {
        let root = std::env::temp_dir().join(format!(
            "zfetch-pacman-test-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        fs::create_dir_all(root.join("nano-8-1")).unwrap();
        fs::create_dir_all(root.join("incomplete-1-1")).unwrap();
        fs::write(root.join("nano-8-1/desc"), b"%NAME%\nnano\n").unwrap();
        assert_eq!(count_pacman_at(&root), Some(1));
        fs::remove_dir_all(root).unwrap();
    }
}
