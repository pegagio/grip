//! Public recovery inventory, restoration, and cleanup workflows.

pub mod cleanup;
pub mod inventory;
pub mod model;
pub mod restore;

/// Route public recovery commands without exposing private storage paths.
pub fn dispatch(
    home: &crate::home::GripHome,
    command: &crate::cli::RecoveryCommand,
) -> Result<crate::result::CommandOutcome, crate::error::GripError> {
    use crate::cli::RecoveryCommand;
    match command {
        RecoveryCommand::List => {
            let entries = inventory::list(home)?;
            let mut outcome = crate::result::CommandOutcome::success(format!(
                "Recovery inventory complete: {} entries",
                entries.len()
            ));
            outcome
                .details
                .insert("operation".into(), "recovery_list".into());
            outcome.details.insert(
                "entries".into(),
                serde_json::to_value(entries).expect("recovery entries serialize"),
            );
            Ok(outcome)
        }
        RecoveryCommand::Show(args) => {
            let entry = inventory::show(home, &args.reference)?;
            let mut outcome = crate::result::CommandOutcome::success("Recovery entry found");
            outcome
                .details
                .insert("operation".into(), "recovery_show".into());
            outcome.details.insert(
                "entry".into(),
                serde_json::to_value(entry).expect("recovery entry serializes"),
            );
            Ok(outcome)
        }
        RecoveryCommand::Restore(args) => {
            let plan = restore::plan(home, &args.reference)?;
            let applied = if args.dry_run || !plan.blockers.is_empty() {
                plan
            } else {
                restore::execute(home, &plan)?
            };
            recovery_outcome(
                "recovery_restore",
                if args.dry_run { "dry_run" } else { "execute" },
                &applied,
            )
        }
        RecoveryCommand::Remove(args) => {
            let plan = cleanup::plan(home, &args.references)?;
            let applied = if args.dry_run || !plan.blockers.is_empty() {
                plan
            } else {
                cleanup::execute(home, &plan)?
            };
            recovery_outcome(
                "recovery_remove",
                if args.dry_run { "dry_run" } else { "execute" },
                &applied,
            )
        }
    }
}

fn recovery_outcome<T: serde::Serialize>(
    operation: &str,
    mode: &str,
    plan: &T,
) -> Result<crate::result::CommandOutcome, crate::error::GripError> {
    let value = serde_json::to_value(plan).map_err(|error| {
        crate::error::GripError::Internal(format!("could not render recovery plan: {error}"))
    })?;
    let blocked = value
        .get("blockers")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|items| !items.is_empty());
    let mut outcome = if blocked {
        crate::result::CommandOutcome {
            category: crate::error::ResultCategory::InvalidConfiguration,
            message: format!("{operation} blocked"),
            details: serde_json::Map::new(),
        }
    } else {
        crate::result::CommandOutcome::success(format!("{operation} complete"))
    };
    outcome.details.insert("operation".into(), operation.into());
    outcome.details.insert("mode".into(), mode.into());
    outcome.details.insert("plan".into(), value);
    let operation_record = outcome.details["plan"]
        .get("operation_record")
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned);
    if let Some(record) = operation_record {
        outcome.details.insert(
            "operation_record".into(),
            serde_json::json!({"available":true,"id":record}),
        );
    }
    Ok(outcome)
}
