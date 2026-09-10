pub mod baseline;
pub mod classification;
pub mod cli;
pub mod delete;
pub mod discovery;
pub mod error;
pub mod home;
pub mod mapping;
pub mod metadata;
pub mod mutation;
pub mod observation;
pub mod operation;
pub mod path_policy;
pub mod project;
pub mod pull;
pub mod push;
pub mod registry;
pub mod result;
pub mod state;
pub mod sync;

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
    let (outcome, render_home) = execute_with_project(&parsed);
    let exit = outcome.category.exit_code();
    let _ = result::emit_diagnostic(
        parsed.verbose,
        "command completed",
        &mut io::stderr().lock(),
    );
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
    home: Option<&project::ProjectPaths>,
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
    execute_with_project(cli).0
}

fn execute_with_project(cli: &cli::Cli) -> (CommandOutcome, Option<project::ProjectPaths>) {
    if cli.project.is_some() && matches!(cli.command, cli::Command::Init(_) | cli::Command::Version)
    {
        let error = GripError::lifecycle(
            "command_parse",
            "inapplicable_project_option",
            ResultCategory::InvalidUsage,
            "--project is not valid with init or version",
        );
        return (CommandOutcome::failure(&error), None);
    }
    if matches!(cli.command, cli::Command::Init(_) | cli::Command::Version) {
        return (execute_selected(cli), None);
    }
    let user_home = match home::select_user_home(None) {
        Ok(value) => value,
        Err(error) => return (CommandOutcome::failure(&error), None),
    };
    let selection = if let Some(path) = &cli.project {
        project::ProjectSelection::Explicit(path.clone())
    } else {
        match std::env::current_dir() {
            Ok(cwd) => project::ProjectSelection::Discovered(cwd),
            Err(error) => {
                return (
                    CommandOutcome::failure(&GripError::from_io(
                        "could not read invocation directory",
                        error,
                    )),
                    None,
                );
            }
        }
    };
    let context = match project::ProjectContext::select(selection.clone(), user_home.clone()) {
        Ok(Some(value)) => value,
        Ok(None) => unreachable!("project-dependent command selected no project"),
        Err(error) => return (CommandOutcome::failure(&error), None),
    };
    let selected_home = project::ProjectPaths::project_metadata(
        context.metadata_dir.clone(),
        context.user_home.path().to_path_buf(),
    );
    let outcome = SELECTED_PROJECT.with(|slot| {
        let previous = slot.replace(Some(context.clone()));
        let outcome = execute_selected(cli);
        slot.replace(previous);
        outcome
    });
    let assessment = state::publication::load(&selected_home)
        .map(|snapshot| snapshot.rebinding)
        .unwrap_or_else(|_| state::rebinding::RebindingAssessment {
            outcome: result::RebindingOutcome::RebindBlocked,
            prior_root: None,
            blockers: vec!["state_unavailable".into()],
        });
    (
        outcome.with_project_state(&context, &assessment),
        Some(selected_home),
    )
}

fn execute_selected(cli: &cli::Cli) -> CommandOutcome {
    match &cli.command {
        cli::Command::Init(args) => match project::init::initialize(args.path.as_deref()) {
            Ok(result) => CommandOutcome::initialization(&result),
            Err(error) => CommandOutcome::failure(&error),
        },
        cli::Command::Version => {
            let mut outcome =
                CommandOutcome::success(format!("grip {}", env!("CARGO_PKG_VERSION")));
            outcome
                .details
                .insert("version".into(), env!("CARGO_PKG_VERSION").into());
            outcome
        }
        cli::Command::Add(args) => match execute_add(args) {
            Ok(outcome) => outcome,
            Err(error) => CommandOutcome::failure(&error),
        },
        cli::Command::List(args) => match execute_list(args) {
            Ok(outcome) => outcome,
            Err(error) => CommandOutcome::failure(&error),
        },
        cli::Command::Remove(args) => match execute_remove(args) {
            Ok(outcome) => outcome,
            Err(error) => CommandOutcome::failure(&error),
        },
        cli::Command::Status(args) => status_outcome(args),
        cli::Command::Diff(args) => inspection_outcome("diff", args),
        cli::Command::Push(args) => match execute_push(args) {
            Ok(outcome) => outcome,
            Err(error) => CommandOutcome::failure(&error),
        },
        cli::Command::Pull(args) => match execute_pull(args) {
            Ok(outcome) => outcome,
            Err(error) => CommandOutcome::failure(&error),
        },
        cli::Command::Sync(args) => match execute_sync(args) {
            Ok(outcome) => outcome,
            Err(error) => CommandOutcome::failure(&error),
        },
    }
}

