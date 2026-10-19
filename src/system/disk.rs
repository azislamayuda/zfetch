use std::process::Command;

pub fn usage() -> String {
    let Ok(output) = Command::new("df")
        .args(["-B1", "--output=used,size", "/"])
        .output()
    else {
        return "Unknown".to_string();
    };

    if !output.status.success() {
        return "Unknown".to_string();
    }

    let text = String::from_utf8_lossy(&output.stdout);

    let Some(data) = text.lines().nth(1) else {
        return "Unknown".to_string();
    };

    let mut values = data.split_whitespace();

    let used = values.next().and_then(|s| s.parse::<u64>().ok());
    let total = values.next().and_then(|s| s.parse::<u64>().ok());

    match (used, total) {
        (Some(used), Some(total)) => {
            let gib = 1024.0 * 1024.0 * 1024.0;

            format!(
                "{:.2} GiB / {:.2} GiB (/)",
                used as f64 / gib,
                total as f64 / gib
            )
        }
        _ => "Unknown".to_string(),
    }
}
