//! Explicit accepted-baseline construction and publication.

use crate::classification::model::{Classification, ClassificationRecord};
use crate::error::GripError;
use crate::state::AcceptedState;
use std::collections::BTreeSet;

#[derive(Debug, Clone)]
pub struct Candidate {
    pub next: AcceptedState,
    pub selected_count: usize,
    pub changed_count: usize,
}

/// Build a scoped accepted-state candidate from only verified actioned identities.
pub fn build_from_actioned(
    expected: &AcceptedState,
    records: &[ClassificationRecord],
    actioned: &BTreeSet<crate::observation::model::EntryIdentity>,
) -> Result<Candidate, GripError> {
    let mut next = expected.clone();
    let mut changed_count = 0;
    for identity in actioned {
        let record = records
            .iter()
            .find(|record| &record.identity == identity)
            .ok_or_else(|| {
                GripError::Internal("actioned identity missing from final observation".into())
            })?;
        if record.blocking
            || record.source_complete.is_none()
            || !record
                .source_complete
                .as_ref()
                .zip(record.destination_complete.as_ref())
                .is_some_and(|(source, destination)| {
                    crate::classification::complete_equivalent(source, destination)
                })
        {
            return Err(GripError::BaselineNotAcceptable {
                records: vec![record.clone()],
            });
        }
        let state = record
            .source_complete
            .clone()
            .expect("equivalent actioned entry has source");
        if expected.complete_baselines.get(identity) != Some(&state) {
            next.complete_baselines.insert(identity.clone(), state);
            changed_count += 1;
        }
    }
    Ok(Candidate {
        next,
        selected_count: records.len(),
        changed_count,
    })
}

pub fn build(
    expected: &AcceptedState,
    records: &[ClassificationRecord],
) -> Result<Candidate, GripError> {
    let mut next = expected.clone();
    let mut changed_count = 0;
    let rejected = records
        .iter()
        .filter(|record| {
            !matches!(
                record.classification,
                Classification::InitialMatch
                    | Classification::ConvergedTwoSidedChange
                    | Classification::Synchronized
                    | Classification::MetadataMigrationReady
            ) || record.source_complete.is_none()
                || !record
                    .source_complete
                    .as_ref()
                    .zip(record.destination_complete.as_ref())
                    .is_some_and(|(source, destination)| {
                        crate::classification::complete_equivalent(source, destination)
                    })
                || record.blocking
        })
        .cloned()
        .collect::<Vec<_>>();
    if !rejected.is_empty() {
        return Err(GripError::BaselineNotAcceptable { records: rejected });
    }
    for record in records {
        let state = record
            .source_complete
            .clone()
            .expect("eligible records have source state");
        if expected.complete_baselines.get(&record.identity) != Some(&state) {
            next.complete_baselines
                .insert(record.identity.clone(), state);
            changed_count += 1;
        }
    }
    Ok(Candidate {
        next,
        selected_count: records.len(),
        changed_count,
    })
}

/// Build the initial accepted state for a newly declared mapping without changing either
/// endpoint. Equal members retain the existing accepted-state rule, while an unequal
/// destination becomes the initial comparison reference so the source is ready to push.
pub fn build_for_add(
    expected: &AcceptedState,
    records: &[ClassificationRecord],
) -> Result<Candidate, GripError> {
    let mut next = expected.clone();
    let mut changed_count = 0;

    for record in records {
        // Before an initial reference exists, an unequal complete pair is represented by the
        // existing `InitialCollision` classification. It is safe for this add-only builder:
        // recording the destination converts it to the normal source-only change. All other
        // blocking records retain the ordinary conservative behavior.
        if record.blocking
            && !blocked_only_by_admitted_destination_link_ancestor(record, records)
            && !matches!(
                record.classification,
                Classification::InitialCollision | Classification::UnresolvedDestinationLink
            )
        {
            return Err(GripError::BaselineNotAcceptable {
                records: vec![record.clone()],
            });
        }

        let baseline = match (&record.source_complete, &record.destination_complete) {
            (Some(source), Some(destination))
                if crate::classification::complete_equivalent(source, destination) =>
            {
                Some(source)
            }
            (Some(_), Some(destination)) => Some(destination),
            // A source-only member is already a normal pending push. Destination-only
            // members are not source-defined ownership, so neither creates an entry.
            (Some(_), None) | (None, Some(_)) | (None, None) => None,
        };
        if let Some(baseline) = baseline
            && expected.complete_baselines.get(&record.identity) != Some(baseline)
        {
            next.complete_baselines
                .insert(record.identity.clone(), baseline.clone());
            changed_count += 1;
        }
    }

    Ok(Candidate {
        next,
        selected_count: records.len(),
        changed_count,
    })
}

fn blocked_only_by_admitted_destination_link_ancestor(
    record: &ClassificationRecord,
    records: &[ClassificationRecord],
) -> bool {
    record.blocking
        && !record.reasons.is_empty()
        && record
            .reasons
            .iter()
            .all(|reason| reason == "destination:symlink" || reason == "unsafe_collision")
        && records.iter().any(|ancestor| {
            ancestor.classification == Classification::UnresolvedDestinationLink
                && ancestor.identity.mapping == record.identity.mapping
                && strict_raw_descendant(
                    &record.identity.relative_path,
                    &ancestor.identity.relative_path,
                )
        })
}

fn strict_raw_descendant(candidate: &[u8], parent: &[u8]) -> bool {
    parent.is_empty()
        || candidate
            .strip_prefix(parent)
            .is_some_and(|suffix| suffix.first() == Some(&b'/'))
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct AcceptanceResult {
    pub operation: &'static str,
    pub completion: &'static str,
    pub result: &'static str,
    pub selected_count: usize,
    pub changed_count: usize,
    pub published: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation: Option<u64>,
}

impl AcceptanceResult {
    pub fn already_current(selected_count: usize, generation: Option<u64>) -> Self {
        Self {
            operation: "baseline_accept",
            completion: "complete",
            result: "already_current",
            selected_count,
            changed_count: 0,
            published: false,
            generation,
        }
    }

    pub fn accepted(selected_count: usize, changed_count: usize, generation: u64) -> Self {
        Self {
            operation: "baseline_accept",
            completion: "complete",
            result: "accepted",
            selected_count,
            changed_count,
            published: true,
            generation: Some(generation),
        }
    }
}
