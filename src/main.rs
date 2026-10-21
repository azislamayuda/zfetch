mod cli;
mod config;
mod display;
mod system;
mod update;

use std::process::ExitCode;

fn main() -> ExitCode {
    let action = match cli::parse() {
        Ok(action) => action,
        Err(error) => {
            eprintln!("zfetch: {error}");
            eprintln!("Try 'zfetch --help' for usage.");
            return ExitCode::from(2);
        }
    };

    match action {
        cli::Action::Help => cli::print_help(),
        cli::Action::Version => println!("zfetch {}", env!("CARGO_PKG_VERSION")),
        cli::Action::CheckUpdate => {
            if let Err(error) = update::run(true) {
                eprintln!("zfetch update: {error}");
                return ExitCode::from(1);
            }
        }
        cli::Action::Update => {
            if let Err(error) = update::run(false) {
                eprintln!("zfetch update: {error}");
                return ExitCode::from(1);
            }
        }
        cli::Action::Run(options) => {
            let config = match config::Config::load(options.config.as_deref()) {
                Ok(config) => config,
                Err(error) => {
                    eprintln!("zfetch: {error}");
                    return ExitCode::from(2);
                }
            };
            let info = system::collect(&config);
            display::render(&info, &options, &config);
        }
    }
    ExitCode::SUCCESS
}
