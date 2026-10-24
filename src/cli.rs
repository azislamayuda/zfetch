use serde::Deserialize;
use std::{env, path::PathBuf};

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogoAlign {
    Top,
    #[default]
    Center,
    Bottom,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum OutputFormat {
    #[default]
    Terminal,
    Json,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LayoutMode {
    Compact,
    #[default]
    Classic,
    Detailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Preset {
    Minimal,
    Default,
    Full,
}

#[derive(Default)]
pub struct Options {
    pub no_color: bool,
    pub no_logo: bool,
    pub logo: Option<PathBuf>,
    pub logo_align: Option<LogoAlign>,
    pub config: Option<PathBuf>,
    pub format: OutputFormat,
    pub only: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
    pub preset: Option<Preset>,
    pub layout: LayoutMode,
    pub theme: Option<String>,
    pub hardware_details: bool,
}

pub enum Action {
    Run(Options),
    Help,
    Version,
    CheckUpdate,
    Update,
    ListModules,
    ListThemes,
    InitConfig,
    ValidateConfig(Option<PathBuf>),
}

fn csv_modules(raw: &str, option: &str) -> Result<Vec<String>, String> {
    let values: Vec<String> = raw.split(',').map(|v| v.trim().to_lowercase()).collect();
    if values.is_empty() || values.iter().any(String::is_empty) {
        return Err(format!(
            "{option} needs a comma-separated list of module names"
        ));
    }
    let mut seen = std::collections::HashSet::new();
    for value in &values {
        if !seen.insert(value) {
            return Err(format!("Duplicate module '{value}' in {option}"));
        }
    }
    Ok(values)
}

fn parse_args<I: Iterator<Item = String>>(mut args: I) -> Result<Action, String> {
    let mut options = Options::default();
    let mut special: Option<&'static str> = None;
    let mut special_count = 0usize;
    let mut ordinary_options = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => return Ok(Action::Help),
            "-V" | "--version" => return Ok(Action::Version),
            "--check-update" => {
                special = Some("check-update");
                special_count += 1;
            }
            "--update" => {
                special = Some("update");
                special_count += 1;
            }
            "--list-modules" => {
                special = Some("list-modules");
                special_count += 1;
            }
            "--list-themes" => {
                special = Some("list-themes");
                special_count += 1;
            }
            "--init-config" => {
                special = Some("init-config");
                special_count += 1;
            }
            "--validate-config" => {
                special = Some("validate-config");
                special_count += 1;
            }
            "--no-color" => {
                options.no_color = true;
                ordinary_options = true;
            }
            "--no-logo" => {
                options.no_logo = true;
                ordinary_options = true;
            }
            "--hardware-details" => {
                options.hardware_details = true;
                ordinary_options = true;
            }
            "--logo" => {
                let value = args.next().ok_or("--logo requires a file path")?;
                let path = PathBuf::from(value);
                if !path.is_file() {
                    return Err(format!("Logo file not found: {}", path.display()));
                }
                options.logo = Some(path);
                ordinary_options = true;
            }
            "--logo-align" => {
                let value = args
                    .next()
                    .ok_or("--logo-align requires top, center or bottom")?;
                options.logo_align = Some(match value.as_str() {
                    "top" => LogoAlign::Top,
                    "center" => LogoAlign::Center,
                    "bottom" => LogoAlign::Bottom,
                    _ => return Err(format!("Invalid logo alignment: {value}")),
                });
                ordinary_options = true;
            }
            "--config" => {
                options.config = Some(PathBuf::from(
                    args.next().ok_or("--config requires a file path")?,
                ));
            }
            "--format" => {
                let value = args.next().ok_or("--format requires terminal or json")?;
                options.format = match value.as_str() {
                    "terminal" => OutputFormat::Terminal,
                    "json" => OutputFormat::Json,
                    _ => return Err(format!("Unsupported format: {value}")),
                };
                ordinary_options = true;
            }
            "--only" => {
                options.only = Some(csv_modules(
                    &args.next().ok_or("--only requires a list")?,
                    "--only",
                )?);
                ordinary_options = true;
            }
            "--exclude" => {
                options.exclude = Some(csv_modules(
                    &args.next().ok_or("--exclude requires a list")?,
                    "--exclude",
                )?);
                ordinary_options = true;
            }
            "--layout" => {
                let value = args
                    .next()
                    .ok_or("--layout requires compact, classic or detailed")?;
                options.layout = match value.as_str() {
                    "compact" => LayoutMode::Compact,
                    "classic" => LayoutMode::Classic,
                    "detailed" => LayoutMode::Detailed,
                    _ => return Err(format!("Unknown layout: {value}")),
                };
                ordinary_options = true;
            }
            "--preset" => {
                let value = args
                    .next()
                    .ok_or("--preset requires minimal, default or full")?;
                options.preset = Some(match value.as_str() {
                    "minimal" => Preset::Minimal,
                    "default" => Preset::Default,
                    "full" => Preset::Full,
                    _ => return Err(format!("Unknown preset: {value}")),
                });
                ordinary_options = true;
            }
            "--theme" => {
                options.theme = Some(args.next().ok_or("--theme requires a theme name")?);
                ordinary_options = true;
            }
            _ => return Err(format!("Unknown argument: {arg}")),
        }
    }

    if options.format == OutputFormat::Json && options.layout != LayoutMode::Classic {
        return Err(
            "--layout compact/detailed is terminal-only; use --hardware-details with JSON".into(),
        );
    }
    if options.no_logo && options.logo.is_some() {
        return Err("--logo and --no-logo cannot be used together".into());
    }
    if options.only.is_some() && options.exclude.is_some() {
        return Err("--only and --exclude cannot be used together".into());
    }
    if special_count > 1 {
        return Err("Only one action flag can be used at a time".into());
    }
    if let Some(name) = special {
        // --config is explicitly allowed for --validate-config only.
        if ordinary_options || (options.config.is_some() && name != "validate-config") {
            return Err(format!("--{name} cannot be combined with display options"));
        }
        return Ok(match name {
            "check-update" => Action::CheckUpdate,
            "update" => Action::Update,
            "list-modules" => Action::ListModules,
            "list-themes" => Action::ListThemes,
            "init-config" => Action::InitConfig,
            "validate-config" => Action::ValidateConfig(options.config),
            _ => unreachable!(),
        });
    }
    Ok(Action::Run(options))
}

pub fn parse() -> Result<Action, String> {
    parse_args(env::args().skip(1))
}

pub fn print_help() {
    println!("zfetch v{}", env!("CARGO_PKG_VERSION"));
    println!("A fast Linux system information tool written in Rust.");
    println!();
    println!("Usage: zfetch [OPTIONS]");
    println!();
    println!("Options:");
    println!("  -h, --help              Show help");
    println!("  -V, --version           Show version");
    println!("      --no-color          Disable ANSI colors");
    println!("      --logo <FILE>       Use custom ASCII logo");
    println!("      --no-logo           Hide ASCII logo");
    println!("      --logo-align <POS>  top, center or bottom");
    println!("      --config <FILE>     Use custom JSONC config");
    println!("      --format <FMT>      terminal (default) or json");
    println!("      --only <MODULES>    Detect only listed modules");
    println!("      --exclude <MODULES> Skip listed modules");
    println!("      --list-modules      Show supported module names");
    println!("      --preset <NAME>     minimal, default or full (module selection)");
    println!("      --layout <NAME>     compact, classic (default), detailed");
    println!("      --theme <NAME>      Select a built-in ANSI theme");
    println!("      --list-themes       Show available themes");
    println!("      --hardware-details Append CPU core/thread details");
    println!("      --init-config       Create an example config without overwrite");
    println!("      --validate-config   Validate JSONC configuration");
    println!("      --check-update      Check latest stable GitHub Release");
    println!("      --update            Check and offer safe .deb update");
}

#[cfg(test)]
mod tests {
    use super::*;
    fn parse(input: &[&str]) -> Result<Action, String> {
        parse_args(input.iter().map(|s| s.to_string()))
    }
    #[test]
    fn sprint03_layout_parsing() {
        for (name, expected) in [
            ("compact", LayoutMode::Compact),
            ("classic", LayoutMode::Classic),
            ("detailed", LayoutMode::Detailed),
        ] {
            match parse(&["--layout", name]).unwrap() {
                Action::Run(options) => assert_eq!(options.layout, expected),
                _ => panic!("expected Run"),
            }
        }
        assert!(parse(&["--layout", "unknown"]).is_err());
        assert!(parse(&["--layout"]).is_err());
    }

    #[test]
    fn sprint03_json_layout_rejects_display_modes() {
        assert!(parse(&["--format", "json", "--layout", "compact"]).is_err());
        assert!(parse(&["--format", "json", "--layout", "detailed"]).is_err());
        assert!(parse(&["--format", "json", "--layout", "classic"]).is_ok());
        assert!(parse(&["--format", "json", "--hardware-details"]).is_ok());
    }

    #[test]
    fn rejects_empty_modules() {
        assert!(parse(&["--only", "os,,cpu"]).is_err());
    }
    #[test]
    fn rejects_conflicting_filters() {
        assert!(parse(&["--only", "os", "--exclude", "gpu"]).is_err());
    }
    #[test]
    fn parses_json_and_only() {
        match parse(&["--format", "json", "--only", "os,cpu"]).unwrap() {
            Action::Run(o) => {
                assert_eq!(o.format, OutputFormat::Json);
                assert_eq!(o.only.unwrap(), vec!["os", "cpu"]);
            }
            _ => panic!("expected Run"),
        }
    }
    #[test]
    fn rejects_multiple_action_flags() {
        assert!(parse(&["--check-update", "--list-modules"]).is_err());
    }
    #[test]
    fn validates_config_action_accepts_path() {
        assert!(matches!(
            parse(&["--validate-config", "--config", "/tmp/example"]),
            Ok(Action::ValidateConfig(Some(_)))
        ));
    }
}
