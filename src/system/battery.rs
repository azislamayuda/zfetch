use std::fs;

#[derive(Debug, Default, Clone)]
pub struct Battery {
    pub percent: Option<u8>,
    pub state: Option<String>,
    /// Optional telemetry such as " | ~1h 02m remaining".
    pub extra: String,
}

impl Battery {
    pub fn formatted(&self, with_percent: bool) -> String {
        let base = match (self.percent, &self.state) {
            (Some(percent), Some(state)) if with_percent => format!("{percent}% ({state})"),
            (_, Some(state)) => state.clone(),
            (Some(percent), None) => format!("{percent}%"),
            (None, None) => "Detected".to_string(),
        };
        base + &self.extra
    }
}

pub fn status() -> Option<Battery> {
    let entries = fs::read_dir("/sys/class/power_supply").ok()?;

    for entry in entries.flatten() {
        let path = entry.path();

        let Ok(kind) = fs::read_to_string(path.join("type")) else {
            continue;
        };

        if kind.trim() != "Battery" {
            continue;
        }

        return Some(Battery {
            percent: fs::read_to_string(path.join("capacity"))
                .ok()
                .and_then(|s| s.trim().parse::<u8>().ok()),
            state: fs::read_to_string(path.join("status"))
                .ok()
                .map(|s| s.trim().to_string()),
            extra: String::new(),
        });
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn battery_formats_with_and_without_percent() {
        let battery = Battery {
            percent: Some(39),
            state: Some("Charging".into()),
            extra: String::new(),
        };
        assert_eq!(battery.formatted(true), "39% (Charging)");
        assert_eq!(battery.formatted(false), "Charging");
        let bare = Battery {
            percent: Some(80),
            ..Battery::default()
        };
        assert_eq!(bare.formatted(true), "80%");
        assert_eq!(bare.formatted(false), "80%");
        assert_eq!(Battery::default().formatted(true), "Detected");
    }
}