fn status_outcome(args: &cli::StatusArgs) -> CommandOutcome {
    let inspection = cli::InspectionArgs {
        destination: args.destination,
        path: args.path.clone(),
    };
    let mut outcome = inspection_outcome("status", &inspection);
    if args.exit_code
        && outcome.category == ResultCategory::Success
        && outcome
            .details
            .get("attention_count")
            .and_then(serde_json::Value::as_u64)
            .is_some_and(|count| count > 0)
    {
        outcome.category = ResultCategory::AttentionRequired;
    }
    outcome
}

thread_local! {
    static SELECTED_PROJECT: std::cell::RefCell<Option<project::ProjectContext>> = const {
        std::cell::RefCell::new(None)
    };
}

fn execute_push(args: &cli::PushArgs) -> Result<CommandOutcome, GripError> {
    if args.force {
        return execute_forced_direction(
            args.dry_run,
            args.destination,
            args.path.as_deref(),
            mutation::model::ConflictWinner::Source,
        );
    }
    let home = selected_home()?;
    let registry = registry::publication::load(&home, false)
        .map_err(|error| error.for_mapping_operation("push"))?;
    let state = state::publication::load(&home)?;
    let path_space = if args.destination {
        observation::model::PathSpace::Destination
    } else {
        observation::model::PathSpace::Source
    };
    let selector = match args.path.as_deref() {
        Some(path) => Some(resolve_portable_selector(
            &selected_project()?,
            path,
            path_space,
        )?),
        None => None,
    };
    let selection = observation::model::resolve_selection(
        &registry,
        &state.accepted,
        selector.as_deref(),
        path_space,
        "push",
    )?;
    let observed = observation::inspect(&home, &registry, &state.accepted, &selection)
        .map_err(|error| error.for_operation("push"))?;
    state::publication::revalidate(&home, &state).map_err(|error| error.for_operation("push"))?;
    let records = observed
        .values()
        .map(|entry| classification::classify_accepted(entry, &state.accepted))
        .collect();
    let scope = classification_scope(&selection, selector.as_deref(), path_space);
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
    revalidate_project_for_mutation()?;
    state::rebinding::require_mutation(&state.rebinding)?;
    let applied = push::execution::execute(&home, &registry, &state, &selection, &plan)?;
    Ok(CommandOutcome::push_applied(&applied))
}

fn execute_pull(args: &cli::PullArgs) -> Result<CommandOutcome, GripError> {
    if args.force {
        return execute_forced_direction(
            args.dry_run,
            args.destination,
            args.path.as_deref(),
            mutation::model::ConflictWinner::Destination,
        );
    }
    let context = selected_project()?;
    let home = selected_home()?;
    let registry = registry::publication::load(&home, false)
        .map_err(|error| error.for_mapping_operation("pull"))?;
    let state = state::publication::load(&home)?;
    let path_space = if args.destination {
        observation::model::PathSpace::Destination
    } else {
        observation::model::PathSpace::Source
    };
    let selector = args
        .path
        .as_deref()
        .map(|path| resolve_portable_selector(&context, path, path_space))
        .transpose()?;
    let selection = observation::model::resolve_selection(
        &registry,
        &state.accepted,
        selector.as_deref(),
        path_space,
        "pull",
    )?;
    let observed = observation::inspect(&home, &registry, &state.accepted, &selection)
        .map_err(|error| error.for_operation("pull"))?;
    state::publication::revalidate(&home, &state).map_err(|error| error.for_operation("pull"))?;
    let records = observed
        .values()
        .map(|entry| classification::classify_accepted(entry, &state.accepted))
        .collect();
    let scope = classification_scope(&selection, selector.as_deref(), path_space);
    let plan = mutation::plan::build_for(mutation::model::MutationDirection::Pull, scope, records)?;
    if args.dry_run || !plan.blockers.is_empty() || plan.actions.is_empty() {
        return Ok(CommandOutcome::mutation_plan(
            &plan,
            if args.dry_run { "dry_run" } else { "execute" },
            state.accepted.generation,
        ));
    }
    revalidate_project_for_mutation()?;
    state::rebinding::require_mutation(&state.rebinding)?;
    let applied = mutation::execution::execute(&home, &registry, &state, &selection, &plan)?;
    Ok(CommandOutcome::mutation_applied(&applied))
}

