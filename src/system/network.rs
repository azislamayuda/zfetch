use std::{ffi::CStr, fs, net::Ipv4Addr, process::Command, ptr};

// ============================================
// DEFAULT IPv4 ROUTE
// ============================================

fn parse_default_route(content: &str) -> Option<String> {
    let mut selected: Option<(&str, u32)> = None;

    for line in content.lines().skip(1) {
        let mut fields = line.split_whitespace();

        let Some(interface) = fields.next() else {
            continue;
        };

        let Some(destination) = fields.next() else {
            continue;
        };

        let _gateway = fields.next();
        let flags = fields.next();
        let _refcnt = fields.next();
        let _use = fields.next();
        let metric = fields.next();
        let mask = fields.next();

        let (Some(flags), Some(metric), Some(mask)) = (flags, metric, mask) else {
            continue;
        };

        if destination != "00000000" || mask != "00000000" {
            continue;
        }

        let Ok(flags) = u32::from_str_radix(flags, 16) else {
            continue;
        };

        // RTF_UP: route is enabled.
        if flags & 0x1 == 0 {
            continue;
        }

        let Ok(metric) = metric.parse::<u32>() else {
            continue;
        };

        if selected
            .as_ref()
            .is_none_or(|(_, best_metric)| metric < *best_metric)
        {
            selected = Some((interface, metric));
        }
    }

    selected.map(|(interface, _)| interface.to_string())
}

fn default_route_interface() -> Option<String> {
    let content = fs::read_to_string("/proc/net/route").ok()?;
    parse_default_route(&content)
}

// ============================================
// IPv4 NETMASK / CIDR
// ============================================

fn netmask_prefix(mask: Ipv4Addr) -> Option<u32> {
    let bits = u32::from(mask);
    let prefix = bits.count_ones();

    let expected = if prefix == 0 {
        0
    } else {
        u32::MAX << (32 - prefix)
    };

    (bits == expected).then_some(prefix)
}

// ============================================
// INTERFACE FILTERS
// ============================================

fn is_virtual_interface(name: &str) -> bool {
    name == "docker0"
        || name.starts_with("br-")
        || name.starts_with("veth")
        || name.starts_with("virbr")
}

// ============================================
// GETIFADDRS MEMORY GUARD
// ============================================

struct InterfaceAddresses(*mut libc::ifaddrs);

impl Drop for InterfaceAddresses {
    fn drop(&mut self) {
        if !self.0.is_null() {
            // SAFETY:
            // The pointer was allocated by getifaddrs()
            // and is released exactly once here.
            unsafe {
                libc::freeifaddrs(self.0);
            }
        }
    }
}

// ============================================
// NATIVE IPv4 DETECTION
// ============================================

