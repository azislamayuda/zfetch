use super::util::read_dmi;

pub fn info() -> String {
    let mut result = [read_dmi("bios_vendor"), read_dmi("bios_version")]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(" ");

    if let Some(date) = read_dmi("bios_date") {
        if !result.is_empty() {
            result.push(' ');
        }
        result.push('(');
        result.push_str(&date);
        result.push(')');
    }

    if result.is_empty() {
        "Unknown".to_string()
    } else {
        result
    }
}
