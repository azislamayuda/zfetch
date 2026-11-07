mod advanced;
mod cli;
mod config;
mod display;
mod doctor;
mod hardware_ext;
mod report;
mod system;
mod update;

use std::process::ExitCode;

/// An error message (with its "zfetch...:" prefix) and the process exit code.
struct CliError(u8, String);

/// Usage/config errors exit 2.
fn usage(error: String) -> CliError {
    CliError(2, format!("zfetch: {error}"))
}

/// Runtime failures exit 1.
fn failure(prefix: &str) -> impl Fn(String) -> CliError + '_ {
    move |error| CliError(1, format!("{prefix}: {error}"))
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(CliError(code, message)) => {
            eprintln!("{message}");
            ExitCode::from(code)
        }
    }
}

fn run() -> Result<(), CliError> {
    let action =
        cli::parse().map_err(|error| usage(format!("{error}\nTry 'zfetch --help' for usage.")))?;

    match action {
        cli::Action::Help => println!("{}", cli::HELP),
        cli::Action::Version => println!("zfetch {}", env!("CARGO_PKG_VERSION")),
        cli::Action::CheckUpdate => update::run(true).map_err(failure("zfetch update"))?,
        cli::Action::Update => update::run(false).map_err(failure("zfetch update"))?,
        cli::Action::ListModules => advanced::list_modules(),
        cli::Action::ListThemes => advanced::list_themes(),
        cli::Action::InitConfig => advanced::init_config().map_err(usage)?,
        cli::Action::ValidateConfig(path) => {
            advanced::validate_config(path.as_deref()).map_err(usage)?;
        }
        cli::Action::Doctor { privacy, config } => doctor::run(privacy, config.as_deref()),
        cli::Action::Benchmark(path) => {
            let config = config::Config::load(path.as_deref()).map_err(usage)?;
            system::start_benchmark();
            let start = std::time::Instant::now();
            system::collect(&config, false);
            doctor::print_benchmark(
                &system::benchmark_timings(),
                start.elapsed().as_secs_f64() * 1000.0,
            );
        }
        cli::Action::Run(mut options) => {
            // Detailed layout explicitly opts into slower sysfs hardware queries.
            if options.layout == cli::LayoutMode::Detailed {
                options.hardware_details = true;
            }
            let mut config = config::Config::load(options.config.as_deref()).map_err(usage)?;
            advanced::configure(&mut config, &options).map_err(usage)?;
            let mut info = system::collect(&config, options.hardware_details);
            let telemetry = hardware_ext::enrich(&mut info, &config, &options);
            // Redact before rendering/serialization, never as a terminal-only filter.
            let sensitive = options.privacy.then(|| report::protect_info(&mut info));
            if options.format == cli::OutputFormat::Json {
                let mut data = advanced::json_document(&info, &config);
                hardware_ext::augment_json(&mut data, &telemetry);
                if let Some(ref sensitive) = sensitive {
                    report::protect_json(&mut data, sensitive);
                }
                let json = serde_json::to_string_pretty(&data)
                    .map_err(|error| failure("zfetch JSON")(error.to_string()))?;
                report::emit(&json, options.export.as_deref()).map_err(failure("zfetch export"))?;
            } else if let Some(path) = options.export.as_deref() {
                // Plain text, no ANSI escapes or external ASCII logo in saved reports.
                let text = display::plain_report(&info, &config);
                report::emit(&text, Some(path)).map_err(failure("zfetch export"))?;
            } else if options.layout == cli::LayoutMode::Compact {
                display::render_compact(&info, &options, &config);
            } else {
                display::render(&info, &options, &config);
            }
        }
    }
    Ok(())
}
