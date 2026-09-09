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
pub mod recovery;
pub mod registry;
pub mod resolve;
pub mod result;
pub mod retire;
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
    let recovery_reference = match &cli.command {
        cli::Command::Recovery(cli::RecoveryArgs {
            command: cli::RecoveryCommand::Restore(args),
        }) if matches!(
            args.reference,
            recovery::model::RecoveryRef::Registry { .. }
        ) =>
        {
            Some(&args.reference)
        }
        _ => None,
    };
    let context = match project::ProjectContext::select(selection.clone(), user_home.clone()) {
        Ok(Some(value)) => value,
        Ok(None) => unreachable!("project-dependent command selected no project"),
        Err(error) => {
            let Some(reference) = recovery_reference else {
                return (CommandOutcome::failure(&error), None);
            };
            match project::ProjectContext::select_for_registry_recovery(
                selection, user_home, reference,
            ) {
                Ok(context) => context,
                Err(bootstrap_error) => {
                    return (CommandOutcome::failure(&bootstrap_error), None);
                }
            }
        }
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
        cli::Command::Validate => match validate_selected_project() {
            Ok((path, state)) => {
                let mut outcome = CommandOutcome::success(format!(
                    "Grip project {} is valid; state is {state}",
                    path.display()
                ));
                outcome
                    .details
                    .insert("project_root".into(), path.display().to_string().into());
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
        cli::Command::Pull(args) => match execute_pull(args) {
            Ok(outcome) => outcome,
            Err(error) => CommandOutcome::failure(&error),
        },
        cli::Command::Sync(args) => match execute_sync(args) {
            Ok(outcome) => outcome,
            Err(error) => CommandOutcome::failure(&error),
        },
        cli::Command::Resolve(args) => match execute_resolve(args) {
            Ok(outcome) => outcome,
            Err(error) => CommandOutcome::failure(&error),
        },
        cli::Command::Delete(args) => match execute_delete(args) {
            Ok(outcome) => outcome,
            Err(error) => CommandOutcome::failure(&error),
        },
        cli::Command::Retire(args) => match execute_retire(args) {
            Ok(outcome) => outcome,
            Err(error) => CommandOutcome::failure(&error),
        },
        cli::Command::Recovery(args) => match execute_recovery(&args.command) {
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

thread_local! {
    static SELECTED_PROJECT: std::cell::RefCell<Option<project::ProjectContext>> = const {
        std::cell::RefCell::new(None)
    };
}

fn execute_delete(args: &cli::DeleteArgs) -> Result<CommandOutcome, GripError> {
    let context = selected_project()?;
    let home = selected_home()?;
    let registry = registry::publication::load(&home, false)
        .map_err(|error| error.for_mapping_operation("delete"))?;
    let state = state::publication::load(&home)?;
    let selector =
        resolve_portable_selector(&context, &args.path, observation::model::PathSpace::Source)?;
    let selection = observation::model::resolve_selection(
        &registry,
        &state.accepted,
        Some(&selector),
        observation::model::PathSpace::Source,
        "delete",
    )?;
    let observed = observation::inspect(&home, &registry, &state.accepted, &selection)
        .map_err(|error| error.for_operation("delete"))?;
    state::publication::revalidate(&home, &state)?;
    let records = observed
        .values()
        .map(|entry| classification::classify_accepted(entry, &state.accepted))
        .collect();
    let authority = if args.source {
        delete::model::DeletionAuthority::Source
    } else {
        delete::model::DeletionAuthority::Destination
    };
    let plan = delete::plan::build(
        authority,
        classification_scope(
            &selection,
            Some(&selector),
            observation::model::PathSpace::Source,
        ),
        records,
    )?;
    if args.dry_run || !plan.blockers.is_empty() || plan.actions.is_empty() {
        return Ok(CommandOutcome::deletion(
            &plan,
            if args.dry_run { "dry_run" } else { "execute" },
            None,
            unattempted_baseline(state.accepted.generation),
        ));
    }
    revalidate_project_for_mutation()?;
    state::rebinding::require_mutation(&state.rebinding)?;
    let result = delete::execution::execute(&home, &registry, &state, &selection, &plan)?;
    Ok(CommandOutcome::deletion(
        &result.plan,
        &result.mode,
        result.operation_record.as_deref(),
        result.baseline,
    ))
}

fn execute_retire(args: &cli::RetireArgs) -> Result<CommandOutcome, GripError> {
    let context = selected_project()?;
    let home = selected_home()?;
    let registry = registry::publication::load(&home, false)
        .map_err(|error| error.for_mapping_operation("retire"))?;
    let state = state::publication::load(&home)?;
    let path_space = if args.destination {
        observation::model::PathSpace::Destination
    } else {
        observation::model::PathSpace::Source
    };
    let selector = match args.path.as_deref() {
        Some(path) => Some(resolve_portable_selector(&context, path, path_space)?),
        None => None,
    };
    let selection = observation::model::resolve_selection(
        &registry,
        &state.accepted,
        selector.as_deref(),
        path_space,
        "retire",
    )?;
    let observed = observation::inspect(&home, &registry, &state.accepted, &selection)
        .map_err(|error| error.for_operation("retire"))?;
    state::publication::revalidate(&home, &state)?;
    let records = observed
        .values()
        .map(|entry| classification::classify_accepted(entry, &state.accepted))
        .collect();
    let plan = retire::plan::build(
        classification_scope(&selection, selector.as_deref(), path_space),
        args.force,
        records,
    )?;
    if args.dry_run || !plan.blockers.is_empty() || plan.actions.is_empty() {
        return Ok(CommandOutcome::retirement(
            &plan,
            if args.dry_run { "dry_run" } else { "execute" },
            None,
            unattempted_baseline(state.accepted.generation),
        ));
    }
    revalidate_project_for_mutation()?;
    state::rebinding::require_mutation(&state.rebinding)?;
    let result = retire::execution::execute(&home, &registry, &state, &selection, &plan)?;
    Ok(CommandOutcome::retirement(
        &result.plan,
        &result.mode,
        result.operation_record.as_deref(),
        result.baseline,
    ))
}

fn execute_recovery(command: &cli::RecoveryCommand) -> Result<CommandOutcome, GripError> {
    let home = selected_home()?;
    recovery::dispatch(&home, command)
}

fn unattempted_baseline(generation: Option<u64>) -> mutation::model::BaselineOutcome {
    mutation::model::BaselineOutcome {
        outcome: "not_attempted".into(),
        prior_generation: generation,
        published_generation: None,
        authoritative_generation: generation,
        publication_visible: false,
        durability_confirmed: true,
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

fn execute_resolve(args: &cli::ResolveArgs) -> Result<CommandOutcome, GripError> {
    let context = selected_project()?;
    let home = selected_home()?;
    let registry = registry::publication::load(&home, false)
        .map_err(|error| error.for_mapping_operation("resolve"))?;
    let state = state::publication::load(&home)?;
    let selector_path =
        resolve_portable_selector(&context, &args.path, observation::model::PathSpace::Source)?;
    let selected = observation::model::resolve_selection(
        &registry,
        &state.accepted,
        Some(&selector_path),
        observation::model::PathSpace::Source,
        "resolve",
    )?;
    let selection = exact_resolution_selection(selected, &state.accepted, &selector_path)?;
    let observed = observation::inspect(&home, &registry, &state.accepted, &selection)
        .map_err(|error| error.for_operation("resolve"))?;
    state::publication::revalidate(&home, &state)
        .map_err(|error| error.for_operation("resolve"))?;
    let records = observed
        .values()
        .map(|entry| classification::classify_accepted(entry, &state.accepted))
        .collect();
    let scope = classification_scope(
        &selection,
        Some(&selector_path),
        observation::model::PathSpace::Source,
    );
    let winner = if args.source {
        mutation::model::ConflictWinner::Source
    } else {
        mutation::model::ConflictWinner::Destination
    };
    let plan = mutation::plan::build_resolution(scope, records, winner)?;
    if args.dry_run || !plan.blockers.is_empty() {
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

fn execute_baseline_accept(args: &cli::InspectionArgs) -> Result<CommandOutcome, GripError> {
    let home = selected_home()?;
    execute_baseline_accept_with_hook(&home, args, || {})
}

#[doc(hidden)]
pub fn execute_baseline_accept_with_hook<F>(
    home: &project::ProjectPaths,
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
    let selector = match args.path.as_deref() {
        Some(path) => Some(resolve_portable_selector(
            &selected_project()?,
            path,
            path_space,
        )?),
        None => None,
    };
    let selection = observation::model::resolve_selection(
        &expected_registry,
        &expected_state.accepted,
        selector.as_deref(),
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
        .map(|entry| classification::classify_accepted(entry, &expected_state.accepted))
        .collect::<Vec<_>>();
    let candidate = baseline::build(&expected_state.accepted, &records)?;
    if candidate.changed_count == 0
        && expected_state.rebinding.outcome != result::RebindingOutcome::RebindEligible
    {
        return Ok(CommandOutcome::baseline(
            &baseline::AcceptanceResult::already_current(
                candidate.selected_count,
                expected_state.accepted.generation,
            ),
        ));
    }
    after_initial();

    state::rebinding::require_mutation(&expected_state.rebinding)?;

    let _mutation_guard = state::mutation_lock::MutationLock::acquire(home, "baseline_accept")?;
    revalidate_project_for_mutation()?;
    let _registry_guard = registry::publication::acquire_guard(home, "baseline_accept")?;
    let state_lock = state::lock::project_lock_path(home, "state.lock")?;
    let _state_guard = state::lock::PublicationLock::acquire(&state_lock)?;
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
        selector.as_deref(),
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
        .map(|entry| classification::classify_accepted(entry, &expected_state.accepted))
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
    let generation = state::publication::publish_complete_locked(
        home,
        &expected_state,
        &locked_candidate.next.complete_baselines,
    )?;
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

pub(crate) fn revalidate_project_for_mutation() -> Result<(), GripError> {
    SELECTED_PROJECT.with(|slot| match slot.borrow().as_ref() {
        Some(context) => context.revalidate(),
        None => Ok(()),
    })
}

fn execute_mapping(command: &cli::MappingCommand) -> Result<CommandOutcome, GripError> {
    use cli::{MappingAddKind, MappingCommand};
    use mapping::{Mapping, MappingKind};
    use registry::publication;

    let context = selected_project()?;
    let home = project::ProjectPaths::project_metadata(
        context.metadata_dir.clone(),
        context.user_home.path().to_path_buf(),
    );
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
            let portable = mapping::PortableMapping::parse(kind, &pair.source, &pair.destination)
                .map_err(|error| error.for_mapping_kind(kind))?;
            let source_evidence = path_policy::inspect_endpoint(
                &portable.source.resolve(&context.root),
                kind,
                true,
                operation,
            )
            .map_err(|error| error.for_mapping_kind(kind))?;
            let destination_evidence = path_policy::inspect_endpoint(
                &portable.destination.resolve(context.user_home.path()),
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
            let candidate = registry::ResolvedRegistry::new(mappings)
                .map_err(|error| error.for_operation(operation).for_mapping_kind(kind))?;
            let _mutation_guard = state::mutation_lock::MutationLock::acquire(&home, operation)?;
            revalidate_project_for_mutation()?;
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
                &result::MappingResultDetails::from_parts(&portable, &added),
            ))
        }
        MappingCommand::List => Ok(CommandOutcome::mapping_list(&context.mapping_details())),
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
                    path_policy::ProjectRelativePath::parse(source, true)
                        .map(|path| path.resolve(&context.root))
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
                path_policy::ProjectRelativePath::parse(&args.source, true)?.resolve(&context.root);
            let index = snapshot
                .registry
                .mappings()
                .iter()
                .position(|mapping| mapping.source == source)
                .ok_or_else(|| {
                    GripError::mapping(
                        operation,
                        "mapping_not_found",
                        vec![source.display().to_string()],
                        "Mapping not found",
                    )
                })?;
            let details = snapshot.mapping_details()?;
            Ok(CommandOutcome::mapping_success(
                operation,
                "Mapping found",
                &details[index],
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
                path_policy::ProjectRelativePath::parse(&args.source, true)?.resolve(&context.root);
            let removed_index = snapshot
                .registry
                .mappings()
                .iter()
                .position(|mapping| mapping.source == source)
                .ok_or_else(|| {
                    GripError::mapping(
                        operation,
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
            let candidate = registry::ResolvedRegistry::new(mappings)
                .map_err(|error| error.for_operation(operation))?;
            let _mutation_guard = state::mutation_lock::MutationLock::acquire(&home, operation)?;
            revalidate_project_for_mutation()?;
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

fn validate_selected_project() -> Result<(std::path::PathBuf, &'static str), GripError> {
    use std::fs;
    let context = selected_project()?;
    let state_path = context.state_dir.join("state.json");
    match fs::symlink_metadata(&state_path) {
        Ok(m) => {
            if m.file_type().is_symlink() || !m.is_file() {
                return Err(GripError::CorruptState(
                    "state.json must be a regular file".into(),
                ));
            }
            let bytes = fs::read(&state_path)
                .map_err(|e| GripError::CorruptState(format!("state.json is unreadable: {e}")))?;
            state::decode_v4(&bytes)?;
            Ok((context.root, "valid"))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok((context.root, "uninitialized")),
        Err(e) => Err(GripError::CorruptState(format!(
            "state.json is unavailable: {e}"
        ))),
    }
}
