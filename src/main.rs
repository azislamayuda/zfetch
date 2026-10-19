mod cli;
mod display;
mod system;

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

        cli::Action::Version => {
            println!("zfetch {}", env!("CARGO_PKG_VERSION"));
        }

        cli::Action::Run(options) => {
            let info = system::collect();
            display::render(&info, &options);
        }
    }

    ExitCode::SUCCESS
}
