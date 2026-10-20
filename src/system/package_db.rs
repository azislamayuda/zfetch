use std::{
    fs::File,
    io::{self, BufRead, BufReader},
};

// ============================================
// PACKAGE VERSION FORMAT
// ============================================

fn upstream_version(version: &str) -> String {
    let without_epoch = version.rsplit(':').next().unwrap_or(version);

    let upstream = without_epoch
        .rsplit_once('-')
        .map_or(without_epoch, |(value, _)| value);

    upstream
        .split(['+', '~'])
        .next()
        .unwrap_or(upstream)
        .to_string()
}

// ============================================
// REUSABLE BUFFER DPKG PARSER
// ============================================

fn find_version<R: BufRead>(mut reader: R, package: &str) -> io::Result<Option<String>> {
    let mut line = Vec::with_capacity(256);

    let mut matched = false;
    let mut installed = false;
    let mut version: Option<String> = None;

    loop {
        line.clear();

        let bytes_read = reader.read_until(b'\n', &mut line)?;

        if bytes_read == 0 {
            break;
        }

        // Handle LF and CRLF line endings.
        let bytes = line.strip_suffix(b"\n").unwrap_or(&line);
        let bytes = bytes.strip_suffix(b"\r").unwrap_or(bytes);

        // End of a DPKG package record.
        if bytes.is_empty() {
            if matched && installed {
                return Ok(version);
            }

            matched = false;
            installed = false;
            version = None;

            continue;
        }

        if let Some(name) = bytes.strip_prefix(b"Package: ") {
            matched = name == package.as_bytes();
        } else if matched {
            if bytes == b"Status: install ok installed" {
                installed = true;
            }

            if let Some(raw) = bytes.strip_prefix(b"Version: ")
                && let Ok(value) = std::str::from_utf8(raw)
            {
                version = Some(upstream_version(value));
            }
        }
    }

    // The last DPKG record might not end
    // with an empty line.
    if matched && installed {
        Ok(version)
    } else {
        Ok(None)
    }
}

// ============================================
// PUBLIC PACKAGE VERSION LOOKUP
// ============================================

pub fn installed_version(package: &str) -> Option<String> {
    let file = File::open("/var/lib/dpkg/status").ok()?;

    let reader = BufReader::new(file);

    find_version(reader, package).ok().flatten()
}

// ============================================
// UNIT TESTS
// ============================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ubuntu_package_version() {
        assert_eq!(upstream_version("50.1-1ubuntu2"), "50.1");

        assert_eq!(upstream_version("1:50.1-1ubuntu2"), "50.1");
    }

    #[test]
    fn finds_installed_ptyxis() {
        let data = b"\
Package: example
Status: install ok installed
Version: 1.0-1

Package: ptyxis
Status: install ok installed
Version: 50.1-1ubuntu2

";

        let version = find_version(&data[..], "ptyxis").unwrap();

        assert_eq!(version.as_deref(), Some("50.1"));
    }

    #[test]
    fn ignores_uninstalled_package() {
        let data = b"\
Package: ptyxis
Status: deinstall ok config-files
Version: 50.1-1ubuntu2

";

        assert_eq!(find_version(&data[..], "ptyxis").unwrap(), None);
    }

    #[test]
    fn handles_last_record_without_blank_line() {
        let data = b"\
Package: ptyxis
Status: install ok installed
Version: 50.1-1ubuntu2";

        assert_eq!(
            find_version(&data[..], "ptyxis").unwrap(),
            Some("50.1".to_string())
        );
    }
}
