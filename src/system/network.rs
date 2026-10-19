use std::process::Command;

pub fn local_ip() -> String {
    let Ok(output) = Command::new("ip")
        .args(["-o", "-4", "addr", "show", "scope", "global"])
        .output()
    else {
        return "Unknown".to_string();
    };

    if !output.status.success() {
        return "Unknown".to_string();
    }

    let text = String::from_utf8_lossy(&output.stdout);

    for line in text.lines() {
        let mut fields = line.split_whitespace();

        let _index = fields.next();
        let interface = fields.next();
        let kind = fields.next();
        let address = fields.next();

        if let (Some(interface), Some("inet"), Some(address)) = (interface, kind, address) {
            return format!("{address} ({interface})");
        }
    }

    "Unknown".to_string()
}