fn execute_forced_direction(
    dry_run: bool,
    destination: bool,
    path: Option<&std::ffi::OsStr>,
    winner: mutation::model::ConflictWinner,
) -> Result<CommandOutcome, GripError> {
    let path = path.ok_or_else(|| {
        GripError::lifecycle(
            "force",
            "force_requires_exact_entry",
            ResultCategory::InvalidUsage,
            "--force requires a selector resolving to exactly one managed entry",
        )
    })?;
    let context = selected_project()?;
    let home = selected_home()?;
    let registry = registry::publication::load(&home, false)?;
    let state = state::publication::load(&home)?;
    let path_space = if destination {
        observation::model::PathSpace::Destination
    } else {
        observation::model::PathSpace::Source
    };
    let selector = resolve_portable_selector(&context, path, path_space)?;
    let selected = observation::model::resolve_selection(
        &registry,
        &state.accepted,
        Some(&selector),
        path_space,
        "force",
    )?;
    let selection = exact_resolution_selection(selected, &state.accepted, &selector)?;
    let observed = observation::inspect(&home, &registry, &state.accepted, &selection)?;
    state::publication::revalidate(&home, &state)?;
    let records = observed
        .values()
        .map(|entry| classification::classify_accepted(entry, &state.accepted))
        .collect::<Vec<_>>();
    let scope = classification_scope(&selection, Some(&selector), path_space);
    let absence_authority = match (winner, records.first().map(|record| record.classification)) {
        (
            mutation::model::ConflictWinner::Source,
            Some(classification::model::Classification::SourceSideDeletion),
        ) => Some(delete::model::DeletionAuthority::Source),
        (
            mutation::model::ConflictWinner::Destination,
            Some(classification::model::Classification::DestinationSideDeletion),
        ) => Some(delete::model::DeletionAuthority::Destination),
        _ => None,
    };
    if records.len() == 1
        && let Some(authority) = absence_authority
    {
        let mut records = records;
        // Ordinary planning blocks one-sided absence. An exact forced direction is the
        // explicit authority that discharges only that blocker.
        records[0].blocking = false;
        let plan = delete::plan::build(authority, scope, records)?;
        let baseline = mutation::model::BaselineOutcome {
            outcome: "not_attempted".into(),
            prior_generation: state.accepted.generation,
            published_generation: None,
            authoritative_generation: state.accepted.generation,
            publication_visible: false,
            durability_confirmed: true,
        };
        if dry_run || !plan.blockers.is_empty() {
            return Ok(CommandOutcome::deletion(
                &plan,
                if dry_run { "dry_run" } else { "execute" },
                None,
                baseline,
            ));
        }
        revalidate_project_for_mutation()?;
        state::rebinding::require_mutation(&state.rebinding)?;
        let applied = delete::execution::execute(&home, &registry, &state, &selection, &plan)?;
        return Ok(CommandOutcome::deletion(
            &applied.plan,
            &applied.mode,
            applied.operation_record.as_deref(),
            applied.baseline,
        ));
    }
    let plan = mutation::plan::build_resolution(scope, records, winner)?;
    if dry_run || !plan.blockers.is_empty() {
        return Ok(CommandOutcome::mutation_plan(
            &plan,
            if dry_run { "dry_run" } else { "execute" },
            state.accepted.generation,
        ));
    }
    revalidate_project_for_mutation()?;
    state::rebinding::require_mutation(&state.rebinding)?;
    let applied = mutation::execution::execute(&home, &registry, &state, &selection, &plan)?;
    Ok(CommandOutcome::mutation_applied(&applied))
}

