use std::fs;

fn read_kib(content: &str, key: &str) -> Option<u64> {
    for line in content.lines() {
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };

        if name.trim() == key {
            return value.split_whitespace().next()?.parse::<u64>().ok();
        }
    }

    None
}

pub fn usage() -> String {
    let Ok(content) = fs::read_to_string("/proc/meminfo") else {
        return "Unknown".to_string();
    };

    let Some(total) = read_kib(&content, "MemTotal") else {
        return "Unknown".to_string();
    };

    let Some(available) = read_kib(&content, "MemAvailable") else {
        return "Unknown".to_string();
    };

    let used = total.saturating_sub(available);
    let gib = 1024.0 * 1024.0;

    format!(
        "{:.2} GiB / {:.2} GiB",
        used as f64 / gib,
        total as f64 / gib
    )
}
