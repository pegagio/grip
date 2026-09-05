pub mod cli;
pub mod discovery;
pub mod error;
pub mod home;
pub mod mapping;
pub mod path_policy;
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
    match &cli.command {
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
        cli::Command::Mapping(args) => match execute_mapping(&args.command) {
            Ok(outcome) => outcome,
            Err(error) => CommandOutcome::failure(&error),
        },
    }
}

fn selected_home() -> Result<home::GripHome, GripError> {
    let selected = home::select(std::env::var_os("GRIP_HOME"), ::home::home_dir())?;
    home::validate(&selected)?;
    Ok(selected)
}

fn execute_mapping(command: &cli::MappingCommand) -> Result<CommandOutcome, GripError> {
    use cli::{MappingAddKind, MappingCommand};
    use mapping::{Mapping, MappingKind};
    use registry::publication;

    let home = selected_home()?;
    match command {
        MappingCommand::Add(add) => {
            let (kind, pair) = match &add.kind {
                MappingAddKind::File(pair) => (MappingKind::File, pair),
                MappingAddKind::Tree(pair) => (MappingKind::Tree, pair),
            };
            let operation = "mapping_add";
            let snapshot = publication::load(&home, true).map_err(|error| {
                error
                    .for_mapping_operation(operation)
                    .for_mapping_kind(kind)
                    .with_paths_if_empty(vec![
                        home.path().join("config.toml").display().to_string(),
                    ])
            })?;
            let source_evidence = path_policy::inspect_endpoint(
                std::path::Path::new(&pair.source),
                kind,
                true,
                operation,
            )
            .map_err(|error| error.for_mapping_kind(kind))?;
            let destination_evidence = path_policy::inspect_endpoint(
                std::path::Path::new(&pair.destination),
                kind,
                false,
                operation,
            )
            .map_err(|error| error.for_mapping_kind(kind))?;
            let added = Mapping::new(
                kind,
                source_evidence.canonical.clone(),
                destination_evidence.canonical.clone(),
            );
            let mut mappings = snapshot.registry.mappings().to_vec();
            mappings.push(added.clone());
            let candidate = registry::Registry::new(mappings)
                .map_err(|error| error.for_operation(operation).for_mapping_kind(kind))?;
            publication::publish_with_evidence(
                &home,
                &snapshot,
                &candidate,
                &[source_evidence, destination_evidence],
            )
            .map_err(|error| {
                error
                    .for_operation(operation)
                    .for_mapping_kind(kind)
                    .with_paths_if_empty(vec![
                        home.path().join("config.toml").display().to_string(),
                    ])
            })?;
            Ok(CommandOutcome::mapping_success(
                operation,
                "Mapping recorded",
                &added,
            ))
        }
        MappingCommand::List => {
            let operation = "mapping_list";
            let snapshot = publication::load(&home, false).map_err(|error| {
                error
                    .for_mapping_operation(operation)
                    .with_paths_if_empty(vec![
                        home.path().join("config.toml").display().to_string(),
                    ])
            })?;
            Ok(CommandOutcome::mapping_list(snapshot.registry.mappings()))
        }
        MappingCommand::Inspect(args) => {
            let operation = "mapping_inspect";
            let registry_path = home.path().join("config.toml").display().to_string();
            let snapshot = publication::load(&home, false).map_err(|error| {
                let message = error.to_string();
                match error.category() {
                    ResultCategory::UnsupportedSchema => error
                        .for_mapping_operation(operation)
                        .with_paths_if_empty(vec![registry_path]),
                    ResultCategory::InternalError => GripError::discovery_operational(
                        operation,
                        "invalid_registry",
                        vec![registry_path],
                        &message,
                    ),
                    _ => GripError::discovery_invalid(
                        operation,
                        "invalid_registry",
                        vec![registry_path],
                        &message,
                    ),
                }
            })?;
            let selected_source = args
                .source
                .as_ref()
                .map(|source| {
                    path_policy::resolve_selector(std::path::Path::new(source), operation)
                })
                .transpose()?;
            let inventory = discovery::inspect(&home, &snapshot, selected_source)?;
            Ok(CommandOutcome::discovery(&inventory))
        }
        MappingCommand::Show(args) => {
            let operation = "mapping_show";
            let snapshot = publication::load(&home, false).map_err(|error| {
                error
                    .for_mapping_operation(operation)
                    .with_paths_if_empty(vec![
                        home.path().join("config.toml").display().to_string(),
                    ])
            })?;
            let source =
                path_policy::resolve_selector(std::path::Path::new(&args.source), operation)?;
            let value = snapshot
                .registry
                .mappings()
                .iter()
                .find(|mapping| mapping.source == source)
                .ok_or_else(|| {
                    GripError::mapping(
                        operation,
                        "mapping_not_found",
                        vec![source.display().to_string()],
                        "Mapping not found",
                    )
                })?;
            Ok(CommandOutcome::mapping_success(
                operation,
                "Mapping found",
                value,
            ))
        }
        MappingCommand::Remove(args) => {
            let operation = "mapping_remove";
            let snapshot = publication::load(&home, true).map_err(|error| {
                error
                    .for_mapping_operation(operation)
                    .with_paths_if_empty(vec![
                        home.path().join("config.toml").display().to_string(),
                    ])
            })?;
            let source =
                path_policy::resolve_selector(std::path::Path::new(&args.source), operation)?;
            let removed = snapshot
                .registry
                .mappings()
                .iter()
                .find(|mapping| mapping.source == source)
                .cloned()
                .ok_or_else(|| {
                    GripError::mapping(
                        operation,
                        "mapping_not_found",
                        vec![source.display().to_string()],
                        "Mapping not found",
                    )
                })?;
            let mappings = snapshot
                .registry
                .mappings()
                .iter()
                .filter(|mapping| mapping.source != source)
                .cloned()
                .collect();
            let candidate = registry::Registry::new(mappings)
                .map_err(|error| error.for_operation(operation))?;
            publication::publish(&home, &snapshot, &candidate).map_err(|error| {
                error.for_operation(operation).with_paths_if_empty(vec![
                    home.path().join("config.toml").display().to_string(),
                ])
            })?;
            Ok(CommandOutcome::mapping_success(
                operation,
                "Mapping removed",
                &removed,
            ))
        }
    }
}

fn validate_selected_home() -> Result<(std::path::PathBuf, &'static str), GripError> {
    use std::fs;
    let selected = selected_home()?;
    registry::publication::load(&selected, false)?;
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