fn execute_sync(args: &cli::SyncArgs) -> Result<CommandOutcome, GripError> {
    let context = selected_project()?;
    let home = selected_home()?;
    let registry = registry::publication::load(&home, false)
        .map_err(|error| error.for_mapping_operation("sync"))?;
    let state = state::publication::load(&home)?;
    let path_space = if args.destination {
        observation::model::PathSpace::Destination
    } else {
        observation::model::PathSpace::Source
    };
    let selector = args
        .path
        .as_deref()
        .map(|path| resolve_portable_selector(&context, path, path_space))
        .transpose()?;
    let selection = observation::model::resolve_selection(
        &registry,
        &state.accepted,
        selector.as_deref(),
        path_space,
        "sync",
    )?;
    let observed = observation::inspect(&home, &registry, &state.accepted, &selection)
        .map_err(|error| error.for_operation("sync"))?;
    state::publication::revalidate(&home, &state).map_err(|error| error.for_operation("sync"))?;
    let records = observed
        .values()
        .map(|entry| classification::classify_accepted(entry, &state.accepted))
        .collect();
    let scope = classification_scope(&selection, selector.as_deref(), path_space);
    let plan = mutation::plan::build_sync_with_parent_requirements(
        scope,
        records,
        registry.missing_destination_parents(),
    )?;
    if args.dry_run
        || !plan.blockers.is_empty()
        || plan.actions.is_empty() && plan.acceptance_identities.is_empty()
    {
        return Ok(CommandOutcome::mutation_plan(
            &plan,
            if args.dry_run { "dry_run" } else { "execute" },
            state.accepted.generation,
        ));
    }
    revalidate_project_for_mutation()?;
    state::rebinding::require_mutation(&state.rebinding)?;
    let applied = mutation::execution::execute(&home, &registry, &state, &selection, &plan)?;
    Ok(CommandOutcome::mutation_applied(&applied))
}

fn exact_resolution_selection(
    selection: observation::model::Selection,
    accepted: &state::AcceptedState,
    requested: &std::path::Path,
) -> Result<observation::model::Selection, GripError> {
    use observation::model::Selection;
    match selection {
        Selection::Entry(identity) => Ok(Selection::Entry(identity)),
        Selection::Subtree(identity) if accepted.complete_baselines.contains_key(&identity) => {
            Ok(Selection::Entry(identity))
        }
        Selection::Mapping(mapping) => {
            let identity = observation::model::EntryIdentity::new(mapping, Vec::new())
                .map_err(|message| GripError::InvalidConfiguration(message.into()))?;
            if accepted.complete_baselines.contains_key(&identity) {
                Ok(Selection::Entry(identity))
            } else {
                Err(resolution_selector_error(requested))
            }
        }
        Selection::All | Selection::Subtree(_) | Selection::Unmanaged(_) => {
            Err(resolution_selector_error(requested))
        }
    }
}

