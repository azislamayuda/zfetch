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

    let Some(total) = read_kib(&content, "SwapTotal") else {
        return "Unknown".to_string();
    };

    let Some(free) = read_kib(&content, "SwapFree") else {
        return "Unknown".to_string();
    };

    if total == 0 {
        return "Disabled".to_string();
    }

    let used = total.saturating_sub(free);
    let gib = 1024.0 * 1024.0;

    let percentage = used.saturating_mul(100) / total;

    format!(
        "{:.2} GiB / {:.2} GiB ({}%)",
        used as f64 / gib,
        total as f64 / gib,
        percentage
    )
}
