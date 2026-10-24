//! Explicit-only GitHub Releases update checker and .deb installer.
//! This module is NEVER called while rendering normal system information.

use serde::Deserialize;
use std::{
    env, fs,
    io::{self, Write},
    os::unix::fs::DirBuilderExt,
    path::{Path, PathBuf},
    process::{self, Command},
    time::{SystemTime, UNIX_EPOCH},
};

const RELEASE_API: &str = "https://api.github.com/repos/azislamayuda/zfetch/releases/latest";
const RELEASE_URL_PREFIX: &str = "https://github.com/azislamayuda/zfetch/releases/";
const ASSET_URL_PREFIX: &str = "https://github.com/azislamayuda/zfetch/releases/download/";

#[derive(Debug, Deserialize)]
struct Release {
    tag_name: String,
    html_url: String,
    draft: bool,
    prerelease: bool,
    assets: Vec<Asset>,
}

#[derive(Debug, Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
    size: u64,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Version(u64, u64, u64);

#[derive(Debug, PartialEq, Eq)]
enum ReleaseStatus {
    LocalNewer,
    SameVersion,
    UpdateAvailable,
}

fn release_status(current: &Version, latest: &Version) -> ReleaseStatus {
    if latest < current {
        ReleaseStatus::LocalNewer
    } else if latest == current {
        ReleaseStatus::SameVersion
    } else {
        ReleaseStatus::UpdateAvailable
    }
}

fn parse_version(input: &str) -> Option<Version> {
    let clean = input.trim().trim_start_matches('v');
    let mut parts = clean.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some(Version(major, minor, patch))
}

fn curl_bytes(url: &str, output: Option<&Path>) -> Result<Vec<u8>, String> {
    let mut cmd = Command::new("curl");
    cmd.args([
        "--fail",
        "--silent",
        "--show-error",
        "--location",
        "--proto",
        "=https",
        "--proto-redir",
        "=https",
        "--max-redirs",
        "5",
        "--connect-timeout",
        "8",
        "--max-time",
        if output.is_some() { "120" } else { "12" },
        "--max-filesize",
        if output.is_some() {
            "67108864"
        } else {
            "1048576"
        },
        "--user-agent",
        "zfetch-updater/0.2.0",
    ]);
    if let Some(path) = output {
        cmd.arg("--output").arg(path);
    }
    cmd.arg(url);
    let result = cmd
        .output()
        .map_err(|error| format!("cannot run curl: {error}"))?;
    if !result.status.success() {
        let details = String::from_utf8_lossy(&result.stderr);
        return Err(format!("network request failed: {}", details.trim()));
    }
    Ok(result.stdout)
}

fn fetch_release() -> Result<Release, String> {
    let json = curl_bytes(RELEASE_API, None)?;
    let text =
        std::str::from_utf8(&json).map_err(|_| "GitHub returned non-UTF-8 response".to_string())?;
    let release: Release =
        json5::from_str(text).map_err(|error| format!("cannot parse GitHub release: {error}"))?;
    if release.draft || release.prerelease {
        return Err("latest release is not stable".into());
    }
    if !release.html_url.starts_with(RELEASE_URL_PREFIX) {
        return Err("unexpected GitHub release URL".into());
    }
    Ok(release)
}

fn native_deb_arch() -> Option<&'static str> {
    match env::consts::ARCH {
        "x86_64" => Some("amd64"),
        "aarch64" => Some("arm64"),
        _ => None,
    }
}

fn trusted_asset_url(release: &Release, asset: &Asset) -> bool {
    let prefix = format!("{ASSET_URL_PREFIX}{}/", release.tag_name);
    let expected = format!("{prefix}{}", asset.name);
    asset.browser_download_url == expected
}

fn deb_asset<'a>(release: &'a Release, version: &str, arch: &str) -> Option<&'a Asset> {
    let expected_start = format!("zfetch_{version}-");
    release.assets.iter().find(|asset| {
        asset.name.starts_with(&expected_start)
            && asset.name.ends_with(&format!("_{arch}.deb"))
            && asset
                .name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-')
            && trusted_asset_url(release, asset)
            && asset.size > 0
            && asset.size <= 64 * 1024 * 1024
    })
}

fn sha256_asset(release: &Release) -> Option<&Asset> {
    release.assets.iter().find(|asset| {
        asset.name == "SHA256SUMS"
            && trusted_asset_url(release, asset)
            && asset.size > 0
            && asset.size <= 1024 * 1024
    })
}

