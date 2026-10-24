mod advanced;
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
        cli::Action::ListModules => advanced::list_modules(),
        cli::Action::ListThemes => advanced::list_themes(),
        cli::Action::InitConfig => {
            if let Err(error) = advanced::init_config() {
                eprintln!("zfetch: {error}");
                return ExitCode::from(2);
            }
        }
        cli::Action::ValidateConfig(path) => {
            if let Err(error) = advanced::validate_config(path.as_deref()) {
                eprintln!("zfetch: {error}");
                return ExitCode::from(2);
            }
        }
        cli::Action::Run(options) => {
            let mut config = match config::Config::load(options.config.as_deref()) {
                Ok(config) => config,
                Err(error) => {
                    eprintln!("zfetch: {error}");
                    return ExitCode::from(2);
                }
            };
            if let Err(error) = advanced::configure(&mut config, &options) {
                eprintln!("zfetch: {error}");
                return ExitCode::from(2);
            }
            let mut info = system::collect(&config);
            advanced::enrich_hardware(&mut info, &config, &options);
            if options.format == cli::OutputFormat::Json {
                let data = advanced::json_document(&info, &config);
                match serde_json::to_string_pretty(&data) {
                    Ok(json) => println!("{json}"),
                    Err(error) => {
                        eprintln!("zfetch JSON: {error}");
                        return ExitCode::from(1);
                    }
                }
            } else {
                display::render(&info, &options, &config);
            }
        }
    }
    ExitCode::SUCCESS
}