fn native_local_ip() -> Option<String> {
    let preferred = default_route_interface();

    let mut head: *mut libc::ifaddrs = ptr::null_mut();

    // SAFETY:
    // getifaddrs() receives a valid output pointer.
    let result = unsafe { libc::getifaddrs(&mut head) };

    if result != 0 {
        return None;
    }

    let addresses = InterfaceAddresses(head);

    let mut current = addresses.0;

    // Keep the first usable address as fallback,
    // preferring physical over Docker interfaces.
    let mut fallback: Option<(bool, String)> = None;

    while !current.is_null() {
        // SAFETY:
        // Current is a node owned by InterfaceAddresses.
        let entry = unsafe { &*current };

        current = entry.ifa_next;

        if entry.ifa_addr.is_null() || entry.ifa_netmask.is_null() || entry.ifa_name.is_null() {
            continue;
        }

        // Only use active, non-loopback interfaces.
        if entry.ifa_flags & (libc::IFF_UP as u32) == 0 {
            continue;
        }

        if entry.ifa_flags & (libc::IFF_LOOPBACK as u32) != 0 {
            continue;
        }

        // SAFETY:
        // ifa_addr points to a valid sockaddr structure.
        let family = unsafe { (*entry.ifa_addr).sa_family };

        if i32::from(family) != libc::AF_INET {
            continue;
        }

        // SAFETY:
        // An AF_INET sockaddr is a sockaddr_in.
        let ipv4 = unsafe { &*(entry.ifa_addr as *const libc::sockaddr_in) };

        let netmask = unsafe { &*(entry.ifa_netmask as *const libc::sockaddr_in) };

        let address = Ipv4Addr::from(ipv4.sin_addr.s_addr.to_ne_bytes());

        if address.is_loopback() || address.is_link_local() || address.is_unspecified() {
            continue;
        }

        let mask = Ipv4Addr::from(netmask.sin_addr.s_addr.to_ne_bytes());

        let Some(prefix) = netmask_prefix(mask) else {
            continue;
        };

        // SAFETY:
        // getifaddrs guarantees a NUL-terminated
        // interface name for each returned node.
        let name = unsafe { CStr::from_ptr(entry.ifa_name) };

        let Ok(name) = name.to_str() else {
            continue;
        };

        let formatted = format!("{address}/{prefix} ({name})");

        // Preferred path: interface from default route.
        if preferred.as_deref() == Some(name) {
            return Some(formatted);
        }

        let physical = !is_virtual_interface(name);

        let replace = fallback
            .as_ref()
            .is_none_or(|(was_physical, _)| physical && !was_physical);

        if replace {
            fallback = Some((physical, formatted));
        }
    }

    fallback.map(|(_, value)| value)
}

// ============================================
// FALLBACK: IPROUTE2
// ============================================

fn ip_command_fallback() -> Option<String> {
    let output = Command::new("ip")
        .args(["-o", "-4", "addr", "show", "scope", "global"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8_lossy(&output.stdout);

    for line in text.lines() {
        let mut fields = line.split_whitespace();

        let _index = fields.next();
        let interface = fields.next();
        let kind = fields.next();
        let address = fields.next();

        if let (Some(interface), Some("inet"), Some(address)) = (interface, kind, address) {
            return Some(format!("{address} ({interface})"));
        }
    }

    None
}

// ============================================
// PUBLIC API
// ============================================

pub fn local_ip() -> String {
    native_local_ip()
        .or_else(ip_command_fallback)
        .unwrap_or_else(|| "Unknown".to_string())
}

// ============================================
// TESTS
// ============================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_default_route() {
        let routes = concat!(
            "Iface Destination Gateway Flags RefCnt Use Metric Mask MTU Window IRTT\n",
            "docker0 00000000 00000000 0001 0 0 1000 00000000 0 0 0\n",
            "wlp0s20f3 00000000 0101A8C0 0003 0 0 600 00000000 0 0 0\n",
        );

        assert_eq!(parse_default_route(routes).as_deref(), Some("wlp0s20f3"));
    }

    #[test]
    fn ignores_non_default_route() {
        let routes = concat!(
            "Iface Destination Gateway Flags RefCnt Use Metric Mask MTU Window IRTT\n",
            "docker0 000011AC 00000000 0001 0 0 0 0000FFFF 0 0 0\n",
        );

        assert_eq!(parse_default_route(routes), None);
    }

    #[test]
    fn ipv4_netmask_prefix() {
        assert_eq!(netmask_prefix(Ipv4Addr::new(255, 255, 255, 0)), Some(24));

        assert_eq!(netmask_prefix(Ipv4Addr::new(255, 255, 0, 0)), Some(16));

        assert_eq!(netmask_prefix(Ipv4Addr::new(255, 0, 255, 0)), None);
    }

    #[test]
    fn docker_interface_filter() {
        assert!(is_virtual_interface("docker0"));
        assert!(is_virtual_interface("br-71e5e4342a5c"));
        assert!(!is_virtual_interface("wlp0s20f3"));
        assert!(!is_virtual_interface("enp2s0"));
    }
}