fn expected_checksum(contents: &str, filename: &str) -> Option<String> {
    contents.lines().find_map(|line| {
        let mut parts = line.split_whitespace();
        let digest = parts.next()?;
        let listed = parts.next()?.trim_start_matches('*');
        if listed == filename
            && digest.len() == 64
            && digest.bytes().all(|byte| byte.is_ascii_hexdigit())
        {
            Some(digest.to_ascii_lowercase())
        } else {
            None
        }
    })
}

fn verify_checksum(file: &Path, expected: &str) -> Result<(), String> {
    let output = Command::new("sha256sum")
        .arg(file)
        .output()
        .map_err(|error| format!("cannot run sha256sum: {error}"))?;
    if !output.status.success() {
        return Err("could not compute SHA-256 checksum".into());
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let actual = text.split_whitespace().next().unwrap_or("");
    if !actual.eq_ignore_ascii_case(expected) {
        return Err("SHA-256 MISMATCH; refusing to install downloaded package".into());
    }
    Ok(())
}

fn verify_deb_package(path: &Path, version: &str, arch: &str) -> Result<(), String> {
    let output = Command::new("dpkg-deb")
        .arg("--field")
        .arg(path)
        .args(["Package", "Version", "Architecture"])
        .output()
        .map_err(|error| format!("cannot inspect .deb metadata: {error}"))?;
    if !output.status.success() {
        return Err("downloaded .deb has invalid metadata".into());
    }
    let fields = String::from_utf8_lossy(&output.stdout);
    let prefix = format!("Version: {version}-");
    if !fields.lines().any(|line| line.trim() == "Package: zfetch")
        || !fields.lines().any(|line| line.starts_with(&prefix))
        || !fields
            .lines()
            .any(|line| line.trim() == format!("Architecture: {arch}"))
    {
        return Err("package name, version, or architecture does not match release".into());
    }
    Ok(())
}

fn installed_by_dpkg() -> bool {
    // Do not update a source build or a shadowed user/local binary.
    let Ok(exe) = env::current_exe().and_then(fs::canonicalize) else {
        return false;
    };
    let Ok(packaged) = fs::canonicalize("/usr/bin/zfetch") else {
        return false;
    };
    if exe != packaged {
        return false;
    }

    let Ok(result) = Command::new("dpkg-query")
        .args(["-W", "-f=${Status}", "zfetch"])
        .output()
    else {
        return false;
    };
    result.status.success()
        && String::from_utf8_lossy(&result.stdout).trim() == "install ok installed"
}

struct TemporaryDirectory(PathBuf);

impl TemporaryDirectory {
    fn new() -> Result<Self, String> {
        for attempt in 0..8 {
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|error| error.to_string())?
                .as_nanos();
            let path =
                env::temp_dir().join(format!("zfetch-update-{}-{nanos}-{attempt}", process::id()));
            let mut builder = fs::DirBuilder::new();
            builder.mode(0o700);
            match builder.create(&path) {
                Ok(()) => return Ok(Self(path)),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(format!("cannot create update directory: {error}")),
            }
        }
        Err("could not create unique update directory".into())
    }
}

impl Drop for TemporaryDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

