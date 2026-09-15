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
                let outcome = CommandOutcome::invalid_usage(error.to_string().trim().to_owned());
                let _ = result::render(outcome, result::OutputMode::Json, &mut io::stdout().lock());
            } else {
                let message = error.to_string();
                let message = message
                    .strip_prefix("error:")
                    .map_or(message.as_str(), |value| value.trim_start());
                let _ = write!(io::stderr().lock(), "Error: {message}");
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
    let invocation_directory = match std::env::current_dir().and_then(std::fs::canonicalize) {
        Ok(cwd) => cwd,
        Err(error) => {
            return (
                CommandOutcome::failure(&GripError::from_io(
                    "could not read invocation directory",
                    error,
                )),
                None,
            );
        }
    };
    let selection = if let Some(path) = &cli.project {
        project::ProjectSelection::Explicit(path.clone())
    } else {
        project::ProjectSelection::Discovered(invocation_directory.clone())
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
        let outcome = INVOCATION_DIRECTORY.with(|directory| {
            let previous_directory = directory.replace(Some(invocation_directory));
            let outcome = execute_selected(cli);
            directory.replace(previous_directory);
            outcome
        });
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
    static INVOCATION_DIRECTORY: std::cell::RefCell<Option<std::path::PathBuf>> = const {
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
        Some(path) => Some(resolve_push_selector(
            &selected_project()?,
            &invocation_directory()?,
            path,
            path_space,
        )?),
        None => None,
    };
    reject_fenced_selector(&home, selector.as_deref(), path_space)?;
    let selection = observation::model::resolve_selection(
        &registry,
        &state.accepted,
        selector.as_deref(),
        path_space,
        "push",
    )?;
    reject_fenced_selection(&home, &selection)?;
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
        let outcome = CommandOutcome::push_plan(
            &plan,
            if args.dry_run { "dry_run" } else { "execute" },
            state.accepted.generation,
        );
        return Ok(
            outcome.with_human_blocker_guidance(force_resolution_guidance(
                &plan,
                &state.accepted,
                &invocation_directory()?,
            )),
        );
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
    reject_fenced_selector(&home, selector.as_deref(), path_space)?;
    let selection = observation::model::resolve_selection(
        &registry,
        &state.accepted,
        selector.as_deref(),
        path_space,
        "pull",
    )?;
    reject_fenced_selection(&home, &selection)?;
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
        let outcome = CommandOutcome::mutation_plan(
            &plan,
            if args.dry_run { "dry_run" } else { "execute" },
            state.accepted.generation,
        );
        return Ok(
            outcome.with_human_blocker_guidance(force_resolution_guidance(
                &plan,
                &state.accepted,
                &invocation_directory()?,
            )),
        );
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
    let selector = if winner == mutation::model::ConflictWinner::Source {
        resolve_push_selector(&context, &invocation_directory()?, path, path_space)?
    } else {
        resolve_portable_selector(&context, path, path_space)?
    };
    reject_fenced_selector(&home, Some(&selector), path_space)?;
    let selected = observation::model::resolve_selection(
        &registry,
        &state.accepted,
        Some(&selector),
        path_space,
        "force",
    )?;
    let selection = exact_resolution_selection(selected, &state.accepted, &selector)?;
    reject_fenced_selection(&home, &selection)?;
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
        let outcome = CommandOutcome::mutation_plan(
            &plan,
            if dry_run { "dry_run" } else { "execute" },
            state.accepted.generation,
        );
        return Ok(
            outcome.with_human_blocker_guidance(force_resolution_guidance(
                &plan,
                &state.accepted,
                &invocation_directory()?,
            )),
        );
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
    reject_fenced_selector(&home, selector.as_deref(), path_space)?;
    let selection = observation::model::resolve_selection(
        &registry,
        &state.accepted,
        selector.as_deref(),
        path_space,
        "sync",
    )?;
    reject_fenced_selection(&home, &selection)?;
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
        let outcome = CommandOutcome::mutation_plan(
            &plan,
            if args.dry_run { "dry_run" } else { "execute" },
            state.accepted.generation,
        );
        return Ok(
            outcome.with_human_blocker_guidance(force_resolution_guidance(
                &plan,
                &state.accepted,
                &invocation_directory()?,
            )),
        );
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

/// Reject mutation work that includes the sole mapping guarded by an incomplete add fence.
/// Explicit selection keeps unrelated mappings usable while the operator retries that add.
fn reject_fenced_selection(
    home: &project::ProjectPaths,
    selection: &observation::model::Selection,
) -> Result<(), GripError> {
    let Some(fence) = state::add_fence::load(home)? else {
        return Ok(());
    };
    let selected = match selection {
        observation::model::Selection::All => true,
        observation::model::Selection::Mapping(mapping) => fence.protects_resolved(mapping),
        observation::model::Selection::Entry(identity)
        | observation::model::Selection::Subtree(identity) => {
            fence.protects_resolved(&identity.mapping)
        }
        observation::model::Selection::Unmanaged(_) => false,
    };
    if selected {
        return Err(GripError::InvalidConfiguration(
            incomplete_fence_message(&fence).into(),
        ));
    }
    Ok(())
}

/// Fail early for a selector that identifies a mapping present only in a pre-descriptor fence.
/// Such a mapping cannot pass normal registry selection, but it is still a known blocked mapping
/// and should direct the operator to the same-add retry.
fn reject_fenced_selector(
    home: &project::ProjectPaths,
    selector: Option<&std::path::Path>,
    path_space: observation::model::PathSpace,
) -> Result<(), GripError> {
    let Some(selector) = selector else {
        return Ok(());
    };
    let Some(fence) = state::add_fence::load(home)? else {
        return Ok(());
    };
    if fence.protects_selector(selector, path_space) {
        return Err(GripError::InvalidConfiguration(
            incomplete_fence_message(&fence).into(),
        ));
    }
    Ok(())
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
    let fence = state::add_fence::load(&home)?;
    if operation == "status"
        && let Some(fence) = fence.as_ref()
        && selector
            .as_deref()
            .is_some_and(|selector| fence.protects_selector(selector, path_space))
    {
        return fenced_status_outcome(fence, selector.as_deref(), path_space);
    }
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
    let mut records = observed
        .values()
        .map(|entry| classification::classify_accepted(entry, &state.accepted))
        .collect::<Vec<_>>();
    if operation == "status"
        && let Some(fence) = fence.as_ref()
    {
        let mut present = false;
        for record in &mut records {
            if fence.protects_resolved(&record.identity.mapping) {
                present = true;
                record.classification = classification::model::Classification::UnsafeCollision;
                record.prospective_direction = classification::model::Direction::None;
                record.attention = true;
                record.blocking = true;
                record.reasons = vec![incomplete_fence_reason(fence).into()];
            }
        }
        if !present && selector.is_none() {
            records.push(fenced_classification_record(fence)?);
        }
    }
    let scope = classification_scope(&selection, selector.as_deref(), path_space);
    let result = classification::model::ClassificationResult::new(operation, scope, records);
    let outcome = CommandOutcome::classification(&result);
    if operation == "status" {
        let invocation_directory = invocation_directory()?;
        let source_paths = result
            .records
            .iter()
            .map(|record| {
                path_policy::git_relative_display(
                    &record.identity.source_path(),
                    &invocation_directory,
                )
            })
            .collect();
        let guidance = status_conflict_guidance(&result, &state.accepted, &invocation_directory);
        Ok(outcome
            .with_human_status_source_paths(source_paths)
            .with_human_status_guidance(guidance))
    } else {
        Ok(outcome)
    }
}

/// Render the mapping from a fence even if descriptor publication never completed.
fn fenced_status_outcome(
    fence: &state::add_fence::AddPublicationFenceV1,
    selector: Option<&std::path::Path>,
    path_space: observation::model::PathSpace,
) -> Result<CommandOutcome, GripError> {
    let mapping = fence.resolved_mapping();
    let selection = observation::model::Selection::Mapping(mapping);
    let record = fenced_classification_record(fence)?;
    let scope = classification_scope(&selection, selector, path_space);
    let result = classification::model::ClassificationResult::new("status", scope, vec![record]);
    let invocation_directory = invocation_directory()?;
    let source_paths = result
        .records
        .iter()
        .map(|record| {
            path_policy::git_relative_display(&record.identity.source_path(), &invocation_directory)
        })
        .collect();
    Ok(CommandOutcome::classification(&result)
        .with_human_status_source_paths(source_paths)
        .with_human_status_guidance(vec![None]))
}

/// Produce the existing blocked-conflict representation for a fence whose descriptor is absent.
fn fenced_classification_record(
    fence: &state::add_fence::AddPublicationFenceV1,
) -> Result<classification::model::ClassificationRecord, GripError> {
    let mapping = fence.resolved_mapping();
    let identity = observation::model::EntryIdentity::new(mapping.clone(), Vec::new())
        .map_err(|message| GripError::CorruptState(message.into()))?;
    Ok(classification::model::ClassificationRecord {
        identity,
        classification: classification::model::Classification::UnsafeCollision,
        mapping_kind: mapping.kind,
        mapping_source: mapping.source.clone(),
        relative_path: None,
        source_path: discovery::model::SafePath::from_path(&mapping.source),
        destination_path: discovery::model::SafePath::from_path(&mapping.destination),
        source: None,
        destination: None,
        baseline: None,
        source_complete: None,
        destination_complete: None,
        baseline_complete: None,
        compatibility_findings: Vec::new(),
        endpoint_capabilities: Vec::new(),
        prospective_direction: classification::model::Direction::None,
        changed_dimensions: classification::model::ChangedDimensions {
            source_to_baseline: None,
            destination_to_baseline: None,
            source_to_destination: None,
        },
        attention: true,
        blocking: true,
        reasons: vec![incomplete_fence_reason(fence).into()],
    })
}

fn incomplete_fence_reason(fence: &state::add_fence::AddPublicationFenceV1) -> &'static str {
    match fence.operation {
        state::add_fence::FenceOperation::Add => "incomplete_add_publication",
        state::add_fence::FenceOperation::Remove => "incomplete_remove_publication",
    }
}

fn incomplete_fence_message(fence: &state::add_fence::AddPublicationFenceV1) -> &'static str {
    match fence.operation {
        state::add_fence::FenceOperation::Add => {
            "selected mapping has an incomplete add publication; rerun grip add for that mapping"
        }
        state::add_fence::FenceOperation::Remove => {
            "selected mapping has an incomplete remove publication; rerun grip remove for that mapping"
        }
    }
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

fn status_conflict_guidance(
    result: &classification::model::ClassificationResult,
    accepted: &state::AcceptedState,
    invocation_directory: &std::path::Path,
) -> Vec<Option<result::HumanConflictGuidance>> {
    result
        .records
        .iter()
        .map(|record| {
            if !is_ordinary_force_conflict(record.classification, &record.compatibility_findings) {
                return None;
            }
            Some(force_resolution_guidance_for_identity(
                &record.identity,
                &record.destination_path.display,
                accepted,
                invocation_directory,
            ))
        })
        .collect()
}

fn force_resolution_guidance(
    plan: &mutation::model::MutationPlan,
    accepted: &state::AcceptedState,
    invocation_directory: &std::path::Path,
) -> Vec<Option<result::HumanConflictGuidance>> {
    plan.blockers
        .iter()
        .map(|blocker| {
            if !matches!(
                blocker.reason.as_str(),
                "initial_collision"
                    | "divergent_change"
                    | "one_sided_absence_requires_force"
                    | "delete_change_conflict"
                    | "change_delete_conflict"
            ) || blocker.paths.len() < 2
            {
                return None;
            }
            let entry = plan.entries.iter().find(|entry| {
                entry.source_path == blocker.paths[0] && entry.destination_path == blocker.paths[1]
            })?;
            Some(force_resolution_guidance_for_identity(
                &entry.identity,
                &entry.destination_path.display,
                accepted,
                invocation_directory,
            ))
        })
        .collect()
}

fn is_ordinary_force_conflict(
    classification: classification::model::Classification,
    findings: &[metadata::model::CompatibilityFinding],
) -> bool {
    matches!(
        classification,
        classification::model::Classification::InitialCollision
            | classification::model::Classification::DivergentConflict
            | classification::model::Classification::SourceSideDeletion
            | classification::model::Classification::DestinationSideDeletion
            | classification::model::Classification::DeleteChangeConflict
            | classification::model::Classification::ChangeDeleteConflict
    ) && !findings.iter().any(|finding| finding.blocking)
}

fn force_resolution_guidance_for_identity(
    identity: &observation::model::EntryIdentity,
    destination: &str,
    accepted: &state::AcceptedState,
    invocation_directory: &std::path::Path,
) -> result::HumanConflictGuidance {
    use observation::model::Selection;
    let selection = match (identity.mapping.kind, identity.relative_path.is_empty()) {
        (mapping::MappingKind::File, _) => Selection::Entry(identity.clone()),
        (mapping::MappingKind::Tree, true) => Selection::Mapping(identity.mapping.clone()),
        (mapping::MappingKind::Tree, false) => Selection::Subtree(identity.clone()),
    };
    let source_path = identity.source_path();
    let source = path_policy::git_relative_display(&source_path, invocation_directory);
    if exact_resolution_selection(selection, accepted, &source_path).is_ok() {
        result::HumanConflictGuidance::ForcePair {
            source,
            destination: destination.into(),
        }
    } else {
        result::HumanConflictGuidance::InspectDiff {
            source,
            destination: destination.into(),
        }
    }
}

fn resolve_portable_selector(
    context: &project::ProjectContext,
    selector: &std::ffi::OsStr,
    path_space: observation::model::PathSpace,
) -> Result<std::path::PathBuf, GripError> {
    match path_space {
        observation::model::PathSpace::Source => {
            path_policy::ProjectRelativePath::parse_cli(selector, true)
                .map(|path| path.resolve(&context.root))
        }
        observation::model::PathSpace::Destination => path_policy::DestinationPath::parse(selector)
            .map(|path| path.resolve(&context.root, context.user_home.path())),
    }
}

fn resolve_push_selector(
    context: &project::ProjectContext,
    invocation_directory: &std::path::Path,
    selector: &std::ffi::OsStr,
    path_space: observation::model::PathSpace,
) -> Result<std::path::PathBuf, GripError> {
    match path_space {
        observation::model::PathSpace::Source => path_policy::resolve_cwd_relative_source_selector(
            &context.root,
            invocation_directory,
            selector,
            "push",
        ),
        observation::model::PathSpace::Destination => {
            resolve_portable_selector(context, selector, path_space)
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

fn invocation_directory() -> Result<std::path::PathBuf, GripError> {
    INVOCATION_DIRECTORY.with(|slot| {
        slot.borrow().clone().ok_or_else(|| {
            GripError::lifecycle(
                "command_execution",
                "invocation_directory_unavailable",
                ResultCategory::InternalError,
                "invocation directory is unavailable during project execution",
            )
        })
    })
}

fn execute_add(args: &cli::AddArgs) -> Result<CommandOutcome, GripError> {
    use mapping::Mapping;

    let operation = "add";
    let context = selected_project()?;
    let home = selected_home()?;
    let source_path =
        path_policy::ProjectRelativePath::parse_cli(&args.source, true)?.resolve(&context.root);
    let destination_path = path_policy::DestinationPath::parse(&args.destination)?
        .resolve(&context.root, context.user_home.path());
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
    let portable = mapping::PortableMapping::parse_cli(kind, &args.source, &args.destination)?;
    let source_evidence = path_policy::inspect_durable_endpoint(
        &portable.source.resolve(&context.root),
        kind,
        true,
        operation,
    )?;
    let destination_evidence = path_policy::inspect_durable_endpoint(
        &portable
            .destination
            .resolve(&context.root, context.user_home.path()),
        kind,
        false,
        operation,
    )?;
    let added = Mapping::new(
        kind,
        source_evidence.canonical.clone(),
        destination_evidence.canonical.clone(),
    );
    let _mutation_guard = state::mutation_lock::MutationLock::acquire(&home, operation)?;
    revalidate_project_for_mutation()?;
    if let Some(outcome) = resume_fenced_add(&home, &added, &portable)? {
        return Ok(outcome);
    }
    // A stale fenced candidate may have just restored the prior descriptor. Reload after that
    // recovery so the new declaration is built from the authoritative registry, not from the
    // candidate that was deliberately rolled back.
    let snapshot = registry::publication::load(&home, true).map_err(|error| {
        error
            .for_mapping_operation(operation)
            .for_mapping_kind(kind)
    })?;
    let replacement = if args.force {
        let matches = snapshot
            .registry
            .mappings()
            .iter()
            .filter(|mapping| {
                mapping.kind == mapping::MappingKind::File
                    && mapping.destination == added.destination
                    && mapping.source != added.source
            })
            .cloned()
            .collect::<Vec<_>>();
        if matches.len() != 1 {
            return Err(GripError::mapping(
                operation,
                "force_add_requires_exact_destination_mapping",
                vec![added.destination.display().to_string()],
                "force add requires exactly one distinct active file mapping with the requested destination",
            ));
        }
        let displaced = matches.into_iter().next().expect("one replacement match");
        let declaration = snapshot
            .portable_mappings()?
            .into_iter()
            .find(|mapping| mapping.source.resolve(&context.root) == displaced.source)
            .ok_or_else(|| {
                GripError::CorruptState(
                    "accepted replacement mapping has no portable declaration".into(),
                )
            })?;
        Some((displaced, declaration))
    } else {
        None
    };
    let mut declarations = snapshot.portable_mappings()?;
    if let Some((displaced, _)) = &replacement {
        declarations.retain(|mapping| mapping.source.resolve(&context.root) != displaced.source);
    }
    declarations.push(portable.clone());
    let candidate_descriptor = registry::ProjectDescriptorV2::new(declarations)?;
    let candidate = registry::ResolvedRegistry::new(
        candidate_descriptor
            .resolve(&context.root, context.user_home.path(), operation)?
            .into_iter()
            .map(|mapping| mapping.ownership_mapping())
            .collect(),
    )?;
    if let Some((displaced, displaced_declaration)) = &replacement {
        let state_snapshot = state::publication::load(&home)?;
        let mut accepted = state_snapshot.accepted.clone();
        let displaced_identity = observation::model::ResolvedMapping::from(displaced);
        accepted
            .complete_baselines
            .retain(|identity, _| identity.mapping != displaced_identity);
        let descriptor_bytes = registry::encode_descriptor(&candidate_descriptor)?;
        let selection = observation::model::Selection::Mapping(
            observation::model::ResolvedMapping::from(&added),
        );
        let observed = observation::inspect_candidate(
            &home,
            &candidate,
            &descriptor_bytes,
            &snapshot,
            &accepted,
            &selection,
        )?;
        state::publication::revalidate(&home, &state_snapshot)?;
        let records = observed
            .values()
            .map(|entry| classification::classify_accepted(entry, &accepted))
            .collect::<Vec<_>>();
        let initial_state = baseline::build_for_add(&accepted, &records)?;
        let state_bytes = state::publication::prepare_candidate_for_descriptor(
            &home,
            &state_snapshot,
            &initial_state.next.complete_baselines,
            &descriptor_bytes,
        )?;
        let fence = state::add_fence::AddPublicationFenceV1::with_context(
            &added,
            state::add_fence::FenceContext::replacement(
                &portable,
                displaced,
                displaced_declaration,
            ),
            snapshot.bytes.clone(),
            descriptor_bytes.clone(),
            state_snapshot.bytes.clone(),
            state_bytes.clone(),
        );
        state::add_fence::create_verified(&home, &fence)?;
        registry::publication::publish_with_evidence(
            &home,
            &snapshot,
            &candidate,
            &candidate_descriptor,
            &[source_evidence, destination_evidence],
        )?;
        let published_registry = registry::publication::load(&home, false)?;
        if !fenced_candidate_is_current(&home, &added, &published_registry, &fence)? {
            return Err(GripError::InvalidConfiguration(
                "destination changed before Grip could record the replacement state; rerun grip add for this mapping"
                    .into(),
            ));
        }
        state::publication::publish_prepared_locked(
            &home,
            &state_snapshot,
            &initial_state.next.complete_baselines,
            &state_bytes,
        )?;
        let published_state = state::publication::load(&home)?;
        if published_registry.bytes != descriptor_bytes
            || published_state.bytes.as_deref() != Some(state_bytes.as_slice())
        {
            return Err(GripError::CorruptState(
                "replacement publication did not match its fenced candidate".into(),
            ));
        }
        state::add_fence::clear_verified(&home, &fence)?;
        return Ok(CommandOutcome::mapping_replaced(
            &result::MappingResultDetails::from_parts(&portable, &added),
            &result::MappingResultDetails::from_parts(displaced_declaration, displaced),
        ));
    }
    if source_kind.is_some() && destination_kind.is_some() {
        let state_snapshot = state::publication::load(&home)?;
        let descriptor_bytes = registry::encode_descriptor(&candidate_descriptor)?;
        let selection = observation::model::Selection::Mapping(
            observation::model::ResolvedMapping::from(&added),
        );
        let observed = observation::inspect_candidate(
            &home,
            &candidate,
            &descriptor_bytes,
            &snapshot,
            &state_snapshot.accepted,
            &selection,
        )?;
        state::publication::revalidate(&home, &state_snapshot)?;
        let records = observed
            .values()
            .map(|entry| classification::classify_accepted(entry, &state_snapshot.accepted))
            .collect::<Vec<_>>();
        let initial_state = baseline::build_for_add(&state_snapshot.accepted, &records)?;
        let unequal = records.iter().any(|record| {
            record.source_complete.is_some()
                && record.destination_complete.is_some()
                && record.source_complete != record.destination_complete
        });
        if unequal {
            let state_bytes = state::publication::prepare_candidate_for_descriptor(
                &home,
                &state_snapshot,
                &initial_state.next.complete_baselines,
                &descriptor_bytes,
            )?;
            let fence = state::add_fence::AddPublicationFenceV1::new(
                &added,
                snapshot.bytes.clone(),
                descriptor_bytes.clone(),
                state_snapshot.bytes.clone(),
                state_bytes.clone(),
            );
            state::add_fence::create_verified(&home, &fence)?;
            registry::publication::publish_with_evidence(
                &home,
                &snapshot,
                &candidate,
                &candidate_descriptor,
                &[source_evidence, destination_evidence],
            )?;
            let published_registry = registry::publication::load(&home, false)?;
            if !fenced_candidate_is_current(&home, &added, &published_registry, &fence)? {
                return Err(GripError::InvalidConfiguration(
                    "destination changed before Grip could record the initial comparison state; rerun grip add for this mapping"
                        .into(),
                ));
            }
            state::publication::publish_prepared_locked(
                &home,
                &state_snapshot,
                &initial_state.next.complete_baselines,
                &state_bytes,
            )?;
            let published_state = state::publication::load(&home)?;
            if published_registry.bytes != descriptor_bytes
                || published_state.bytes.as_deref() != Some(state_bytes.as_slice())
            {
                return Err(GripError::CorruptState(
                    "add publication did not match its fenced candidate".into(),
                ));
            }
            state::add_fence::clear_verified(&home, &fence)?;
            return Ok(CommandOutcome::mapping_success(
                operation,
                "Mapping recorded",
                &result::MappingResultDetails::from_parts(&portable, &added),
            ));
        }
    }
    registry::publication::publish_with_evidence(
        &home,
        &snapshot,
        &candidate,
        &candidate_descriptor,
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

/// Complete a previously fenced add when its descriptor/state candidate is still authoritative.
/// A fence with an unchanged prior pair is harmless cleanup; a visible candidate descriptor is
/// completed from the exact State V4 bytes captured before its publication began.
fn resume_fenced_add(
    home: &project::ProjectPaths,
    added: &mapping::Mapping,
    portable: &mapping::PortableMapping,
) -> Result<Option<CommandOutcome>, GripError> {
    let Some(fence) = state::add_fence::load(home)? else {
        return Ok(None);
    };
    if fence.operation != state::add_fence::FenceOperation::Add {
        return Err(GripError::InvalidConfiguration(
            "another mapping has an incomplete remove publication".into(),
        ));
    }
    if !fence.protects(added) {
        return Err(GripError::InvalidConfiguration(
            "another mapping has an incomplete add publication".into(),
        ));
    }
    let registry = registry::publication::load(home, false)?;
    let state_snapshot = state::publication::load(home)?;
    let descriptor_digest = state::add_fence::digest(&registry.bytes);
    let state_digest = state_snapshot
        .bytes
        .as_deref()
        .map(state::add_fence::digest);

    if descriptor_digest == fence.candidate_descriptor_digest {
        if !fenced_candidate_is_current(home, added, &registry, &fence)? {
            restore_fenced_add(home, &fence)?;
            return Ok(None);
        }
        if state_digest != Some(fence.candidate_state_digest.clone()) {
            if state_digest != fence.prior_state_digest {
                return Err(GripError::InvalidConfiguration(
                    "incomplete add publication state differs from both its prior and candidate state; retry cannot safely continue"
                        .into(),
                ));
            }
            let candidate_state = state::decode_v4(&fence.candidate_state_bytes)?;
            let runtime = state::runtime_from_accepted_v4(home, &candidate_state)?;
            state::publication::publish_prepared_locked(
                home,
                &state_snapshot,
                &runtime.baselines,
                &fence.candidate_state_bytes,
            )?;
        }
        let verified_state = state::publication::load(home)?;
        if verified_state.bytes.as_deref() != Some(fence.candidate_state_bytes.as_slice()) {
            return Err(GripError::CorruptState(
                "fenced add retry did not publish the expected State V4 candidate".into(),
            ));
        }
        state::add_fence::clear_verified(home, &fence)?;
        return Ok(Some(add_fence_outcome(portable, added, &fence)?));
    }

    if descriptor_digest == fence.prior_descriptor_digest
        && state_digest == fence.prior_state_digest
    {
        state::add_fence::clear_verified(home, &fence)?;
        return Ok(None);
    }

    Err(GripError::InvalidConfiguration(
        "incomplete add publication differs from both its prior and candidate state; retry cannot safely continue"
            .into(),
    ))
}

fn add_fence_outcome(
    portable: &mapping::PortableMapping,
    added: &mapping::Mapping,
    fence: &state::add_fence::AddPublicationFenceV1,
) -> Result<CommandOutcome, GripError> {
    if fence.result == state::add_fence::FenceResult::Replaced {
        let added = mapping_details_from_fence(fence.declaration.as_ref(), &fence.mapping)?;
        let replaced = replacement_details_from_fence(fence)?;
        Ok(CommandOutcome::mapping_replaced(&added, &replaced))
    } else {
        let added = result::MappingResultDetails::from_parts(portable, added);
        Ok(CommandOutcome::mapping_success(
            "add",
            "Mapping recorded",
            &added,
        ))
    }
}

fn replacement_details_from_fence(
    fence: &state::add_fence::AddPublicationFenceV1,
) -> Result<result::MappingResultDetails, GripError> {
    let replaced = fence.replaced_mapping.as_ref().ok_or_else(|| {
        GripError::CorruptState("replacement fence omitted the displaced mapping".into())
    })?;
    mapping_details_from_fence(fence.replaced_declaration.as_ref(), replaced)
}

fn mapping_details_from_fence(
    declaration: Option<&state::add_fence::FenceDeclaration>,
    expected: &state::add_fence::FenceMapping,
) -> Result<result::MappingResultDetails, GripError> {
    let declaration = declaration.ok_or_else(|| {
        GripError::CorruptState("publication fence omitted the protected declaration".into())
    })?;
    Ok(result::MappingResultDetails {
        declared: result::DeclaredMappingDetails {
            kind: declaration.kind,
            source: declaration.source.clone(),
            destination: declaration.destination.clone(),
        },
        resolved: result::ResolvedMappingDetails {
            source: discovery::model::SafePath::from_path(std::path::Path::new(&expected.source)),
            destination: discovery::model::SafePath::from_path(std::path::Path::new(
                &expected.destination,
            )),
        },
    })
}

/// Reinspect the candidate mapping against the destination-derived State V4 evidence captured
/// in the fence. A changed destination makes the candidate stale; it must be restored rather
/// than accepted merely because its descriptor bytes are still present.
fn fenced_candidate_is_current(
    home: &project::ProjectPaths,
    added: &mapping::Mapping,
    registry: &registry::publication::RegistrySnapshot,
    fence: &state::add_fence::AddPublicationFenceV1,
) -> Result<bool, GripError> {
    let candidate_state = state::decode_v4(&fence.candidate_state_bytes)?;
    let runtime = state::runtime_from_accepted_v4(home, &candidate_state)?;
    let accepted = state::AcceptedState {
        generation: Some(runtime.generation),
        complete_baselines: runtime.baselines,
        accepted_bytes: Some(fence.candidate_state_bytes.clone()),
    };
    let selection =
        observation::model::Selection::Mapping(observation::model::ResolvedMapping::from(added));
    let observed = observation::inspect(home, registry, &accepted, &selection)?;
    for (identity, baseline) in &accepted.complete_baselines {
        if identity.mapping != observation::model::ResolvedMapping::from(added) {
            continue;
        }
        let Some(entry) = observed.get(identity) else {
            return Ok(false);
        };
        if entry.blocking
            || entry
                .destination_complete
                .as_ref()
                .map(|state| &state.state)
                != Some(baseline)
        {
            return Ok(false);
        }
    }
    Ok(observed.values().all(|entry| !entry.blocking))
}

/// Restore the exact pre-add descriptor/state pair after a fenced candidate fails revalidation.
/// The fence itself supplies the expected candidate digests, so restoration refuses to overwrite
/// an unrelated concurrent publication.
fn restore_fenced_add(
    home: &project::ProjectPaths,
    fence: &state::add_fence::AddPublicationFenceV1,
) -> Result<(), GripError> {
    registry::publication::restore_exact(
        home,
        &fence.prior_descriptor_bytes,
        &fence.candidate_descriptor_digest,
    )?;
    let current = state::publication::load(home)?;
    if current.bytes.as_deref() != fence.prior_state_bytes.as_deref() {
        if let Some(prior_state) = &fence.prior_state_bytes {
            let candidate_state = state::decode_v4(&fence.candidate_state_bytes)?;
            state::publication::restore_exact(
                home,
                prior_state,
                candidate_state.generation,
                &fence.candidate_state_digest,
            )?;
        } else {
            state::publication::remove_exact_candidate(home, &fence.candidate_state_bytes)?;
        }
    }
    let restored = state::publication::load(home)?;
    if restored.bytes.as_deref() != fence.prior_state_bytes.as_deref() {
        return Err(GripError::CorruptState(
            "fenced add restoration did not recover the prior State V4 evidence".into(),
        ));
    }
    state::add_fence::clear_verified(home, fence)
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
    let selected = args.source.is_some();
    let mappings = if let Some(source) = &args.source {
        let source =
            path_policy::ProjectRelativePath::parse_cli(source, true)?.resolve(&context.root);
        details
            .into_iter()
            .filter(|mapping| {
                mapping.resolved.source == crate::discovery::model::SafePath::from_path(&source)
            })
            .collect::<Vec<_>>()
    } else {
        details
    };
    Ok(CommandOutcome::mapping_list(&mappings).with_human_mapping_selection(selected))
}

fn execute_remove(args: &cli::RemoveArgs) -> Result<CommandOutcome, GripError> {
    let context = selected_project()?;
    let home = selected_home()?;
    let source =
        path_policy::ProjectRelativePath::parse_cli(&args.source, true)?.resolve(&context.root);
    let _mutation_guard = state::mutation_lock::MutationLock::acquire(&home, "remove")?;
    revalidate_project_for_mutation()?;
    if let Some(outcome) = resume_fenced_remove(&home, &source)? {
        return Ok(outcome);
    }
    let snapshot = registry::publication::load(&home, true)
        .map_err(|error| error.for_mapping_operation("remove"))?;
    let removed = snapshot
        .mapping_details()?
        .into_iter()
        .find(|mapping| {
            mapping.resolved.source == crate::discovery::model::SafePath::from_path(&source)
        })
        .ok_or_else(|| {
            GripError::mapping(
                "remove",
                "mapping_not_found",
                vec![source.display().to_string()],
                "Mapping not found",
            )
        })?;
    let removed_mapping = snapshot
        .registry
        .mappings()
        .iter()
        .find(|mapping| mapping.source == source)
        .cloned()
        .ok_or_else(|| GripError::CorruptState("removed mapping is absent from registry".into()))?;
    let mappings = snapshot
        .registry
        .mappings()
        .iter()
        .filter(|mapping| mapping.source != source)
        .cloned()
        .collect();
    let candidate = registry::ResolvedRegistry::new(mappings)?;
    let declarations = snapshot.portable_mappings()?;
    let removed_declaration = declarations
        .iter()
        .find(|mapping| mapping.source.resolve(&context.root) == source)
        .cloned()
        .ok_or_else(|| {
            GripError::CorruptState("removed mapping has no portable declaration".into())
        })?;
    let portable = registry::ProjectDescriptorV2::new(
        declarations
            .into_iter()
            .filter(|mapping| mapping.source.resolve(&context.root) != source)
            .collect(),
    )?;
    let descriptor_bytes = registry::encode_descriptor(&portable)?;
    let state_snapshot = state::publication::load(&home)?;
    let mut next = state_snapshot.accepted.clone();
    next.complete_baselines
        .retain(|identity, _| identity.mapping.source != source);
    let state_bytes = state::publication::prepare_candidate_for_descriptor(
        &home,
        &state_snapshot,
        &next.complete_baselines,
        &descriptor_bytes,
    )?;
    let fence = state::add_fence::AddPublicationFenceV1::with_context(
        &removed_mapping,
        state::add_fence::FenceContext::removal(&removed_declaration),
        snapshot.bytes.clone(),
        descriptor_bytes.clone(),
        state_snapshot.bytes.clone(),
        state_bytes.clone(),
    );
    state::add_fence::create_verified(&home, &fence)?;
    registry::publication::publish_with_descriptor(&home, &snapshot, &candidate, &portable)?;
    state::publication::publish_prepared_locked(
        &home,
        &state_snapshot,
        &next.complete_baselines,
        &state_bytes,
    )?;
    let published_registry = registry::publication::load(&home, false)?;
    let published_state = state::publication::load(&home)?;
    if published_registry.bytes != descriptor_bytes
        || published_state.bytes.as_deref() != Some(state_bytes.as_slice())
    {
        return Err(GripError::CorruptState(
            "remove publication did not match its fenced candidate".into(),
        ));
    }
    state::add_fence::clear_verified(&home, &fence)?;
    Ok(CommandOutcome::mapping_success(
        "remove",
        "Mapping removed",
        &removed,
    ))
}

fn resume_fenced_remove(
    home: &project::ProjectPaths,
    source: &std::path::Path,
) -> Result<Option<CommandOutcome>, GripError> {
    let Some(fence) = state::add_fence::load(home)? else {
        return Ok(None);
    };
    if fence.operation != state::add_fence::FenceOperation::Remove {
        return Err(GripError::InvalidConfiguration(
            "another mapping has an incomplete add publication".into(),
        ));
    }
    if !fence.protects_selector(source, observation::model::PathSpace::Source) {
        return Err(GripError::InvalidConfiguration(
            "another mapping has an incomplete remove publication".into(),
        ));
    }
    let registry = registry::publication::load(home, false)?;
    let state_snapshot = state::publication::load(home)?;
    let descriptor_digest = state::add_fence::digest(&registry.bytes);
    let state_digest = state_snapshot
        .bytes
        .as_deref()
        .map(state::add_fence::digest);
    if descriptor_digest == fence.candidate_descriptor_digest {
        if state_digest != Some(fence.candidate_state_digest.clone()) {
            if state_digest != fence.prior_state_digest {
                return Err(GripError::InvalidConfiguration(
                    "incomplete remove publication state differs from both its prior and candidate state; retry cannot safely continue"
                        .into(),
                ));
            }
            let candidate_state = state::decode_v4(&fence.candidate_state_bytes)?;
            let runtime = state::runtime_from_accepted_v4(home, &candidate_state)?;
            state::publication::publish_prepared_locked(
                home,
                &state_snapshot,
                &runtime.baselines,
                &fence.candidate_state_bytes,
            )?;
        }
        let verified = state::publication::load(home)?;
        if verified.bytes.as_deref() != Some(fence.candidate_state_bytes.as_slice()) {
            return Err(GripError::CorruptState(
                "fenced remove retry did not publish the expected State V4 candidate".into(),
            ));
        }
        let removed = mapping_details_from_fence(fence.declaration.as_ref(), &fence.mapping)?;
        state::add_fence::clear_verified(home, &fence)?;
        return Ok(Some(CommandOutcome::mapping_success(
            "remove",
            "Mapping removed",
            &removed,
        )));
    }
    if descriptor_digest == fence.prior_descriptor_digest
        && state_digest == fence.prior_state_digest
    {
        state::add_fence::clear_verified(home, &fence)?;
        return Ok(None);
    }
    Err(GripError::InvalidConfiguration(
        "incomplete remove publication differs from both its prior and candidate state; retry cannot safely continue"
            .into(),
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
    let candidate = baseline::build_for_add(&state.accepted, &records)?;
    if candidate.changed_count > 0 {
        state::publication::publish_current_locked(home, &state, &candidate.next)?;
    }
    Ok(())
}

pub(crate) fn revalidate_project_for_mutation() -> Result<(), GripError> {
    SELECTED_PROJECT.with(|slot| match slot.borrow().as_ref() {
        Some(context) => context.revalidate(),
        None => Ok(()),
    })
}
