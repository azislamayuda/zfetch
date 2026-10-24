use std::{env, fs};

fn cpuinfo_name(content: &str) -> Option<String> {
    const KEYS: [&str; 4] = ["model name", "Hardware", "Processor", "cpu model"];
    for key in KEYS {
        for line in content.lines() {
            let Some((field, value)) = line.split_once(':') else {
                continue;
            };
            if field.trim().eq_ignore_ascii_case(key) && !value.trim().is_empty() {
                let text = value.trim();
                // Some ARM kernels put a numeric processor index in 'Processor'.
                if key == "Processor" && text.chars().all(|c| c.is_ascii_digit()) {
                    continue;
                }
                return Some(text.to_owned());
            }
        }
    }
    None
}

fn device_tree_model(raw: &[u8]) -> Option<String> {
    let value = String::from_utf8_lossy(raw);
    let value = value.trim_matches(|ch: char| ch == '\0' || ch.is_whitespace());
    (!value.is_empty()).then(|| value.to_owned())
}

fn detect(cpuinfo: Option<&str>, device_tree: Option<&[u8]>, architecture: &str) -> String {
    // Prefer a concrete processor model. Device-tree model often describes the
    // board rather than CPU; use it only when cpuinfo has no usable CPU name.
    cpuinfo
        .and_then(cpuinfo_name)
        .or_else(|| device_tree.and_then(device_tree_model))
        .unwrap_or_else(|| format!("Unknown CPU ({architecture})"))
}

pub fn name() -> String {
    let cpuinfo = fs::read_to_string("/proc/cpuinfo").ok();
    // Device tree is common on ARM boards; do not read it when cpuinfo is usable.
    let dt = if cpuinfo.as_deref().and_then(cpuinfo_name).is_none() {
        fs::read("/proc/device-tree/model")
            .ok()
            .or_else(|| fs::read("/sys/firmware/devicetree/base/model").ok())
    } else {
        None
    };
    detect(cpuinfo.as_deref(), dt.as_deref(), env::consts::ARCH)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intel_cpuinfo_preserved() {
        let text = "processor : 0\nmodel name : 11th Gen Intel(R) Core(TM) i5-1145G7 @ 2.60GHz\n";
        assert_eq!(
            cpuinfo_name(text).as_deref(),
            Some("11th Gen Intel(R) Core(TM) i5-1145G7 @ 2.60GHz")
        );
    }

    #[test]
    fn arm_processor_and_hardware() {
        let text = "processor : 0\nProcessor : AArch64 Processor rev 1\nHardware : Raspberry Pi\n";
        assert_eq!(cpuinfo_name(text).as_deref(), Some("Raspberry Pi"));
    }

    #[test]
    fn device_tree_fallback_and_unknown() {
        assert_eq!(
            detect(
                Some("processor : 0"),
                Some(b"Raspberry Pi 5 Model B\0"),
                "aarch64"
            ),
            "Raspberry Pi 5 Model B"
        );
        assert_eq!(detect(None, None, "aarch64"), "Unknown CPU (aarch64)");
    }

    #[test]
    fn invalid_device_tree_is_ignored() {
        assert_eq!(device_tree_model(b"\0\0"), None);
    }
}
