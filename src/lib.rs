pub mod cli;
pub mod error;
pub mod home;
pub mod registry;
pub mod result;
pub mod state;

pub use error::{GripError, ResultCategory};
pub use result::{CommandOutcome, ResultEnvelopeV1};

pub fn run_process() -> std::process::ExitCode {
    use clap::Parser;
    use std::ffi::OsString;
    use std::io::{self, Write};
    let args: Vec<OsString> = std::env::args_os().collect();
    let json_requested = cli::requests_json(&args);
    let parsed = match cli::Cli::try_parse_from(&args) {
        Ok(value) => value,
        Err(error) => {
            if matches!(
                error.kind(),
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion
            ) {
                let _ = write!(io::stdout().lock(), "{error}");
                return std::process::ExitCode::SUCCESS;
            }
            if json_requested && error.kind() != clap::error::ErrorKind::InvalidValue {
                let outcome = CommandOutcome {
                    category: ResultCategory::InvalidUsage,
                    message: error.to_string().trim().to_owned(),
                    details: serde_json::Map::new(),
                };
                let _ = result::render(outcome, result::OutputMode::Json, &mut io::stdout().lock());
            } else {
                let _ = write!(io::stderr().lock(), "{error}");
            }
            return std::process::ExitCode::from(2);
        }
    };
    let outcome = execute(&parsed);
    let exit = outcome.category.exit_code();
    let _ = result::emit_diagnostic(
        parsed.verbose,
        "command completed",
        &mut io::stderr().lock(),
    );
    if result::render(outcome, parsed.output.into(), &mut io::stdout().lock()).is_err() {
        let _ = writeln!(io::stderr().lock(), "grip: could not write command result");
        return std::process::ExitCode::from(20);
    }
    std::process::ExitCode::from(exit)
}

pub fn execute(cli: &cli::Cli) -> CommandOutcome {
    match cli.command {
        cli::Command::Version => {
            let mut outcome =
                CommandOutcome::success(format!("grip {}", env!("CARGO_PKG_VERSION")));
            outcome
                .details
                .insert("version".into(), env!("CARGO_PKG_VERSION").into());
            outcome
        }
        cli::Command::Validate => match validate_selected_home() {
            Ok((path, state)) => {
                let mut outcome = CommandOutcome::success(format!(
                    "Grip home {} is valid; state is {state}",
                    path.display()
                ));
                outcome
                    .details
                    .insert("grip_home".into(), path.display().to_string().into());
                outcome.details.insert("registry".into(), "valid".into());
                outcome.details.insert("state".into(), state.into());
                outcome
            }
            Err(error) => CommandOutcome::failure(&error),
        },
    }
}

fn validate_selected_home() -> Result<(std::path::PathBuf, &'static str), GripError> {
    use std::fs;
    let selected = home::select(std::env::var_os("GRIP_HOME"), ::home::home_dir())?;
    home::validate(&selected)?;
    let config_path = selected.path().join("config.toml");
    let metadata = fs::symlink_metadata(&config_path)
        .map_err(|e| GripError::InvalidConfiguration(format!("config.toml is unavailable: {e}")))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(GripError::InvalidConfiguration(
            "config.toml must be a regular file".into(),
        ));
    }
    let config = fs::read_to_string(&config_path)
        .map_err(|e| GripError::InvalidConfiguration(format!("config.toml is unreadable: {e}")))?;
    registry::decode(&config)?;
    let state_path = selected.path().join("state/state.json");
    match fs::symlink_metadata(&state_path) {
        Ok(m) => {
            if m.file_type().is_symlink() || !m.is_file() {
                return Err(GripError::CorruptState(
                    "state.json must be a regular file".into(),
                ));
            }
            let text = fs::read_to_string(&state_path)
                .map_err(|e| GripError::CorruptState(format!("state.json is unreadable: {e}")))?;
            state::decode(&text)?;
            Ok((selected.path().to_owned(), "valid"))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            Ok((selected.path().to_owned(), "uninitialized"))
        }
        Err(e) => Err(GripError::CorruptState(format!(
            "state.json is unavailable: {e}"
        ))),
    }
}
