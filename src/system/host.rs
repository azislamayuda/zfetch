use super::util::read_dmi;

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
