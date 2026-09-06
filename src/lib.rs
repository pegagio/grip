pub mod baseline;
pub mod classification;
pub mod cli;
pub mod discovery;
pub mod error;
pub mod home;
pub mod mapping;
pub mod observation;
pub mod operation;
pub mod path_policy;
pub mod push;
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
    let render_home = selected_home().ok();
    if render_command_result(
        outcome,
        parsed.output.into(),
        &mut io::stdout().lock(),
        render_home.as_ref(),
    )
    .is_err()
    {
        let _ = writeln!(io::stderr().lock(), "grip: could not write command result");
        return std::process::ExitCode::from(20);
    }
    std::process::ExitCode::from(exit)
}

/// Render once and finalize only the associated operation's delivery evidence.
#[doc(hidden)]
pub fn render_command_result(
    outcome: CommandOutcome,
    mode: result::OutputMode,
    writer: &mut dyn std::io::Write,
    home: Option<&home::GripHome>,
) -> std::io::Result<()> {
    let operation_id = outcome
        .details
        .get("operation_record")
        .and_then(serde_json::Value::as_object)
        .and_then(|record| record.get("id"))
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned);
    let rendered = result::render(outcome, mode, writer);
    if let (Some(operation_id), Some(home)) = (&operation_id, home) {
        let delivery = if rendered.is_ok() {
            "delivered"
        } else {
            "failed"
        };
        let _ = operation::publication::finalize_result_delivery(home, operation_id, delivery);
    }
    rendered
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
        cli::Command::Status(args) => inspection_outcome("status", args),
        cli::Command::Check(args) => inspection_outcome("check", args),
        cli::Command::Diff(args) => inspection_outcome("diff", args),
        cli::Command::Push(args) => match execute_push(args) {
            Ok(outcome) => outcome,
            Err(error) => CommandOutcome::failure(&error),
        },
        cli::Command::Baseline(args) => match &args.command {
            cli::BaselineCommand::Accept(arguments) => match execute_baseline_accept(arguments) {
                Ok(outcome) => outcome,
                Err(error) => CommandOutcome::failure(&error),
            },
        },
    }
}

fn execute_push(args: &cli::PushArgs) -> Result<CommandOutcome, GripError> {
    let home = selected_home()?;
    let registry = registry::publication::load(&home, false)
        .map_err(|error| error.for_mapping_operation("push"))?;
    let state = state::publication::load(&home)?;
    let path_space = if args.destination {
        observation::model::PathSpace::Destination
    } else {
        observation::model::PathSpace::Source
    };
    let selector = args.path.as_deref().map(std::path::Path::new);
    let selection = observation::model::resolve_selection(
        &registry,
        &state.accepted,
        selector,
        path_space,
        "push",
    )?;
    let observed = observation::inspect(&home, &registry, &state.accepted, &selection)
        .map_err(|error| error.for_operation("push"))?;
    state::publication::revalidate(&home, &state).map_err(|error| error.for_operation("push"))?;
    let records = observed
        .values()
        .map(|entry| classification::classify(entry, state.accepted.baselines.get(&entry.identity)))
        .collect();
    let scope = classification_scope(&selection, selector, path_space);
    let plan = push::plan::build_with_parent_requirements(
        scope,
        records,
        registry.missing_destination_parents(),
    )?;
    if args.dry_run || !plan.blockers.is_empty() || plan.actions.is_empty() {
        return Ok(CommandOutcome::push_plan(
            &plan,
            if args.dry_run { "dry_run" } else { "execute" },
            state.accepted.generation,
        ));
    }
    let applied = push::execution::execute(&home, &registry, &state, &selection, &plan)?;
    Ok(CommandOutcome::push_applied(&applied))
}

fn inspection_outcome(operation: &str, args: &cli::InspectionArgs) -> CommandOutcome {
    match execute_inspection(operation, args) {
        Ok(outcome) => outcome,
        Err(error) => CommandOutcome::failure(&error),
    }
}

fn execute_inspection(
    operation: &str,
    args: &cli::InspectionArgs,
) -> Result<CommandOutcome, GripError> {
    let home = selected_home()?;
    let registry = registry::publication::load(&home, false)
        .map_err(|error| error.for_mapping_operation(operation))?;
    let state = state::publication::load(&home)?;
    let path_space = if args.destination {
        observation::model::PathSpace::Destination
    } else {
        observation::model::PathSpace::Source
    };
    let selector = args.path.as_deref().map(std::path::Path::new);
    let selection = observation::model::resolve_selection(
        &registry,
        &state.accepted,
        selector,
        path_space,
        operation,
    )?;
    let observed = observation::inspect(&home, &registry, &state.accepted, &selection)
        .map_err(|error| error.for_operation(operation))?;
    state::publication::revalidate(&home, &state)
        .map_err(|error| error.for_operation(operation))?;
    let records = observed
        .values()
        .map(|entry| classification::classify(entry, state.accepted.baselines.get(&entry.identity)))
        .collect();
    let scope = classification_scope(&selection, selector, path_space);
    let result = classification::model::ClassificationResult::new(operation, scope, records);
    Ok(CommandOutcome::classification(&result))
}

