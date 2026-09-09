//! Validation of project-local State V4 after a project root or destination home changes.

use crate::error::{GripError, ResultCategory};
use crate::project::ProjectPaths;
use crate::result::RebindingOutcome;
use crate::state::{AcceptedStateV4, current_binding, decode_v4_identity_path};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RebindingAssessment {
    pub outcome: RebindingOutcome,
    pub prior_root: Option<String>,
    pub blockers: Vec<String>,
}

pub fn assess(
    home: &ProjectPaths,
    state: &AcceptedStateV4,
) -> Result<RebindingAssessment, GripError> {
    let descriptor_bytes = std::fs::read(home.path().join("config.toml")).map_err(|error| {
        GripError::from_io("could not read descriptor for state binding", error)
    })?;
    let descriptor_text = std::str::from_utf8(&descriptor_bytes)
        .map_err(|_| GripError::InvalidConfiguration("project descriptor must be UTF-8".into()))?;
    let descriptor = crate::registry::decode_descriptor(descriptor_text)?;
    let current = current_binding(home, &descriptor_bytes, &descriptor)?;
    let retirement_identities = state
        .pending_retirements
        .iter()
        .map(|retirement| &retirement.identity)
        .collect::<BTreeSet<_>>();
    let location_changed = current.project_root != state.binding.project_root
        || current.user_home != state.binding.user_home;
    let mut blockers = Vec::new();
    for retirement in &state.pending_retirements {
        if descriptor.mappings().contains(&retirement.identity.mapping) {
            blockers.push("contradictory_pending_retirement".into());
        }
        if !state.baselines.contains_key(&retirement.identity) {
            blockers.push("missing_pending_retirement_baseline".into());
        }
    }
    if current == state.binding {
        blockers.sort();
        blockers.dedup();
        return Ok(RebindingAssessment {
            outcome: if blockers.is_empty() {
                RebindingOutcome::Bound
            } else {
                RebindingOutcome::RebindBlocked
            },
            prior_root: (!blockers.is_empty()).then(|| state.binding.project_root.clone()),
            blockers,
        });
    }
    for (identity, accepted) in &state.baselines {
        if !descriptor.mappings().contains(&identity.mapping)
            && !retirement_identities.contains(identity)
        {
            if location_changed {
                blockers.push("missing_mapping".into());
            }
            // A descriptor-only removal at the same root/home is an ordinary
            // pending-retirement transition. Retirement planning reobserves
            // both peers and applies its explicit force policy, so rebinding
            // must not preempt that stronger command-specific check.
            continue;
        }
        let relative = decode_v4_identity_path(&identity.relative_path_hex)?;
        let source = append_raw(
            &identity.mapping.source.resolve(home.project_root()?),
            &relative,
        );
        let destination_home = home.destination_home().ok_or_else(|| {
            GripError::InvalidConfiguration("project destination home is unavailable".into())
        })?;
        let destination = append_raw(
            &identity.mapping.destination.resolve(destination_home),
            &relative,
        );
        for (role, path) in [("source", source), ("destination", destination)] {
            match crate::observation::fingerprint::inspect_complete(&path, accepted.node_kind) {
                Ok(observed)
                    if observed.state == *accepted
                        && observed.unknown_xattrs.is_empty()
                        && observed.unsupported_bsd_flags.is_empty() => {}
                Ok(_) => blockers.push(format!("{role}_drift")),
                Err(_) => blockers.push(format!("{role}_unavailable")),
            }
        }
    }
    blockers.sort();
    blockers.dedup();
    Ok(RebindingAssessment {
        outcome: if blockers.is_empty() {
            RebindingOutcome::RebindEligible
        } else {
            RebindingOutcome::RebindBlocked
        },
        prior_root: Some(state.binding.project_root.clone()),
        blockers,
    })
}

pub fn require_mutation(assessment: &RebindingAssessment) -> Result<(), GripError> {
    if assessment.outcome == RebindingOutcome::RebindBlocked {
        return Err(GripError::lifecycle(
            "state_rebinding",
            "rebind_blocked",
            ResultCategory::InvalidConfiguration,
            format!(
                "project-local state cannot authorize mutation: {}",
                assessment.blockers.join(", ")
            ),
        ));
    }
    Ok(())
}

fn append_raw(root: &std::path::Path, relative: &[u8]) -> std::path::PathBuf {
    use std::os::unix::ffi::OsStringExt;
    let mut path = root.to_path_buf();
    if !relative.is_empty() {
        path.push(std::ffi::OsString::from_vec(relative.to_vec()));
    }
    path
}
