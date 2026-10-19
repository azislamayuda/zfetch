use std::fs;

pub fn name() -> String {
    let Ok(content) = fs::read_to_string("/proc/cpuinfo") else {
        return "Unknown".to_string();
    };

    for line in content.lines() {
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim();
            let value = value.trim();

            if matches!(key, "model name" | "Hardware" | "Processor") && !value.is_empty() {
                return value.to_string();
            }
        }
    }

    "Unknown".to_string()
}
