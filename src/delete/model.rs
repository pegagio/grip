//! Typed requests, plans, and outcomes for authorized deletion.

use crate::classification::model::{Classification, ClassificationScope};
use crate::discovery::model::SafePath;
use crate::mutation::model::{ActionStatus, BaselineOutcome, PlanBlocker};
use crate::observation::model::{EntryIdentity, SupportedState};
use serde::{Deserialize, Serialize};

/// The side whose current absence the user accepts as authoritative.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeletionAuthority {
    Source,
    Destination,
}

impl DeletionAuthority {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Source => "source",
            Self::Destination => "destination",
        }
    }
}

/// Kind of descriptor-relative removal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DeletionActionKind {
    RemoveFile,
    RemoveDirectory,
}

/// Planner disposition for one selected identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DeletionDisposition {
    Action,
    NoAction,
    Blocked,
}

/// Independently observable destructive-action milestones.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DeletionEvidence {
    pub revalidation: String,
    pub recovery: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recovery_ref: Option<String>,
    pub removal: String,
    pub verification: String,
    pub durability_confirmed: bool,
}

impl Default for DeletionEvidence {
    fn default() -> Self {
        Self {
            revalidation: "not_attempted".into(),
            recovery: "planned".into(),
            recovery_ref: None,
            removal: "not_attempted".into(),
            verification: "not_attempted".into(),
            durability_confirmed: false,
        }
    }
}

/// One child-first authorized removal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DeletionAction {
    pub index: usize,
    pub kind: DeletionActionKind,
    #[serde(skip)]
    pub identity: EntryIdentity,
    pub authority: DeletionAuthority,
    pub target_side: String,
    #[serde(skip)]
    pub target: std::path::PathBuf,
    pub target_path: SafePath,
    pub expected_target: SupportedState,
    pub expected_absent_peer: String,
    pub expected_children: Vec<SafePath>,
    pub dependencies: Vec<usize>,
    pub status: ActionStatus,
    pub milestones: DeletionEvidence,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure: Option<String>,
}

/// Stable entry-level deletion decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DeletionEntryDisposition {
    #[serde(skip)]
    pub identity: EntryIdentity,
    pub classification: Classification,
    pub source_path: SafePath,
    pub destination_path: SafePath,
    pub disposition: DeletionDisposition,
    pub action_indexes: Vec<usize>,
    pub reasons: Vec<String>,
}

/// Counts projected into deletion results.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct DeletionCounts {
    pub selected: usize,
    pub actionable: usize,
    pub no_action: usize,
    pub blockers: usize,
    pub completed: usize,
    pub failed: usize,
    pub unattempted: usize,
}

/// Complete deterministic mode-neutral deletion plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DeletionPlan {
    pub operation: &'static str,
    pub authority: DeletionAuthority,
    pub plan_id: String,
    pub scope: ClassificationScope,
    pub entries: Vec<DeletionEntryDisposition>,
    pub actions: Vec<DeletionAction>,
    pub baseline_retirements: Vec<SafePath>,
    pub blockers: Vec<PlanBlocker>,
    pub counts: DeletionCounts,
}

/// Complete deletion execution result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DeletionResult {
    pub operation: &'static str,
    pub authority: DeletionAuthority,
    pub mode: String,
    pub completion: String,
    pub result: String,
    pub plan: DeletionPlan,
    pub operation_record: Option<String>,
    pub baseline: BaselineOutcome,
}
