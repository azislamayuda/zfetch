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
pub fn rpm_database_present() -> bool {
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
    Some(
        output
            .stdout
            .split(|&byte| byte == b'\n')
            .filter(|name| !name.is_empty())
            .count(),
    )
}

// Alpine: one "P:<name>" line per installed package record.
fn count_apk_at(path: &Path) -> Option<usize> {
    let text = fs::read_to_string(path).ok()?;
    Some(text.lines().filter(|line| line.starts_with("P:")).count())
}

// Gentoo Portage: /var/db/pkg/<category>/<package-version>/.
fn count_portage_at(path: &Path) -> Option<usize> {
    if !path.is_dir() {
        return None;
    }
    Some(directories(path).iter().map(|c| directories(c).len()).sum())
}

// Void: pkgdb-<ver>.plist holds every package; installed ones carry
// <key>state</key><string>installed</string>. Counting *.plist files would
// include pkgdb itself and miss packages without a files list.
fn count_xbps_at(path: &Path) -> Option<usize> {
    let pkgdb = fs::read_dir(path).ok()?.flatten().find(|entry| {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        name.starts_with("pkgdb-") && name.ends_with(".plist")
    })?;
    let text = fs::read_to_string(pkgdb.path()).ok()?;
    Some(
        text.lines()
            .filter(|line| line.trim() == "<string>installed</string>")
            .count(),
    )
}

pub fn summary() -> String {
    format_summary(&counts())
}

/// "2134 (dpkg), 12 (flatpak)", or "Unknown" when no manager was found.
pub fn format_summary(counts: &[(&str, usize)]) -> String {
    if counts.is_empty() {
        return "Unknown".into();
    }
    counts
        .iter()
        .map(|(name, n)| format!("{n} ({name})"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Per-manager counts in display order; system managers are kept at zero, add-ons are not.
pub fn counts() -> Vec<(&'static str, usize)> {
    let mut out = Vec::new();
    let mut add = |name: &'static str, count: Option<usize>, include_zero: bool| {
        if let Some(n) = count
            && (include_zero || n > 0)
        {
            out.push((name, n));
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
    add(
        "apk",
        count_apk_at(Path::new("/lib/apk/db/installed")),
        true,
    );
    add("portage", count_portage_at(Path::new("/var/db/pkg")), true);
    add("xbps", count_xbps_at(Path::new("/var/db/xbps")), true);
    add("flatpak", count_flatpak(), false);
    add("snap", count_snap(), false);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    /// The v0.5.0 `summary()` body, kept verbatim as the byte-identity reference.
    fn summary_v050(input: &[(&str, Option<usize>, bool)]) -> String {
        let mut out = Vec::new();
        let mut add = |name: &str, count: Option<usize>, include_zero: bool| {
            if let Some(n) = count
                && (include_zero || n > 0)
            {
                out.push(format!("{n} ({name})"));
            }
        };
        for &(name, count, zero) in input {
            add(name, count, zero);
        }
        if out.is_empty() {
            "Unknown".into()
        } else {
            out.join(", ")
        }
    }

    #[test]
    fn split_counts_rebuild_identical_summary() {
        let fixtures: [&[(&str, Option<usize>, bool)]; 4] = [
            &[],
            &[("dpkg", Some(2134), true), ("flatpak", Some(12), false)],
            &[
                ("appimage", Some(0), false),
                ("dpkg", Some(0), true),
                ("rpm", None, true),
                ("snap", Some(3), false),
            ],
            &[("flatpak", Some(0), false), ("snap", None, false)],
        ];
        for input in fixtures {
            let counts: Vec<(&str, usize)> = input
                .iter()
                .filter_map(|&(name, count, zero)| {
                    count.filter(|&n| zero || n > 0).map(|n| (name, n))
                })
                .collect();
            assert_eq!(format_summary(&counts), summary_v050(input));
        }
    }

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
    fn fixture_dir(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "zfetch-{name}-test-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn apk_counts_package_records() {
        let root = fixture_dir("apk");
        let db = root.join("installed");
        fs::write(
            &db,
            "C:Q1abc=\nP:musl\nV:1.2.5-r0\n\nC:Q1def=\nP:busybox\nV:1.36\n",
        )
        .unwrap();
        assert_eq!(count_apk_at(&db), Some(2));
        assert_eq!(count_apk_at(&root.join("missing")), None);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn portage_counts_category_package_dirs() {
        let root = fixture_dir("portage");
        for dir in [
            "sys-apps/portage-3.0",
            "sys-apps/sed-4.9",
            "app-editors/nano-8.0",
        ] {
            fs::create_dir_all(root.join(dir)).unwrap();
        }
        assert_eq!(count_portage_at(&root), Some(3));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn xbps_counts_installed_states() {
        let root = fixture_dir("xbps");
        fs::write(
            root.join("pkgdb-0.38.plist"),
            "<dict>\n<key>bash</key>\n<dict>\n<key>state</key>\n<string>installed</string>\n</dict>\n<key>old</key>\n<dict>\n<key>state</key>\n<string>half-removed</string>\n</dict>\n<key>nano</key>\n<dict>\n<key>state</key>\n<string>installed</string>\n</dict>\n</dict>\n",
        )
        .unwrap();
        fs::write(root.join(".bash-files.plist"), "").unwrap();
        assert_eq!(count_xbps_at(&root), Some(2));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn new_parsers_handle_missing_empty_and_malformed_input() {
        let root = fixture_dir("edge");
        let missing = root.join("missing");
        assert_eq!(count_apk_at(&missing), None);
        assert_eq!(count_portage_at(&missing), None);
        assert_eq!(count_xbps_at(&missing), None);

        let apk = root.join("installed");
        fs::write(&apk, "").unwrap();
        assert_eq!(count_apk_at(&apk), Some(0));
        fs::write(&apk, "garbage\nPX:no\n P:indented\np:lower\nP:musl\n").unwrap();
        assert_eq!(count_apk_at(&apk), Some(1));
        fs::write(&apk, [0xff, 0xfe, b'\n']).unwrap();
        assert_eq!(
            count_apk_at(&apk),
            None,
            "non-UTF-8 is unreadable, not a panic"
        );

        // A file where a directory is expected, and stray files inside a category.
        assert_eq!(count_portage_at(&apk), None);
        let portage = root.join("pkg");
        fs::create_dir_all(portage.join("sys-apps/sed-4.9")).unwrap();
        fs::write(portage.join("sys-apps/stray-file"), "").unwrap();
        fs::write(portage.join("stray-top-file"), "").unwrap();
        assert_eq!(count_portage_at(&portage), Some(1));

        let xbps = root.join("xbps");
        fs::create_dir_all(&xbps).unwrap();
        assert_eq!(count_xbps_at(&xbps), None, "no pkgdb file");
        fs::write(xbps.join("pkgdb-0.38.plist"), "").unwrap();
        assert_eq!(count_xbps_at(&xbps), Some(0));
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
