use super::util::{Usage, meminfo};

/// `total == 0` means swap is disabled.
pub fn usage(meminfo_text: &str) -> Option<Usage> {
    let total = meminfo(meminfo_text, "SwapTotal")?;
    let free = meminfo(meminfo_text, "SwapFree")?;
    Some(Usage {
        used: total.saturating_sub(free),
        total,
    })
}