pub fn run(check_only: bool) -> Result<(), String> {
    println!("zfetch Update Manager");
    println!("Checking official GitHub Releases...");

    let release = fetch_release()?;
    let current = env!("CARGO_PKG_VERSION");
    let current_version = parse_version(current).ok_or("invalid local application version")?;
    let remote_version = parse_version(&release.tag_name)
        .ok_or("latest release has an unsupported version format")?;

    println!("Current version : v{current}");
    println!("Latest release  : {}", release.tag_name);
    println!("Release         : {}", release.html_url);

    match release_status(&current_version, &remote_version) {
        ReleaseStatus::LocalNewer => {
            println!("Your current build is newer than the latest public release.");
            println!("No update required.");
            return Ok(());
        }
        ReleaseStatus::SameVersion => {
            println!("You are running the latest public release.");
            return Ok(());
        }
        ReleaseStatus::UpdateAvailable => {}
    }

    if check_only {
        println!("A newer version is available. Run 'zfetch --update' to review it.");
        return Ok(());
    }

    if !installed_by_dpkg() {
        return Err(format!(
            "this zfetch executable is not managed by dpkg. Download and install manually from {}",
            release.html_url
        ));
    }

    let arch = native_deb_arch().ok_or("automatic .deb update supports only x86_64 and aarch64")?;

    let version = release.tag_name.trim_start_matches('v');
    let deb = deb_asset(&release, version, arch)
        .ok_or(format!("latest release has no matching {arch} .deb asset"))?;
    let sums = sha256_asset(&release)
        .ok_or("release has no SHA256SUMS asset; automatic update refused for safety")?;

    println!("Package: {} ({} bytes)", deb.name, deb.size);
    print!("Download, verify SHA-256, and install via APT? [y/N]: ");
    io::stdout().flush().map_err(|error| error.to_string())?;
    let mut answer = String::new();
    io::stdin()
        .read_line(&mut answer)
        .map_err(|error| error.to_string())?;
    if !matches!(answer.trim(), "y" | "Y" | "yes" | "YES") {
        println!("Update cancelled. No files were changed.");
        return Ok(());
    }

    let workspace = TemporaryDirectory::new()?;
    let deb_path = workspace.0.join(&deb.name);
    let sums_path = workspace.0.join("SHA256SUMS");

    println!("Downloading {}...", deb.name);
    curl_bytes(&deb.browser_download_url, Some(&deb_path))?;
    curl_bytes(&sums.browser_download_url, Some(&sums_path))?;

    let actual_size = fs::metadata(&deb_path)
        .map_err(|error| format!("missing downloaded package: {error}"))?
        .len();
    if actual_size != deb.size {
        return Err("downloaded package size differs from GitHub asset metadata".into());
    }

    let sum_size = fs::metadata(&sums_path)
        .map_err(|error| format!("missing checksum file: {error}"))?
        .len();
    if sum_size > 1024 * 1024 || sum_size != sums.size {
        return Err("invalid checksum file size".into());
    }
    let sum_text = fs::read_to_string(&sums_path)
        .map_err(|error| format!("cannot read SHA256SUMS: {error}"))?;
    let expected = expected_checksum(&sum_text, &deb.name)
        .ok_or("SHA256SUMS is missing a valid checksum for the .deb package")?;
    verify_checksum(&deb_path, &expected)?;
    verify_deb_package(&deb_path, version, arch)?;
    println!("Checksum and package metadata verified.");
    println!("Starting APT installation (sudo may request your password)...");

    let status = Command::new("sudo")
        .arg("apt-get")
        .arg("install")
        .arg("--")
        .arg(&deb_path)
        .status()
        .map_err(|error| format!("cannot launch sudo apt-get: {error}"))?;

    if !status.success() {
        return Err(format!("APT installation exited with status {status}"));
    }

    println!("Update installed. Open a new terminal and run 'zfetch --version'.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_arch_is_supported_or_explicitly_rejected() {
        let actual = native_deb_arch();
        assert!(matches!(actual, Some("amd64") | Some("arm64") | None));
    }

    #[test]
    fn asset_urls_must_match_release_exactly() {
        let release = Release {
            tag_name: "v0.4.0".into(),
            html_url: format!("{RELEASE_URL_PREFIX}tag/v0.4.0"),
            draft: false,
            prerelease: false,
            assets: vec![],
        };
        let good = Asset {
            name: "zfetch_0.4.0-1_arm64.deb".into(),
            browser_download_url: format!("{ASSET_URL_PREFIX}v0.4.0/zfetch_0.4.0-1_arm64.deb"),
            size: 1234,
        };
        assert!(trusted_asset_url(&release, &good));
        let mut bad = good;
        bad.browser_download_url =
            "https://github.com/azislamayuda/zfetch/releases/download/v0.4.0/another.deb".into();
        assert!(!trusted_asset_url(&release, &bad));
    }

    #[test]
    fn compares_versions_numerically() {
        assert!(parse_version("v0.2.10") > parse_version("0.2.9"));
        assert!(parse_version("v1.0.0") > parse_version("0.99.99"));
        assert_eq!(parse_version("0.2.0"), Some(Version(0, 2, 0)));
        assert!(parse_version("0.2.0-beta").is_none());
    }

    #[test]
    fn categorizes_release_versions() {
        let current = Version(0, 2, 0);
        assert_eq!(
            release_status(&current, &Version(0, 1, 0)),
            ReleaseStatus::LocalNewer
        );
        assert_eq!(
            release_status(&current, &Version(0, 2, 0)),
            ReleaseStatus::SameVersion
        );
        assert_eq!(
            release_status(&current, &Version(0, 2, 1)),
            ReleaseStatus::UpdateAvailable
        );
    }

    #[test]
    fn reads_expected_checksum_only() {
        let sum = format!(
            "{}  other.deb\n{}  zfetch_0.2.1-1_amd64.deb\n",
            "a".repeat(64),
            "b".repeat(64)
        );
        assert_eq!(
            expected_checksum(&sum, "zfetch_0.2.1-1_amd64.deb"),
            Some("b".repeat(64))
        );
        assert!(expected_checksum(&sum, "zfetch_0.2.2-1_amd64.deb").is_none());
    }
}