fn resolution_selector_error(path: &std::path::Path) -> GripError {
    GripError::mapping(
        "resolve",
        "resolution_requires_exact_entry",
        vec![crate::discovery::model::SafePath::from_path(path).display],
        "resolution requires one exact established managed entry in source-path space",
    )
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
    let context = selected_project()?;
    let home = selected_home()?;
    let registry = registry::publication::load(&home, false)
        .map_err(|error| error.for_mapping_operation(operation))?;
    let state = state::publication::load(&home)?;
    let path_space = if args.destination {
        observation::model::PathSpace::Destination
    } else {
        observation::model::PathSpace::Source
    };
    let selector = args
        .path
        .as_deref()
        .map(|path| resolve_portable_selector(&context, path, path_space))
        .transpose()?;
    let selection = observation::model::resolve_selection(
        &registry,
        &state.accepted,
        selector.as_deref(),
        path_space,
        operation,
    )?;
    let observed = observation::inspect(&home, &registry, &state.accepted, &selection)
        .map_err(|error| error.for_operation(operation))?;
    state::publication::revalidate(&home, &state)
        .map_err(|error| error.for_operation(operation))?;
    let records = observed
        .values()
        .map(|entry| classification::classify_accepted(entry, &state.accepted))
        .collect();
    let scope = classification_scope(&selection, selector.as_deref(), path_space);
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

fn resolve_portable_selector(
    context: &project::ProjectContext,
    selector: &std::ffi::OsStr,
    path_space: observation::model::PathSpace,
) -> Result<std::path::PathBuf, GripError> {
    match path_space {
        observation::model::PathSpace::Source => {
            path_policy::ProjectRelativePath::parse(selector, true)
                .map(|path| path.resolve(&context.root))
        }
        observation::model::PathSpace::Destination => {
            path_policy::HomeRelativePath::parse(selector)
                .map(|path| path.resolve(context.user_home.path()))
        }
    }
}

fn selected_home() -> Result<project::ProjectPaths, GripError> {
    selected_project().map(|context| {
        project::ProjectPaths::project_metadata(
            context.metadata_dir,
            context.user_home.path().to_path_buf(),
        )
    })
}

fn selected_project() -> Result<project::ProjectContext, GripError> {
    SELECTED_PROJECT.with(|slot| {
        slot.borrow().clone().ok_or_else(|| {
            GripError::InvalidConfiguration("Grip project context is unavailable".into())
        })
    })
}

fn execute_add(args: &cli::AddArgs) -> Result<CommandOutcome, GripError> {
    use mapping::Mapping;

    let operation = "add";
    let context = selected_project()?;
    let home = selected_home()?;
    let source_path =
        path_policy::ProjectRelativePath::parse(&args.source, true)?.resolve(&context.root);
    let destination_path =
        path_policy::HomeRelativePath::parse(&args.destination)?.resolve(context.user_home.path());
    let source_kind = existing_mapping_kind(&source_path, operation)?;
    let destination_kind = existing_mapping_kind(&destination_path, operation)?;
    let kind = match (source_kind, destination_kind) {
        (None, None) => {
            return Err(GripError::mapping(
                operation,
                "both_endpoints_absent",
                vec![
                    source_path.display().to_string(),
                    destination_path.display().to_string(),
                ],
                "add requires at least one existing source or destination endpoint",
            ));
        }
        (Some(kind), None) | (None, Some(kind)) => kind,
        (Some(source), Some(destination)) if source == destination => source,
        (Some(_), Some(_)) => {
            return Err(GripError::mapping(
                operation,
                "incompatible_endpoint_kinds",
                vec![
                    source_path.display().to_string(),
                    destination_path.display().to_string(),
                ],
                "source and destination endpoints must both be files or both be directories",
            ));
        }
    };
    let portable = mapping::PortableMapping::parse(kind, &args.source, &args.destination)?;
    let snapshot = registry::publication::load(&home, true).map_err(|error| {
        error
            .for_mapping_operation(operation)
            .for_mapping_kind(kind)
    })?;
    let source_evidence = path_policy::inspect_durable_endpoint(
        &portable.source.resolve(&context.root),
        kind,
        true,
        operation,
    )?;
    let destination_evidence = path_policy::inspect_durable_endpoint(
        &portable.destination.resolve(context.user_home.path()),
        kind,
        false,
        operation,
    )?;
    let added = Mapping::new(
        kind,
        source_evidence.canonical.clone(),
        destination_evidence.canonical.clone(),
    );
    let mut mappings = snapshot.registry.mappings().to_vec();
    mappings.push(added.clone());
    let candidate = registry::ResolvedRegistry::new(mappings)?;
    let _mutation_guard = state::mutation_lock::MutationLock::acquire(&home, operation)?;
    revalidate_project_for_mutation()?;
    registry::publication::publish_with_evidence(
        &home,
        &snapshot,
        &candidate,
        &[source_evidence, destination_evidence],
    )?;
    if source_kind.is_some() && destination_kind.is_some() {
        establish_added_baseline(&home, &added)?;
    }
    Ok(CommandOutcome::mapping_success(
        operation,
        "Mapping recorded",
        &result::MappingResultDetails::from_parts(&portable, &added),
    ))
}

fn existing_mapping_kind(
    path: &std::path::Path,
    operation: &str,
) -> Result<Option<mapping::MappingKind>, GripError> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => Err(GripError::mapping(
            operation,
            "symlink_endpoint",
            vec![path.display().to_string()],
            "mapping endpoint must not be a symbolic link",
        )),
        Ok(metadata) if metadata.is_file() => Ok(Some(mapping::MappingKind::File)),
        Ok(metadata) if metadata.is_dir() => Ok(Some(mapping::MappingKind::Tree)),
        Ok(_) => Err(GripError::mapping(
            operation,
            "unsupported_node",
            vec![path.display().to_string()],
            "mapping endpoint must be a regular file or directory",
        )),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(GripError::from_io(
            "could not inspect mapping endpoint",
            error,
        )),
    }
}

