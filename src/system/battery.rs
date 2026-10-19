use std::fs;

pub fn status() -> Option<String> {
    let entries = fs::read_dir("/sys/class/power_supply").ok()?;

    for entry in entries.flatten() {
        let path = entry.path();

        let Ok(kind) = fs::read_to_string(path.join("type")) else {
            continue;
        };

        if kind.trim() != "Battery" {
            continue;
        }

        let percentage = fs::read_to_string(path.join("capacity"))
            .ok()
            .and_then(|s| s.trim().parse::<u8>().ok());

        let state = fs::read_to_string(path.join("status"))
            .ok()
            .map(|s| s.trim().to_string());

        let result = match (percentage, state) {
            (Some(percent), Some(state)) => {
                format!("{percent}% ({state})")
            }
            (Some(percent), None) => format!("{percent}%"),
            (None, Some(state)) => state,
            (None, None) => "Detected".to_string(),
        };

        return Some(result);
    }

    None
}
