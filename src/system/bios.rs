use std::fs;

fn read_dmi(field: &str) -> Option<String> {
    fs::read_to_string(format!("/sys/class/dmi/id/{field}"))
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

pub fn info() -> String {
    let vendor = read_dmi("bios_vendor");
    let version = read_dmi("bios_version");
    let date = read_dmi("bios_date");

    let mut parts = Vec::new();

    if let Some(vendor) = vendor {
        parts.push(vendor);
    }

    if let Some(version) = version {
        parts.push(version);
    }

    let mut result = parts.join(" ");

    if let Some(date) = date {
        if !result.is_empty() {
            result.push(' ');
        }

        result.push_str(&format!("({date})"));
    }

    if result.is_empty() {
        "Unknown".to_string()
    } else {
        result
    }
}
