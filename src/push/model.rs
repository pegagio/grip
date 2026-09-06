//! Typed push requests, plans, actions, and results.

use crate::classification::model::{Classification, ClassificationScope};
use crate::discovery::model::SafePath;
use crate::observation::model::{EntryIdentity, SupportedState};
use serde::Serialize;

/// Whether a push is a preview or an execution request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PushMode {
    DryRun,
    Execute,
}

/// The fully resolved public request for one push.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PushRequest {
    pub mode: PushMode,
    pub destination_space: bool,
    pub selector: Option<std::path::PathBuf>,
}

/// The planner's outcome for one selected managed entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Disposition {
    Action,
    NoAction,
    Blocked,
}

/// A stable entry-level planning result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EntryDisposition {
    #[serde(skip)]
    pub identity: EntryIdentity,
    pub classification: Classification,
    pub source_path: SafePath,
    pub destination_path: SafePath,
    pub disposition: Disposition,
    pub action_indexes: Vec<usize>,
    pub reasons: Vec<String>,
}

/// A supported push side effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionKind {
    CreateParentDirectory,
    CreateDirectory,
    AddFile,
    ReplaceFile,
}

/// Last-known execution status for one action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionStatus {
    Unattempted,
    InProgress,
    Completed,
    Failed,
}

/// Independently observable action milestones.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ActionEvidence {
    pub revalidation: String,
    pub recovery: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recovery_ref: Option<String>,
    pub staging: String,
    pub publication: String,
    pub verification: String,
    pub durability_confirmed: bool,
}

impl Default for ActionEvidence {
    fn default() -> Self {
        Self {
            revalidation: "not_attempted".into(),
            recovery: "not_required".into(),
            recovery_ref: None,
            staging: "not_attempted".into(),
            publication: "not_attempted".into(),
            verification: "not_attempted".into(),
            durability_confirmed: false,
        }
    }
}

/// One dependency-ordered operation in a push plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PushAction {
    pub index: usize,
    pub kind: ActionKind,
    #[serde(skip)]
    pub identity: Option<EntryIdentity>,
    pub dependent_identities: Vec<SafePath>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_path: Option<SafePath>,
    #[serde(skip)]
    pub destination: std::path::PathBuf,
    pub destination_path: SafePath,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_source: Option<SupportedState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_destination: Option<SupportedState>,
    pub dependencies: Vec<usize>,
    pub status: ActionStatus,
    pub milestones: ActionEvidence,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure: Option<String>,
}

/// A stable preflight blocker.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PlanBlocker {
    pub reason: String,
    pub paths: Vec<SafePath>,
}

/// Counts projected into human and machine-readable results.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct PushCounts {
    pub selected: usize,
    #[serde(rename = "actions")]
    pub actionable: usize,
    pub no_action: usize,
    pub blockers: usize,
    pub completed: usize,
    pub failed: usize,
    pub unattempted: usize,
}

/// A complete, deterministic, mode-neutral push plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PushPlan {
    pub plan_id: String,
    pub scope: ClassificationScope,
    pub entries: Vec<EntryDisposition>,
    pub actions: Vec<PushAction>,
    pub blockers: Vec<PlanBlocker>,
    pub counts: PushCounts,
}

/// Accepted-state publication outcome for a push.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BaselineOutcome {
    pub outcome: String,
    pub prior_generation: Option<u64>,
    pub published_generation: Option<u64>,
    pub authoritative_generation: Option<u64>,
    pub publication_visible: bool,
    pub durability_confirmed: bool,
}

/// Complete typed result from push planning or execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PushResult {
    pub operation: &'static str,
    pub mode: PushMode,
    pub completion: String,
    pub result: String,
    pub scope: ClassificationScope,
    pub plan_id: String,
    pub counts: PushCounts,
    pub entries: Vec<EntryDisposition>,
    pub actions: Vec<PushAction>,
    pub blockers: Vec<PlanBlocker>,
    pub operation_record: Option<String>,
    pub baseline: BaselineOutcome,
}
