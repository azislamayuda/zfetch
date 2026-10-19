use std::process::Command;

fn parse_mode(line: &str) -> Option<(u32, u32, f64)> {
    let mode = line
        .trim_start_matches(|c: char| !c.is_ascii_digit())
        .split_whitespace()
        .next()?;

    let (resolution, refresh) = mode.split_once('@')?;
    let (width, height) = resolution.split_once('x')?;

    Some((
        width.parse().ok()?,
        height.parse().ok()?,
        refresh.parse().ok()?,
    ))
}

pub fn active() -> Vec<String> {
    let Ok(output) = Command::new("gdctl").arg("show").output() else {
        return Vec::new();
    };

    if !output.status.success() {
        return Vec::new();
    }

    let text = String::from_utf8_lossy(&output.stdout);

    let mut displays = Vec::new();
    let mut connector = String::new();
    let mut expect_mode = false;

    for line in text.lines() {
        if let Some((_, name)) = line.split_once("Monitor ") {
            connector = name
                .split_whitespace()
                .next()
                .unwrap_or("Unknown")
                .to_string();

            expect_mode = false;
            continue;
        }

        if line.contains("Current mode") {
            expect_mode = true;
            continue;
        }

        if expect_mode {
            if let Some((width, height, refresh)) = parse_mode(line) {
                let hz = if (refresh - refresh.round()).abs() < 0.01 {
                    format!("{refresh:.0}")
                } else {
                    format!("{refresh:.2}")
                };

                let built_in = connector.starts_with("eDP") || connector.starts_with("LVDS");

                let label = if built_in { " [Built-in]" } else { "" };

                displays.push(format!("{width}x{height} @ {hz} Hz{label} ({connector})"));
            }

            expect_mode = false;
        }
    }

    displays
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_current_mode() {
        assert_eq!(
            parse_mode("│   └──1920x1080@60.000"),
            Some((1920, 1080, 60.0))
        );
    }
}
