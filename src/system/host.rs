use std::fs;

fn read_dmi(field: &str) -> Option<String> {
    fs::read_to_string(format!("/sys/class/dmi/id/{field}"))
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

pub fn name() -> String {
    let vendor = read_dmi("sys_vendor").unwrap_or_default();
    let product = read_dmi("product_name").unwrap_or_default();

    let host = format!("{vendor} {product}");
    let host = host.trim();

    if host.is_empty() {
        "Unknown".to_string()
    } else {
        host.to_string()
    }
}
