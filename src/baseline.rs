//! Explicit accepted-baseline construction and publication.

use crate::classification::model::{Classification, ClassificationRecord};
use crate::error::GripError;
use crate::state::AcceptedState;

#[derive(Debug, Clone)]
pub struct Candidate {
    pub next: AcceptedState,
    pub selected_count: usize,
    pub changed_count: usize,
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
            ) || record.source.is_none()
                || record.source != record.destination
                || record.blocking
        })
        .cloned()
        .collect::<Vec<_>>();
    if !rejected.is_empty() {
        return Err(GripError::BaselineNotAcceptable { records: rejected });
    }
    for record in records {
        let state = record
            .source
            .clone()
            .expect("eligible records have source state");
        if expected.baselines.get(&record.identity) != Some(&state) {
            next.baselines.insert(record.identity.clone(), state);
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
