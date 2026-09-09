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
            || record.source_complete != record.destination_complete
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
                || record.source_complete != record.destination_complete
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
