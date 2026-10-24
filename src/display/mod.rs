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

fn render_line(line: &Line, theme: &Theme) -> String {
    match line {
        Line::Title(title) => theme.paint(title, Color::Title),

        Line::Subtitle(value) => theme.paint(value, Color::Muted),

        Line::Heading(title) => {
            let text = format!("{title} {}", "─".repeat(28));

            theme.paint(&text, Color::Section)
        }

        Line::Item {
            label,
            value,
            percentage,
            battery,
        } => {
            let formatted_label = format!("{label:<11}");

            let colored_label = theme.paint(&formatted_label, Color::Label);

            let value_color = match percentage {
                Some(percent) if *battery => {
                    if *percent < 20 {
                        Color::Danger
                    } else if *percent < 50 {
                        Color::Warning
                    } else {
                        Color::Good
                    }
                }

                Some(percent) => {
                    if *percent >= 90 {
                        Color::Danger
                    } else if *percent >= 75 {
                        Color::Warning
                    } else {
                        Color::Good
                    }
                }

                None => Color::Value,
            };

            let colored_value = theme.paint(value, value_color);

            format!("{colored_label} {colored_value}")
        }

        Line::Spacer => String::new(),
    }
}

/// Compact vertical list with no ASCII logo, decorative sections or blank lines.
/// Reuses the classic renderer's existing theme and module formatting.
pub fn render_compact(info: &SystemInfo, options: &Options, config: &Config) {
    let theme = Theme::auto(options.no_color, &config.theme);
    println!(
        "{}",
        theme.paint(
            &format!("zfetch v{}", env!("CARGO_PKG_VERSION")),
            Color::Title
        )
    );
    for line in layout::build(info, config) {
        if let Line::Item { .. } = &line {
            println!("{}", render_line(&line, &theme));
        }
    }
}

pub fn render(info: &SystemInfo, options: &Options, config: &Config) {
    let theme = Theme::auto(options.no_color, &config.theme);

    let configured_logo = config.logo_path();

    let logo_path = options.logo.as_deref().or(configured_logo.as_deref());

    let no_logo = options.no_logo || (!config.logo.enabled && options.logo.is_none());

    let logo_lines = logo::load(logo_path, no_logo);
    let info_lines = layout::build(info, config);

    let logo_width = logo_lines
        .iter()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(0);

    let logo_height = logo_lines.len();
    let info_height = info_lines.len();

    let total_lines = logo_height.max(info_height);

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
