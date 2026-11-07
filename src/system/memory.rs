use super::util::{Usage, meminfo};

pub fn usage(meminfo_text: &str) -> Option<Usage> {
    let total = meminfo(meminfo_text, "MemTotal")?;
    let available = meminfo(meminfo_text, "MemAvailable")?;
    Some(Usage {
        used: total.saturating_sub(available),
        total,
    })
}
