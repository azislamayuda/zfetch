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

#[derive(Default)]
pub struct Options {
    pub no_color: bool,
    pub no_logo: bool,
    pub logo: Option<PathBuf>,
    pub logo_align: Option<LogoAlign>,
    pub config: Option<PathBuf>,
}

pub enum Action {
    Run(Options),
    Help,
    Version,
    CheckUpdate,
    Update,
}

pub fn parse() -> Result<Action, String> {
    let mut options = Options::default();
    let mut args = env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => return Ok(Action::Help),
            "-V" | "--version" => return Ok(Action::Version),
            "--check-update" => {
                if args.next().is_some() {
                    return Err("--check-update does not accept additional options".into());
                }
                return Ok(Action::CheckUpdate);
            }
            "--update" => {
                if args.next().is_some() {
                    return Err("--update does not accept additional options".into());
                }
                return Ok(Action::Update);
            }
            "--no-color" => options.no_color = true,
            "--no-logo" => options.no_logo = true,
            "--logo" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--logo requires a file path".to_string())?;
                let path = PathBuf::from(value);
                if !path.is_file() {
                    return Err(format!("Logo file not found: {}", path.display()));
                }
                options.logo = Some(path);
            }
            "--logo-align" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--logo-align requires top, center or bottom".to_string())?;
                options.logo_align = Some(match value.as_str() {
                    "top" => LogoAlign::Top,
                    "center" => LogoAlign::Center,
                    "bottom" => LogoAlign::Bottom,
                    _ => return Err(format!("Invalid logo alignment: {value}")),
                });
            }
            "--config" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--config requires a file path".to_string())?;
                options.config = Some(PathBuf::from(value));
            }
            _ => return Err(format!("Unknown argument: {arg}")),
        }
    }

    if options.no_logo && options.logo.is_some() {
        return Err("--logo and --no-logo cannot be used together".into());
    }

    Ok(Action::Run(options))
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
    println!("      --config <FILE>     Use custom config file");
    println!("      --check-update      Check latest stable GitHub Release");
    println!("      --update            Check and offer safe .deb update");
}
