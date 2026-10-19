use std::{env, path::Path, process::Command};

pub fn name() -> String {
    let shell = match env::var("SHELL") {
        Ok(value) if !value.is_empty() => value,
        _ => return "Unknown".to_string(),
    };

    let executable = Path::new(&shell)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Unknown");

    if executable == "bash"
        && let Ok(output) = Command::new(&shell).arg("--version").output()
        && output.status.success()
    {
        let text = String::from_utf8_lossy(&output.stdout);

        if let Some(line) = text.lines().next()
            && let Some((_, version)) = line.split_once("version ")
            && let Some(version) = version.split('(').next()
        {
            return format!("bash {}", version.trim());
        }
    }

    executable.to_string()
}
