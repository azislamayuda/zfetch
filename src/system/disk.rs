use std::{collections::HashSet, ffi::CString, fs, mem::MaybeUninit, process::Command};

#[derive(Debug)]
pub struct DiskInfo {
    pub mount_point: String,
    pub filesystem: String,
    pub used: u64,
    pub total: u64,
}

impl DiskInfo {
    pub fn formatted(&self) -> String {
        let gib = 1024.0 * 1024.0 * 1024.0;

        let percent = if self.total > 0 {
            self.used as f64 / self.total as f64 * 100.0
        } else {
            0.0
        };

        format!(
            "{:.2} GiB / {:.2} GiB ({:.0}%) - {} [{}]",
            self.used as f64 / gib,
            self.total as f64 / gib,
            percent,
            self.filesystem,
            self.mount_point
        )
    }
}

fn supported_filesystem(name: &str) -> bool {
    matches!(
        name,
        "ext2"
            | "ext3"
            | "ext4"
            | "btrfs"
            | "xfs"
            | "f2fs"
            | "zfs"
            | "ntfs"
            | "ntfs3"
            | "exfat"
            | "vfat"
    )
}

// ============================================
// MOUNTINFO PARSER
// ============================================

struct MountEntry {
    source: String,
    filesystem: String,
    mount_point: String,
}

fn decode_mount_field(raw: &str) -> String {
    let bytes = raw.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] == b'\\'
            && index + 3 < bytes.len()
            && bytes[index + 1..index + 4]
                .iter()
                .all(|b| (b'0'..=b'7').contains(b))
        {
            let value = u16::from(bytes[index + 1] - b'0') * 64
                + u16::from(bytes[index + 2] - b'0') * 8
                + u16::from(bytes[index + 3] - b'0');

            if value <= 255 {
                decoded.push(value as u8);
                index += 4;
                continue;
            }
        }

        decoded.push(bytes[index]);
        index += 1;
    }

    String::from_utf8_lossy(&decoded).into_owned()
}

fn parse_mount(line: &str) -> Option<MountEntry> {
    let (left, right) = line.split_once(" - ")?;

    // /proc/self/mountinfo:
    // ID PARENT MAJ:MIN ROOT MOUNTPOINT OPTIONS ... - FSTYPE SOURCE OPTIONS

    let mount_point = left.split_whitespace().nth(4)?;

    let mut fields = right.split_whitespace();

    let filesystem = fields.next()?;

    if !supported_filesystem(filesystem) {
        return None;
    }

    let source = fields.next()?;

    Some(MountEntry {
        source: decode_mount_field(source),
        filesystem: filesystem.to_string(),
        mount_point: decode_mount_field(mount_point),
    })
}

// ============================================
// NATIVE FILESYSTEM STATISTICS
// ============================================

fn filesystem_usage(path: &str) -> Option<(u64, u64)> {
    let path = CString::new(path).ok()?;

    let mut stats = MaybeUninit::<libc::statvfs>::uninit();

    // SAFETY:
    // `path` is a valid NUL-terminated CString.
    // `stats` points to writable memory with the correct type.
    // We read the result only if statvfs returns success.
    let result = unsafe { libc::statvfs(path.as_ptr(), stats.as_mut_ptr()) };

    if result != 0 {
        return None;
    }

    // SAFETY: statvfs initialized the structure on success.
    let stats = unsafe { stats.assume_init() };

    let block_size = if stats.f_frsize > 0 {
        stats.f_frsize
    } else {
        stats.f_bsize
    };

    let total = stats.f_blocks.saturating_mul(block_size);

    let used_blocks = stats.f_blocks.saturating_sub(stats.f_bfree);

    let used = used_blocks.saturating_mul(block_size);

    Some((used, total))
}

