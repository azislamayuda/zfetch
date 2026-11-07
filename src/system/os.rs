use super::util::{os_release, os_release_value};
use std::fs;

pub fn name() -> String {
    os_release()
        .as_deref()
        .and_then(|content| os_release_value(content, "PRETTY_NAME"))
        .map_or_else(|| "Linux".to_string(), str::to_string)
}

pub fn kernel() -> String {
    fs::read_to_string("/proc/sys/kernel/osrelease")
        .map_or_else(|_| "Unknown".to_string(), |s| s.trim().to_string())
}
