use std::fs;

pub fn formatted() -> String {
    let Ok(content) = fs::read_to_string("/proc/uptime") else {
        return "Unknown".to_string();
    };

    let seconds = content
        .split_whitespace()
        .next()
        .and_then(|value| value.split('.').next())
        .and_then(|value| value.parse::<u64>().ok());

    let Some(seconds) = seconds else {
        return "Unknown".to_string();
    };

    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;

    if days > 0 {
        format!("{days}d {hours}h {minutes}m")
    } else {
        format!("{hours}h {minutes}m")
    }
}