fn classification_scope(
    selection: &observation::model::Selection,
    selector: Option<&std::path::Path>,
    path_space: observation::model::PathSpace,
) -> classification::model::ClassificationScope {
    use observation::model::Selection;
    let (kind, mapping_source) = match selection {
        Selection::All => ("all", None),
        Selection::Mapping(mapping) => ("mapping", Some(mapping.source.display().to_string())),
        Selection::Entry(identity) => {
            ("entry", Some(identity.mapping.source.display().to_string()))
        }
        Selection::Subtree(identity) => (
            "subtree",
            Some(identity.mapping.source.display().to_string()),
        ),
        Selection::Unmanaged(_) => ("unmanaged", None),
    };
    classification::model::ClassificationScope {
        kind: kind.into(),
        path_space,
        selector: selector.map(crate::discovery::model::SafePath::from_path),
        mapping_source,
    }
}

fn execute_baseline_accept(args: &cli::InspectionArgs) -> Result<CommandOutcome, GripError> {
    let home = selected_home()?;
    execute_baseline_accept_with_hook(&home, args, || {})
}

#[doc(hidden)]
pub fn execute_baseline_accept_with_hook<F>(
    home: &home::GripHome,
    args: &cli::InspectionArgs,
    after_initial: F,
) -> Result<CommandOutcome, GripError>
where
    F: FnOnce(),
{
    let expected_registry = registry::publication::load(home, false)
        .map_err(|error| error.for_mapping_operation("baseline_accept"))?;
    let expected_state = state::publication::load(home)?;
    let path_space = if args.destination {
        observation::model::PathSpace::Destination
    } else {
        observation::model::PathSpace::Source
    };
    let selector = args.path.as_deref().map(std::path::Path::new);
    let selection = observation::model::resolve_selection(
        &expected_registry,
        &expected_state.accepted,
        selector,
        path_space,
        "baseline_accept",
    )?;
    let observed = observation::inspect(
        home,
        &expected_registry,
        &expected_state.accepted,
        &selection,
    )
    .map_err(|error| error.for_operation("baseline_accept"))?;
    state::publication::revalidate(home, &expected_state)
        .map_err(|error| error.for_operation("baseline_accept"))?;
    let records = observed
        .values()
        .map(|entry| {
            classification::classify(
                entry,
                expected_state.accepted.baselines.get(&entry.identity),
            )
        })
        .collect::<Vec<_>>();
    let candidate = baseline::build(&expected_state.accepted, &records)?;
    if candidate.changed_count == 0 {
        return Ok(CommandOutcome::baseline(
            &baseline::AcceptanceResult::already_current(
                candidate.selected_count,
                expected_state.accepted.generation,
            ),
        ));
    }
    after_initial();

    let _mutation_guard = state::mutation_lock::MutationLock::acquire(home, "baseline_accept")?;
    let _registry_guard = registry::publication::acquire_guard(home, "baseline_accept")?;
    let state_dir = state::publication::prepare_directory(home)?;
    let _state_guard = state::lock::PublicationLock::acquire(&state_dir.join("state.lock"))?;
    let locked_registry = registry::publication::load(home, false)
        .map_err(|error| error.for_mapping_operation("baseline_accept"))?;
    if locked_registry.bytes != expected_registry.bytes
        || locked_registry.registry != expected_registry.registry
    {
        return Err(GripError::discovery_operational(
            "baseline_accept",
            "stale_registry_evidence",
            vec![home.path().join("config.toml").display().to_string()],
            "accepted registry changed before baseline publication",
        ));
    }
    state::publication::revalidate(home, &expected_state)
        .map_err(|error| error.for_operation("baseline_accept"))?;
    let locked_selection = observation::model::resolve_selection(
        &locked_registry,
        &expected_state.accepted,
        selector,
        path_space,
        "baseline_accept",
    )?;
    let locked_observed = observation::inspect(
        home,
        &locked_registry,
        &expected_state.accepted,
        &locked_selection,
    )
    .map_err(|error| error.for_operation("baseline_accept"))?;
    let locked_records = locked_observed
        .values()
        .map(|entry| {
            classification::classify(
                entry,
                expected_state.accepted.baselines.get(&entry.identity),
            )
        })
        .collect::<Vec<_>>();
    if locked_records != records {
        return Err(GripError::discovery_operational(
            "baseline_accept",
            "stale_baseline_evidence",
            Vec::new(),
            "accepted baseline evidence changed before publication",
        ));
    }
    let locked_candidate = baseline::build(&expected_state.accepted, &locked_records)?;
    registry::publication::revalidate_readonly(home, &locked_registry, "baseline_accept")?;
    let generation =
        state::publication::publish_accepted_locked(home, &expected_state, &locked_candidate.next)?;
    let result = match generation {
        Some(generation) => baseline::AcceptanceResult::accepted(
            locked_candidate.selected_count,
            locked_candidate.changed_count,
            generation,
        ),
        None => baseline::AcceptanceResult::already_current(
            locked_candidate.selected_count,
            expected_state.accepted.generation,
        ),
    };
    Ok(CommandOutcome::baseline(&result))
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
            let _mutation_guard = state::mutation_lock::MutationLock::acquire(&home, operation)?;
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
            let _mutation_guard = state::mutation_lock::MutationLock::acquire(&home, operation)?;
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
            let bytes = fs::read(&state_path)
                .map_err(|e| GripError::CorruptState(format!("state.json is unreadable: {e}")))?;
            state::decode_accepted(Some(&bytes))?;
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
