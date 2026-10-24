mod advanced;
mod cli;
mod config;
mod display;
mod hardware_ext;
mod report;
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
        cli::Action::Run(mut options) => {
            // Detailed layout explicitly opts into slower sysfs hardware queries.
            if options.layout == cli::LayoutMode::Detailed {
                options.hardware_details = true;
            }
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
            let telemetry = hardware_ext::enrich(&mut info, &config, &options);
            // Redact before rendering/serialization, never as a terminal-only filter.
            let sensitive = options.privacy.then(|| report::protect_info(&mut info));
            if options.format == cli::OutputFormat::Json {
                let mut data = advanced::json_document(&info, &config);
                hardware_ext::augment_json(&mut data, &telemetry);
                if let Some(ref sensitive) = sensitive {
                    report::protect_json(&mut data, sensitive);
                }
                match serde_json::to_string_pretty(&data) {
                    Ok(json) => {
                        if let Err(error) = report::emit(&json, options.export.as_deref()) {
                            eprintln!("zfetch export: {error}");
                            return ExitCode::from(1);
                        }
                    }
                    Err(error) => {
                        eprintln!("zfetch JSON: {error}");
                        return ExitCode::from(1);
                    }
                }
            } else if let Some(path) = options.export.as_deref() {
                // Plain text, no ANSI escapes or external ASCII logo in saved reports.
                let text = display::plain_report(&info, &config);
                if let Err(error) = report::emit(&text, Some(path)) {
                    eprintln!("zfetch export: {error}");
                    return ExitCode::from(1);
                }
            } else {
                if options.layout == cli::LayoutMode::Compact {
                    display::render_compact(&info, &options, &config);
                } else {
                    display::render(&info, &options, &config);
                }
            }
        }
    }
    ExitCode::SUCCESS
}
