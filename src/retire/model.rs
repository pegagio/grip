//! Typed requests, plans, and outcomes for accepted-state retirement.

use crate::classification::model::{Classification, ClassificationScope};
use crate::discovery::model::SafePath;
use crate::mutation::model::{ActionStatus, BaselineOutcome, PlanBlocker};
use crate::observation::model::EntryIdentity;
use serde::Serialize;

/// Planner disposition for one selected accepted identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RetirementDisposition {
    Retire,
    ForceRequired,
    NoAction,
    Blocked,
}

/// One state-only accepted-record removal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RetirementAction {
    pub index: usize,
    #[serde(skip)]
    pub identity: EntryIdentity,
    pub path: SafePath,
    pub reason: String,
    pub force: bool,
    pub status: ActionStatus,
    pub milestones: crate::operation::model::ActionCheckpointEvidenceV1,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure: Option<String>,
}

/// Stable retirement decision for one selected entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RetirementEntryDisposition {
    #[serde(skip)]
    pub identity: EntryIdentity,
    pub classification: Classification,
    pub path: SafePath,
    pub disposition: RetirementDisposition,
    pub differences: Vec<String>,
    pub reasons: Vec<String>,
}

/// Complete deterministic state-only retirement plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RetirementPlan {
    pub operation: &'static str,
    pub plan_id: String,
    pub scope: ClassificationScope,
    pub force: bool,
    pub entries: Vec<RetirementEntryDisposition>,
    pub actions: Vec<RetirementAction>,
    pub retired_identities: Vec<SafePath>,
    pub blockers: Vec<PlanBlocker>,
}

/// Complete retirement result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RetirementResult {
    pub operation: &'static str,
    pub mode: String,
    pub completion: String,
    pub result: String,
    pub plan: RetirementPlan,
    pub operation_record: Option<String>,
    pub baseline: BaselineOutcome,
}
