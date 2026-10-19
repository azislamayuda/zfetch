use std::{env, path::PathBuf};

#[derive(Default)]
pub struct Options {
    pub no_color: bool,
    pub no_logo: bool,
    pub logo: Option<PathBuf>,
}

pub enum Action {
    Run(Options),
    Help,
    Version,
}

pub fn parse() -> Result<Action, String> {
    let mut options = Options::default();
    let mut args = env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => return Ok(Action::Help),

            "-V" | "--version" => return Ok(Action::Version),

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
    println!("  -h, --help         Show this help message");
    println!("  -V, --version      Show application version");
    println!("      --no-color     Disable ANSI colors");
    println!("      --logo <FILE>  Load custom ASCII logo");
    println!("      --no-logo      Hide ASCII logo");
}