fn execute_list(args: &cli::ListArgs) -> Result<CommandOutcome, GripError> {
    let context = selected_project()?;
    let home = selected_home()?;
    let snapshot = registry::publication::load(&home, false)
        .map_err(|error| error.for_mapping_operation("list"))?;
    let details = snapshot.mapping_details()?;
    let mappings = if let Some(source) = &args.source {
        let source = path_policy::ProjectRelativePath::parse(source, true)?.resolve(&context.root);
        details
            .into_iter()
            .filter(|mapping| {
                mapping.resolved.source == crate::discovery::model::SafePath::from_path(&source)
            })
            .collect::<Vec<_>>()
    } else {
        details
    };
    Ok(CommandOutcome::mapping_list(&mappings))
}

fn execute_remove(args: &cli::RemoveArgs) -> Result<CommandOutcome, GripError> {
    let context = selected_project()?;
    let home = selected_home()?;
    let snapshot = registry::publication::load(&home, true)
        .map_err(|error| error.for_mapping_operation("remove"))?;
    let source =
        path_policy::ProjectRelativePath::parse(&args.source, true)?.resolve(&context.root);
    let removed_index = snapshot
        .registry
        .mappings()
        .iter()
        .position(|mapping| mapping.source == source)
        .ok_or_else(|| {
            GripError::mapping(
                "remove",
                "mapping_not_found",
                vec![source.display().to_string()],
                "Mapping not found",
            )
        })?;
    let removed = snapshot.mapping_details()?[removed_index].clone();
    let mappings = snapshot
        .registry
        .mappings()
        .iter()
        .filter(|mapping| mapping.source != source)
        .cloned()
        .collect();
    let candidate = registry::ResolvedRegistry::new(mappings)?;
    let _mutation_guard = state::mutation_lock::MutationLock::acquire(&home, "remove")?;
    revalidate_project_for_mutation()?;
    registry::publication::publish(&home, &snapshot, &candidate)?;
    prune_removed_baseline(&home, &source)?;
    Ok(CommandOutcome::mapping_success(
        "remove",
        "Mapping removed",
        &removed,
    ))
}

/// Establish the first accepted baseline only when `add` found both endpoints present.
fn establish_added_baseline(
    home: &project::ProjectPaths,
    added: &mapping::Mapping,
) -> Result<(), GripError> {
    let registry = registry::publication::load(home, false)?;
    let state = state::publication::load(home)?;
    let selection =
        observation::model::Selection::Mapping(observation::model::ResolvedMapping::from(added));
    let observed = observation::inspect(home, &registry, &state.accepted, &selection)?;
    state::publication::revalidate(home, &state)?;
    let records = observed
        .values()
        .map(|entry| classification::classify_accepted(entry, &state.accepted))
        .collect::<Vec<_>>();
    match baseline::build(&state.accepted, &records) {
        Ok(candidate) => {
            state::publication::publish_current_locked(home, &state, &candidate.next)?;
        }
        // Differing endpoints are an intentionally unbaselined initial collision. They are
        // recorded, inspected, and later made explicit with `push --force` or `pull --force`.
        Err(GripError::BaselineNotAcceptable { .. }) => {}
        Err(error) => return Err(error),
    }
    Ok(())
}

/// Forget accepted evidence for a removed declaration without changing endpoint payloads.
fn prune_removed_baseline(
    home: &project::ProjectPaths,
    source: &std::path::Path,
) -> Result<(), GripError> {
    let state = state::publication::load(home)?;
    let mut next = state.accepted.clone();
    next.complete_baselines
        .retain(|identity, _| identity.mapping.source != source);
    state::publication::publish_current_locked(home, &state, &next)?;
    Ok(())
}

pub(crate) fn revalidate_project_for_mutation() -> Result<(), GripError> {
    SELECTED_PROJECT.with(|slot| match slot.borrow().as_ref() {
        Some(context) => context.revalidate(),
        None => Ok(()),
    })
}
