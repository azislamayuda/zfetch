use crate::config::ThemeConfig;

use std::{
    env,
    io::{self, IsTerminal},
};

pub enum Color {
    Logo,
    Title,
    Section,
    Label,
    Muted,
    Good,
    Warning,
    Danger,
}

pub struct Theme {
    enabled: bool,
    logo_code: &'static str,
    heading_code: &'static str,
}

impl Theme {
    pub fn auto(no_color: bool, config: &ThemeConfig) -> Self {
        let terminal = io::stdout().is_terminal();
        let disabled_by_env = env::var_os("NO_COLOR").is_some();
        let dumb = env::var("TERM").is_ok_and(|term| term == "dumb");

        Self {
            enabled: terminal && config.color && !no_color && !disabled_by_env && !dumb,

            logo_code: config.logo_color.ansi(),
            heading_code: config.heading_color.ansi(),
        }
    }

    pub fn paint(&self, text: &str, color: Color) -> String {
        if !self.enabled {
            return text.to_string();
        }

        let code = match color {
            Color::Logo => self.logo_code,
            Color::Title | Color::Section => self.heading_code,
            Color::Label => "1;37",
            Color::Muted => "90",
            Color::Good => "32",
            Color::Warning => "33",
            Color::Danger => "31",
        };

        format!("\x1b[{code}m{text}\x1b[0m")
    }
}
