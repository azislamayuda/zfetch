use std::{collections::HashSet, process::Command};

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

fn supported_filesystem(fs: &str) -> bool {
    matches!(
        fs,
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

pub fn all() -> Vec<DiskInfo> {
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

        // Skip available bytes and percentage columns.
        let _available = parts.next();
        let _percentage = parts.next();

        let mount_point = parts.collect::<Vec<_>>().join(" ");

        if total == 0 || mount_point.is_empty() {
            continue;
        }

        // Avoid repeated bind mounts of the same device.
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

    // Show root filesystem first.
    disks.sort_by(|a, b| {
        (a.mount_point != "/", &a.mount_point).cmp(&(b.mount_point != "/", &b.mount_point))
    });

    disks
}
