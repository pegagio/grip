//! Typed direction-neutral mutation requests, plans, actions, and results.

use crate::classification::model::{Classification, ClassificationScope};
use crate::discovery::model::SafePath;
use crate::observation::model::{EntryIdentity, SupportedState};
use serde::Serialize;

/// Direction in which a mutation transfers complete supported state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MutationDirection {
    Push,
    Pull,
}

impl MutationDirection {
    pub fn operation(self) -> &'static str {
        match self {
            Self::Push => "push",
            Self::Pull => "pull",
        }
    }
}

/// Whether a mutation is a preview or an execution request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MutationMode {
    DryRun,
    Execute,
}

/// The fully resolved public request for one directional mutation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MutationRequest {
    pub mode: MutationMode,
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

/// A supported filesystem mutation.
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

/// One dependency-ordered operation in a directional mutation plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MutationAction {
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
pub struct MutationCounts {
    pub selected: usize,
    #[serde(rename = "actions")]
    pub actionable: usize,
    pub no_action: usize,
    pub blockers: usize,
    pub completed: usize,
    pub failed: usize,
    pub unattempted: usize,
}

/// A complete, deterministic, mode-neutral directional mutation plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MutationPlan {
    pub direction: MutationDirection,
    pub plan_id: String,
    pub scope: ClassificationScope,
    pub entries: Vec<EntryDisposition>,
    pub actions: Vec<MutationAction>,
    pub blockers: Vec<PlanBlocker>,
    pub counts: MutationCounts,
}

/// Accepted-state publication outcome for a directional mutation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BaselineOutcome {
    pub outcome: String,
    pub prior_generation: Option<u64>,
    pub published_generation: Option<u64>,
    pub authoritative_generation: Option<u64>,
    pub publication_visible: bool,
    pub durability_confirmed: bool,
}

/// Complete typed result from directional mutation planning or execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MutationResult {
    pub operation: &'static str,
    pub direction: MutationDirection,
    pub mode: MutationMode,
    pub completion: String,
    pub result: String,
    pub scope: ClassificationScope,
    pub plan_id: String,
    pub counts: MutationCounts,
    pub entries: Vec<EntryDisposition>,
    pub actions: Vec<MutationAction>,
    pub blockers: Vec<PlanBlocker>,
    pub operation_record: Option<String>,
    pub baseline: BaselineOutcome,
}

/// Compatibility alias for the Feature 005 push API.
pub type PushMode = MutationMode;
/// Compatibility alias for the Feature 005 push API.
pub type PushRequest = MutationRequest;
/// Compatibility alias for the Feature 005 push API.
pub type PushAction = MutationAction;
/// Compatibility alias for the Feature 005 push API.
pub type PushCounts = MutationCounts;
/// Compatibility alias for the Feature 005 push API.
pub type PushPlan = MutationPlan;
/// Compatibility alias for the Feature 005 push API.
pub type PushResult = MutationResult;