fn native_disks() -> Vec<DiskInfo> {
    let Ok(content) = fs::read_to_string("/proc/self/mountinfo") else {
        return Vec::new();
    };

    let mut mounts: Vec<_> = content.lines().filter_map(parse_mount).collect();

    // Prefer root and shorter mount paths when
    // multiple mounts share the same source.
    mounts.sort_by(|a, b| {
        (a.mount_point != "/", a.mount_point.len(), &a.mount_point).cmp(&(
            b.mount_point != "/",
            b.mount_point.len(),
            &b.mount_point,
        ))
    });

    let mut seen_sources = HashSet::new();
    let mut disks = Vec::new();

    for mount in mounts {
        if seen_sources.contains(&mount.source) {
            continue;
        }

        let Some((used, total)) = filesystem_usage(&mount.mount_point) else {
            continue;
        };

        if total == 0 {
            continue;
        }

        seen_sources.insert(mount.source);

        disks.push(DiskInfo {
            mount_point: mount.mount_point,
            filesystem: mount.filesystem,
            used,
            total,
        });
    }

    sort_disks(&mut disks);
    disks
}

fn sort_disks(disks: &mut [DiskInfo]) {
    disks.sort_by(|a, b| {
        (a.mount_point != "/", &a.mount_point).cmp(&(b.mount_point != "/", &b.mount_point))
    });
}

// ============================================
// FALLBACK: ORIGINAL DF IMPLEMENTATION
// ============================================

fn df_fallback() -> Vec<DiskInfo> {
    let Ok(output) = Command::new("df").args(["-P", "-T", "-B1", "-l"]).output() else {
        return Vec::new();
    };

    if !output.status.success() {
        return Vec::new();
    }

    let text = String::from_utf8_lossy(&output.stdout);

    let mut disks = Vec::new();
    let mut seen_devices = HashSet::new();

    for line in text.lines().skip(1) {
        let mut parts = line.split_whitespace();

        let Some(device) = parts.next() else {
            continue;
        };

        let Some(filesystem) = parts.next() else {
            continue;
        };

        if !supported_filesystem(filesystem) {
            continue;
        }

        let Some(total) = parts.next().and_then(|v| v.parse::<u64>().ok()) else {
            continue;
        };

        let Some(used) = parts.next().and_then(|v| v.parse::<u64>().ok()) else {
            continue;
        };

        let _available = parts.next();
        let _percentage = parts.next();

        let mount_point = parts.collect::<Vec<_>>().join(" ");

        if total == 0 || mount_point.is_empty() {
            continue;
        }

        if !seen_devices.insert(device.to_string()) {
            continue;
        }

        disks.push(DiskInfo {
            mount_point,
            filesystem: filesystem.to_string(),
            used,
            total,
        });
    }

    sort_disks(&mut disks);
    disks
}

// ============================================
// PUBLIC API
// ============================================

pub fn all() -> Vec<DiskInfo> {
    let disks = native_disks();

    if disks.is_empty() {
        df_fallback()
    } else {
        disks
    }
}

// ============================================
// UNIT TESTS
// ============================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mountinfo_parser() {
        let line = concat!("42 35 8:1 / / rw,relatime - ", "ext4 /dev/sda1 rw");

        let mount = parse_mount(line).unwrap();

        assert_eq!(mount.mount_point, "/");
        assert_eq!(mount.filesystem, "ext4");
        assert_eq!(mount.source, "/dev/sda1");
    }

    #[test]
    fn test_escaped_mount_path() {
        assert_eq!(decode_mount_field("/media/My\\040Disk"), "/media/My Disk");
    }

    #[test]
    fn test_supported_filesystems() {
        assert!(supported_filesystem("ext4"));
        assert!(supported_filesystem("btrfs"));
        assert!(!supported_filesystem("tmpfs"));
        assert!(!supported_filesystem("overlay"));
    }

    #[test]
    fn test_root_filesystem_usage() {
        let result = filesystem_usage("/");
        assert!(result.is_some());

        let (used, total) = result.unwrap();
        assert!(total > 0);
        assert!(used <= total);
    }
}
