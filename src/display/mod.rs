mod colors;
mod layout;
mod logo;

use crate::{
    cli::{LogoAlign, Options},
    config::Config,
    system::SystemInfo,
};

use colors::{Color, Theme};
use layout::Line;

fn extract_percent(value: &str) -> Option<u8> {
    let before_percent = value.split_once('%')?.0;

    let number = before_percent
        .rsplit(|c: char| !c.is_ascii_digit())
        .next()?;

    number.parse::<u8>().ok()
}

fn resource_color(label: &str, value: &str) -> Option<Color> {
    let is_extra_disk = label.is_empty() && value.contains(" GiB / ") && value.contains('%');

    if !matches!(label, "Memory" | "Swap" | "Disk" | "Battery") && !is_extra_disk {
        return None;
    }

    let percent = extract_percent(value)?;

    if label == "Battery" {
        return Some(if percent < 20 {
            Color::Danger
        } else if percent < 50 {
            Color::Warning
        } else {
            Color::Good
        });
    }

    Some(if percent >= 90 {
        Color::Danger
    } else if percent >= 75 {
        Color::Warning
    } else {
        Color::Good
    })
}

fn render_line(line: &Line, theme: &Theme) -> String {
    match line {
        Line::Title(title) => theme.paint(title, Color::Title),

        Line::Subtitle(value) => theme.paint(value, Color::Muted),

        Line::Heading(title) => {
            let text = format!("{title} {}", "─".repeat(28));
            theme.paint(&text, Color::Section)
        }

        Line::Item(label, value) => {
            let formatted_label = format!("{label:<11}");
            let colored_label = theme.paint(&formatted_label, Color::Label);

            let colored_value = match resource_color(label, value) {
                Some(color) => theme.paint(value, color),
                None => value.to_string(),
            };

            format!("{colored_label} {colored_value}")
        }

        Line::Spacer => String::new(),
    }
}

pub fn render(info: &SystemInfo, options: &Options, config: &Config) {
    let theme = Theme::auto(options.no_color, &config.theme);

    // CLI logo overrides configured logo.
    let configured_logo = config.logo_path();

    let logo_path = options.logo.as_deref().or(configured_logo.as_deref());

    // An explicit CLI logo can override enabled=false.
    let no_logo = options.no_logo || (!config.logo.enabled && options.logo.is_none());

    let logo_lines = logo::load(logo_path, no_logo);
    let info_lines = layout::build(info);

    let logo_width = logo_lines
        .iter()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(0);

    let logo_height = logo_lines.len();
    let info_height = info_lines.len();
    let total_lines = logo_height.max(info_height);

    // CLI alignment overrides configured alignment.
    let alignment = options.logo_align.unwrap_or(config.logo.align);

    let logo_start = match alignment {
        LogoAlign::Top => 0,
        LogoAlign::Center => total_lines.saturating_sub(logo_height) / 2,
        LogoAlign::Bottom => total_lines.saturating_sub(logo_height),
    };

    println!();

    for row in 0..total_lines {
        let logo_index = row.checked_sub(logo_start);

        let raw_logo = logo_index
            .and_then(|index| logo_lines.get(index))
            .map(String::as_str)
            .unwrap_or("");

        let padding = logo_width.saturating_sub(raw_logo.chars().count());

        let left = format!("{raw_logo}{}", " ".repeat(padding));

        let right = info_lines
            .get(row)
            .map(|line| render_line(line, &theme))
            .unwrap_or_default();

        if logo_width > 0 {
            let colored_logo = theme.paint(&left, Color::Logo);

            println!("{colored_logo}   {right}");
        } else {
            println!("{right}");
        }
    }

    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_percentage() {
        assert_eq!(extract_percent("80%"), Some(80));
        assert_eq!(extract_percent("100% (Full)"), Some(100));
        assert_eq!(extract_percent("7.00 GiB / 14.00 GiB (50%)"), Some(50));
        assert_eq!(extract_percent("Unknown"), None);
    }
}
