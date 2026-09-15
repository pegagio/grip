use crate::error::{GripError, ResultCategory};
use serde::Serialize;
use serde_json::{Map, Value};
use std::io::{self, Write};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RebindingOutcome {
    Uninitialized,
    Bound,
    RebindEligible,
    RebindBlocked,
}

#[derive(Debug, Clone, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectResultDetails {
    pub root: crate::discovery::model::SafePath,
    pub state: RebindingOutcome,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prior_root: Option<crate::discovery::model::SafePath>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DeclaredMappingDetails {
    pub kind: crate::mapping::MappingKind,
    pub source: String,
    pub destination: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResolvedMappingDetails {
    pub source: crate::discovery::model::SafePath,
    pub destination: crate::discovery::model::SafePath,
}

#[derive(Debug, Clone, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MappingResultDetails {
    pub declared: DeclaredMappingDetails,
    pub resolved: ResolvedMappingDetails,
}

impl From<&crate::mapping::ResolvedMapping> for MappingResultDetails {
    fn from(mapping: &crate::mapping::ResolvedMapping) -> Self {
        Self {
            declared: DeclaredMappingDetails {
                kind: mapping.declaration.kind,
                source: mapping.declaration.source.as_str().into(),
                destination: mapping.declaration.destination.as_str().into(),
            },
            resolved: ResolvedMappingDetails {
                source: crate::discovery::model::SafePath::from_path(&mapping.source),
                destination: crate::discovery::model::SafePath::from_path(&mapping.destination),
            },
        }
    }
}

impl MappingResultDetails {
    pub fn from_parts(
        declaration: &crate::mapping::PortableMapping,
        resolved: &crate::mapping::Mapping,
    ) -> Self {
        Self {
            declared: DeclaredMappingDetails {
                kind: declaration.kind,
                source: declaration.source.as_str().into(),
                destination: declaration.destination.as_str().into(),
            },
            resolved: ResolvedMappingDetails {
                source: crate::discovery::model::SafePath::from_path(&resolved.source),
                destination: crate::discovery::model::SafePath::from_path(&resolved.destination),
            },
        }
    }
}

/// Safe typed projection shared by metadata-aware human and JSON results.
#[derive(Debug, Clone, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MetadataResultDetails {
    pub metadata_dimensions: Vec<crate::metadata::model::MetadataDimension>,
    pub evidence: std::collections::BTreeMap<String, crate::metadata::model::Evidence<String>>,
    pub compatibility_findings: Vec<crate::metadata::model::CompatibilityFinding>,
    pub extended_attributes: Vec<crate::metadata::model::XattrFingerprint>,
    pub recovery_authority: Option<String>,
    pub verification: String,
    pub durability_confirmed: bool,
    pub accepted_generation: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    Human,
    Json,
}

/// Non-serialized next-step guidance for one human-visible conflict.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HumanConflictGuidance {
    ForcePair { source: String, destination: String },
    InspectDiff { source: String, destination: String },
}

#[derive(Debug, Clone)]
pub struct CommandOutcome {
    pub category: ResultCategory,
    pub message: String,
    pub details: Map<String, Value>,
    human_status_source_paths: Option<Vec<String>>,
    human_status_guidance: Option<Vec<Option<HumanConflictGuidance>>>,
    human_blocker_guidance: Vec<Option<HumanConflictGuidance>>,
    human_mapping_selected: bool,
}

impl CommandOutcome {
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            category: ResultCategory::Success,
            message: message.into(),
            details: Map::new(),
            human_status_source_paths: None,
            human_status_guidance: None,
            human_blocker_guidance: Vec::new(),
            human_mapping_selected: false,
        }
    }

    /// Build a parse or command-line usage failure before project execution begins.
    pub fn invalid_usage(message: impl Into<String>) -> Self {
        Self {
            category: ResultCategory::InvalidUsage,
            message: message.into(),
            details: Map::new(),
            human_status_source_paths: None,
            human_status_guidance: None,
            human_blocker_guidance: Vec::new(),
            human_mapping_selected: false,
        }
    }

    pub fn initialization(result: &crate::project::init::InitializationResult) -> Self {
        let initialization = match result.outcome {
            crate::project::init::InitializationOutcome::Initialized => "initialized",
            crate::project::init::InitializationOutcome::AlreadyInitialized => {
                "already_initialized"
            }
        };
        let root = crate::discovery::model::SafePath::from_path(&result.root);
        let mut outcome = Self::success(format!(
            "Grip project {initialization}: {}",
            result.root.display()
        ));
        outcome
            .details
            .insert("project".into(), serde_json::json!({"root": root}));
        outcome
            .details
            .insert("initialization".into(), initialization.into());
        outcome
    }

    pub fn with_project(mut self, context: &crate::project::ProjectContext) -> Self {
        self.details.insert(
            "project".into(),
            serde_json::json!({
                "root": crate::discovery::model::SafePath::from_path(&context.root)
            }),
        );
        self
    }

    pub fn with_project_state(
        mut self,
        context: &crate::project::ProjectContext,
        assessment: &crate::state::rebinding::RebindingAssessment,
    ) -> Self {
        self.details.insert(
            "project".into(),
            serde_json::json!({
                "root": crate::discovery::model::SafePath::from_path(&context.root),
                "state": assessment.outcome,
                "prior_root": assessment.prior_root.as_ref().map(|root| {
                    crate::discovery::model::SafePath::from_path(std::path::Path::new(root))
                }),
                "blockers": assessment.blockers,
            }),
        );
        self
    }

    /// Build a metadata-aware outcome without exposing raw extended-attribute values.
    pub fn metadata(
        category: ResultCategory,
        message: impl Into<String>,
        details: &MetadataResultDetails,
    ) -> Self {
        Self {
            category,
            message: message.into(),
            details: serde_json::to_value(details)
                .expect("typed metadata result serializes")
                .as_object()
                .expect("metadata result details serialize as an object")
                .clone(),
            human_status_source_paths: None,
            human_status_guidance: None,
            human_blocker_guidance: Vec::new(),
            human_mapping_selected: false,
        }
    }

    /// Build a stable result for deletion planning or execution.
    pub fn deletion(
        plan: &crate::delete::model::DeletionPlan,
        mode: &str,
        operation_record: Option<&str>,
        baseline: crate::mutation::model::BaselineOutcome,
    ) -> Self {
        let (category, completion, result) = if !plan.blockers.is_empty() {
            (ResultCategory::InvalidConfiguration, "blocked", "blocked")
        } else if plan.actions.is_empty() {
            (ResultCategory::Success, "complete", "no_op")
        } else if mode == "dry_run" {
            (ResultCategory::Success, "complete", "planned")
        } else {
            (ResultCategory::Success, "complete", "applied")
        };
        operation_outcome(
            category,
            format!(
                "Delete {result}: {} action(s), {} blocker(s)",
                plan.actions.len(),
                plan.blockers.len()
            ),
            plan,
            mode,
            completion,
            result,
            operation_record,
            &baseline,
        )
    }

    pub fn failure(error: &GripError) -> Self {
        let mut outcome = Self {
            category: error.category(),
            message: error.to_string(),
            details: Map::new(),
            human_status_source_paths: None,
            human_status_guidance: None,
            human_blocker_guidance: Vec::new(),
            human_mapping_selected: false,
        };
        if let GripError::Mapping {
            operation,
            reason,
            paths,
            kind,
            publication_visible,
            conflicts,
            ..
        } = error
        {
            outcome
                .details
                .insert("operation".into(), operation.clone().into());
            outcome
                .details
                .insert("reason".into(), reason.clone().into());
            if !paths.is_empty() {
                if operation == "mapping_inspect" {
                    let safe_paths = paths
                        .iter()
                        .map(|display| serde_json::json!({"display": display}))
                        .collect::<Vec<_>>();
                    outcome.details.insert("paths".into(), safe_paths.into());
                } else {
                    outcome.details.insert("paths".into(), paths.clone().into());
                }
            }
            if let Some(kind) = kind {
                outcome.details.insert(
                    "kind".into(),
                    serde_json::to_value(kind).expect("mapping kind serializes"),
                );
            }
            if *publication_visible {
                outcome
                    .details
                    .insert("publication_visible".into(), true.into());
                outcome
                    .details
                    .insert("durability_confirmed".into(), false.into());
            }
            if !conflicts.is_empty() {
                outcome.details.insert(
                    "conflicts".into(),
                    serde_json::to_value(conflicts).expect("ownership conflicts serialize"),
                );
            }
        }
        if let GripError::Lifecycle {
            operation,
            reason,
            publication_visible,
            verification,
            durability_confirmed,
            ..
        } = error
        {
            outcome
                .details
                .insert("operation".into(), operation.clone().into());
            outcome
                .details
                .insert("reason".into(), reason.clone().into());
            outcome
                .details
                .insert("publication_visible".into(), (*publication_visible).into());
            outcome
                .details
                .insert("verification".into(), verification.clone().into());
            outcome.details.insert(
                "durability_confirmed".into(),
                (*durability_confirmed).into(),
            );
        }
        if let GripError::OperationLifecycle {
            operation,
            reason,
            operation_id,
            details,
            publication_visible,
            verification,
            durability_confirmed,
            ..
        } = error
        {
            outcome.details.extend(details.as_ref().clone());
            outcome
                .details
                .insert("operation".into(), operation.clone().into());
            outcome
                .details
                .insert("reason".into(), reason.clone().into());
            outcome.details.insert(
                "operation_record".into(),
                serde_json::json!({"available":true,"id":operation_id}),
            );
            outcome
                .details
                .insert("publication_visible".into(), (*publication_visible).into());
            outcome
                .details
                .insert("verification".into(), verification.clone().into());
            outcome.details.insert(
                "durability_confirmed".into(),
                (*durability_confirmed).into(),
            );
        }
        if let GripError::BaselineNotAcceptable { records } = error {
            outcome
                .details
                .insert("operation".into(), "baseline_accept".into());
            outcome
                .details
                .insert("reason".into(), "baseline_not_acceptable".into());
            outcome.details.insert(
                "records".into(),
                serde_json::to_value(records).expect("classification records serialize"),
            );
        }
        if matches!(error, GripError::StateContention) {
            outcome
                .details
                .insert("operation".into(), "baseline_accept".into());
            outcome
                .details
                .insert("reason".into(), "state_contention".into());
        }
        if let GripError::MetadataContract { reason, paths, .. } = error {
            outcome.details.insert(
                "reason".into(),
                serde_json::to_value(reason).expect("metadata reason serializes"),
            );
            outcome.details.insert(
                "paths".into(),
                serde_json::to_value(paths).expect("safe metadata paths serialize"),
            );
            outcome.details.insert("blocking".into(), true.into());
        }
        if let GripError::MutationContention {
            requested_operation,
            owner,
        } = error
        {
            outcome.details.insert(
                "operation".into(),
                owner
                    .as_ref()
                    .map(|value| value.operation.clone())
                    .unwrap_or_else(|| "unknown".into())
                    .into(),
            );
            outcome
                .details
                .insert("reason".into(), "state_contention".into());
            outcome.details.insert(
                "requested_operation".into(),
                requested_operation.clone().into(),
            );
            if matches!(requested_operation.as_str(), "push" | "pull") {
                outcome
                    .details
                    .insert("direction".into(), requested_operation.clone().into());
            }
            if let Some(owner) = owner {
                outcome.details.insert(
                    "owner".into(),
                    serde_json::to_value(owner).expect("mutation lock owner serializes"),
                );
            }
        }
        if let GripError::PushFailed(failure) | GripError::MutationFailed(failure) = error {
            let plan = &failure.plan;
            let operation = plan.operation.as_str();
            outcome.message = format!(
                "{} failed after {} of {} actions",
                operation_title(operation),
                plan.counts.completed,
                plan.counts.actionable
            );
            outcome.details.insert("operation".into(), operation.into());
            if let Some(direction) = plan.direction {
                outcome
                    .details
                    .insert("direction".into(), direction.operation().into());
            }
            if let Some(winner) = plan.winner {
                outcome.details.insert("winner".into(), json(&winner));
            }
            outcome.details.insert("mode".into(), "execute".into());
            outcome
                .details
                .insert("completion".into(), failure.completion.clone().into());
            outcome.details.insert("result".into(), "failed".into());
            outcome.details.insert("scope".into(), json(&plan.scope));
            outcome
                .details
                .insert("plan_id".into(), plan_id(&plan.plan_id));
            outcome.details.insert("counts".into(), json(&plan.counts));
            outcome
                .details
                .insert("entries".into(), public_mutation_entries(&plan.entries));
            outcome
                .details
                .insert("actions".into(), public_mutation_actions(&plan.actions));
            outcome
                .details
                .insert("blockers".into(), json(&plan.blockers));
            outcome.details.insert(
                "failure".into(),
                serde_json::json!({
                    "reason": failure.reason,
                    "phase": failure.phase,
                    "category": failure.category.code(),
                    "paths": failure.paths,
                    "action_index": failure.failed_action_index,
                    "expected_source": failure.expected_source,
                    "expected_destination": failure.expected_destination,
                    "observed_source": failure.observed_source,
                    "observed_destination": failure.observed_destination,
                    "guidance": "inspect status and operation recovery evidence before retrying"
                }),
            );
            outcome.details.insert(
                "operation_record".into(),
                serde_json::json!({"available":true,"id":failure.operation_id}),
            );
            outcome
                .details
                .insert("baseline".into(), json(&failure.baseline));
        }
        outcome
    }

    pub fn mapping_success(operation: &str, message: &str, mapping: &MappingResultDetails) -> Self {
        let mut outcome = Self::success(message);
        outcome.details.insert("operation".into(), operation.into());
        outcome.details.insert(
            "mapping".into(),
            serde_json::to_value(mapping).expect("mapping serializes"),
        );
        outcome
    }

    /// Build the stable result for an explicit mapping replacement.
    pub fn mapping_replaced(
        mapping: &MappingResultDetails,
        replaced_mapping: &MappingResultDetails,
    ) -> Self {
        let mut outcome = Self::mapping_success("add", "Mapping replaced", mapping);
        outcome.details.insert(
            "replaced_mapping".into(),
            serde_json::to_value(replaced_mapping).expect("mapping serializes"),
        );
        outcome
    }

    pub fn mapping_list(mappings: &[MappingResultDetails]) -> Self {
        let mut outcome = Self::success(format!("{} mapping(s)", mappings.len()));
        outcome
            .details
            .insert("operation".into(), "mapping_list".into());
        outcome.details.insert(
            "mappings".into(),
            serde_json::to_value(mappings).expect("mappings serialize"),
        );
        outcome
    }

    /// Build the stable human/JSON result for a complete read-only inventory.
    pub fn discovery(inventory: &crate::discovery::model::DiscoveryInventory) -> Self {
        use crate::discovery::model::DiscoveryScope;
        let message = format!(
            "Inspected {} entries; {} blocking finding(s)",
            inventory.records.len(),
            inventory.blocking_count
        );
        let mut outcome = Self::success(message);
        outcome
            .details
            .insert("operation".into(), "mapping_inspect".into());
        let mut scope = Map::new();
        match &inventory.scope {
            DiscoveryScope::All => {
                scope.insert("kind".into(), "all".into());
            }
            DiscoveryScope::Mapping(source) => {
                scope.insert("kind".into(), "mapping".into());
                scope.insert("source".into(), source.display().to_string().into());
            }
        }
        outcome.details.insert("scope".into(), scope.into());
        outcome.details.insert(
            "counts".into(),
            serde_json::to_value(&inventory.counts).expect("discovery counts serialize"),
        );
        outcome
            .details
            .insert("blocking_count".into(), inventory.blocking_count.into());
        outcome.details.insert(
            "records".into(),
            serde_json::to_value(&inventory.records).expect("discovery records serialize"),
        );
        outcome
    }

    pub fn classification(result: &crate::classification::model::ClassificationResult) -> Self {
        let category = if result.operation == "check" && result.attention_count > 0 {
            ResultCategory::AttentionRequired
        } else {
            ResultCategory::Success
        };
        let mut value = serde_json::to_value(result).expect("classification result serializes");
        let mut capabilities = std::collections::BTreeMap::new();
        if let Some(records) = value.get_mut("records").and_then(Value::as_array_mut) {
            for record in records {
                let Some(record) = record.as_object_mut() else {
                    continue;
                };
                if record.get("classification").and_then(Value::as_str) == Some("synchronized") {
                    record.remove("source_complete");
                    record.remove("destination_complete");
                    record.remove("baseline_complete");
                }
                let mapping_source = record
                    .get("mapping_source")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown")
                    .to_owned();
                if let Some(profiles) = record.remove("endpoint_capabilities") {
                    capabilities.entry(mapping_source).or_insert(profiles);
                }
            }
        }
        let details = value
            .as_object_mut()
            .expect("classification result is an object")
            .clone();
        let mut details = details;
        details.insert(
            "endpoint_capabilities".into(),
            Value::Array(
                capabilities
                    .into_iter()
                    .map(|(mapping_source, profiles)| {
                        serde_json::json!({
                            "mapping_source": mapping_source,
                            "profiles": profiles,
                        })
                    })
                    .collect(),
            ),
        );
        Self {
            category,
            message: format!(
                "{} complete: {} entries; {} attention; {} blocking",
                title(&result.operation),
                result.records.len(),
                result.attention_count,
                result.blocking_count
            ),
            details,
            human_status_source_paths: None,
            human_status_guidance: None,
            human_blocker_guidance: Vec::new(),
            human_mapping_selected: false,
        }
    }

    /// Attach non-serialized source displays for default human status rendering.
    pub fn with_human_status_source_paths(mut self, source_paths: Vec<String>) -> Self {
        self.human_status_source_paths = Some(source_paths);
        self
    }

    /// Attach non-serialized next-step guidance for default human status rendering.
    pub fn with_human_status_guidance(
        mut self,
        guidance: Vec<Option<HumanConflictGuidance>>,
    ) -> Self {
        self.human_status_guidance = Some(guidance);
        self
    }

    /// Attach non-serialized next-step guidance for blocked mutation output.
    pub fn with_human_blocker_guidance(
        mut self,
        guidance: Vec<Option<HumanConflictGuidance>>,
    ) -> Self {
        self.human_blocker_guidance = guidance;
        self
    }

    /// Mark a mapping-list result as a source-selected human view without changing JSON details.
    pub fn with_human_mapping_selection(mut self, selected: bool) -> Self {
        self.human_mapping_selected = selected;
        self
    }

    pub fn baseline(result: &crate::baseline::AcceptanceResult) -> Self {
        let mut value = serde_json::to_value(result).expect("baseline result serializes");
        let details = value
            .as_object_mut()
            .expect("baseline result is an object")
            .clone();
        let message = if result.published {
            format!(
                "Accepted {} baseline entries; generation {}",
                result.changed_count,
                result.generation.expect("published result has generation")
            )
        } else {
            "Baseline already current; no state published".into()
        };
        Self {
            category: ResultCategory::Success,
            message,
            details,
            human_status_source_paths: None,
            human_status_guidance: None,
            human_blocker_guidance: Vec::new(),
            human_mapping_selected: false,
        }
    }

    /// Build the stable result for a complete push plan.
    pub fn push_plan(
        plan: &crate::push::model::PushPlan,
        mode: &str,
        generation: Option<u64>,
    ) -> Self {
        Self::mutation_plan(plan, mode, generation)
    }

    /// Build the stable result for a complete mutation plan.
    pub fn mutation_plan(
        plan: &crate::mutation::model::MutationPlan,
        mode: &str,
        generation: Option<u64>,
    ) -> Self {
        let operation = plan.operation.as_str();
        let title = operation_title(operation);
        let (category, completion, result, message) = if !plan.blockers.is_empty() {
            (
                ResultCategory::InvalidConfiguration,
                "blocked",
                "blocked",
                format!(
                    "{title} blocked: {} selected; {} action(s); {} blocker(s)",
                    plan.counts.selected, plan.counts.actionable, plan.counts.blockers
                ),
            )
        } else if plan.actions.is_empty() && plan.acceptance_identities.is_empty() {
            (
                ResultCategory::Success,
                "complete",
                "no_op",
                format!(
                    "{title} complete: {} selected; no actions",
                    plan.counts.selected
                ),
            )
        } else {
            (
                ResultCategory::Success,
                "complete",
                "planned",
                format!(
                    "{title} preview complete: {} selected; {} action(s); 0 blockers",
                    plan.counts.selected, plan.counts.actionable
                ),
            )
        };
        let mut details = Map::new();
        details.insert("operation".into(), operation.into());
        if let Some(direction) = plan.direction {
            details.insert("direction".into(), direction.operation().into());
        }
        if let Some(winner) = plan.winner {
            details.insert("winner".into(), json(&winner));
        }
        details.insert("mode".into(), mode.into());
        details.insert("completion".into(), completion.into());
        details.insert("result".into(), result.into());
        details.insert("scope".into(), json(&plan.scope));
        details.insert("plan_id".into(), plan_id(&plan.plan_id));
        details.insert("counts".into(), json(&plan.counts));
        details.insert("entries".into(), public_mutation_entries(&plan.entries));
        details.insert("actions".into(), public_mutation_actions(&plan.actions));
        details.insert("blockers".into(), json(&plan.blockers));
        details.insert("operation_record".into(), Value::Null);
        details.insert(
            "baseline".into(),
            serde_json::json!({
                "outcome": "not_attempted",
                "prior_generation": generation,
                "published_generation": null,
                "authoritative_generation": generation,
                "publication_visible": false,
                "durability_confirmed": true
            }),
        );
        Self {
            category,
            message,
            details,
            human_status_source_paths: None,
            human_status_guidance: None,
            human_blocker_guidance: Vec::new(),
            human_mapping_selected: false,
        }
    }

    /// Build the stable successful result for an accepted push execution.
    pub fn push_applied(success: &crate::push::execution::ExecutionSuccess) -> Self {
        Self::mutation_applied(success)
    }

    /// Build the stable successful result for an accepted mutation execution.
    pub fn mutation_applied(success: &crate::mutation::execution::ExecutionSuccess) -> Self {
        let plan = &success.plan;
        let operation = plan.operation.as_str();
        let title = operation_title(operation);
        let mut details = Map::new();
        details.insert("operation".into(), operation.into());
        if let Some(direction) = plan.direction {
            details.insert("direction".into(), direction.operation().into());
        }
        if let Some(winner) = plan.winner {
            details.insert("winner".into(), json(&winner));
        }
        details.insert("mode".into(), "execute".into());
        details.insert("completion".into(), "complete".into());
        details.insert("result".into(), "applied".into());
        details.insert("scope".into(), json(&plan.scope));
        details.insert("plan_id".into(), plan_id(&plan.plan_id));
        details.insert("counts".into(), json(&plan.counts));
        details.insert("entries".into(), public_mutation_entries(&plan.entries));
        details.insert("actions".into(), public_mutation_actions(&plan.actions));
        details.insert("blockers".into(), json(&plan.blockers));
        details.insert(
            "operation_record".into(),
            serde_json::json!({"available":true,"id":success.operation_id}),
        );
        details.insert(
            "baseline".into(),
            serde_json::json!({
                "outcome":"published",
                "prior_generation":success.prior_generation,
                "published_generation":success.generation,
                "authoritative_generation":success.generation,
                "publication_visible":true,
                "durability_confirmed":true
            }),
        );
        Self {
            category: ResultCategory::Success,
            message: format!(
                "{title} applied: {} action(s); accepted generation {}",
                plan.counts.completed, success.generation
            ),
            details,
            human_status_source_paths: None,
            human_status_guidance: None,
            human_blocker_guidance: Vec::new(),
            human_mapping_selected: false,
        }
    }
}

fn json<T: Serialize + ?Sized>(value: &T) -> Value {
    serde_json::to_value(value).expect("typed result value serializes")
}

fn public_mutation_actions(actions: &[crate::mutation::model::MutationAction]) -> Value {
    let mut value = json(actions);
    if let Some(actions) = value.as_array_mut() {
        for action in actions {
            let Some(action) = action.as_object_mut() else {
                continue;
            };
            if action.get("metadata").is_some() {
                action.remove("expected_source");
                action.remove("expected_destination");
            }
        }
    }
    value
}

fn public_mutation_entries(entries: &[crate::mutation::model::EntryDisposition]) -> Value {
    let reportable = entries
        .iter()
        .filter(|entry| {
            entry.disposition != crate::mutation::model::Disposition::NoAction
                || entry.classification
                    != crate::classification::model::Classification::Synchronized
        })
        .collect::<Vec<_>>();
    json(&reportable)
}

fn plan_id(digest: &str) -> Value {
    serde_json::json!({"algorithm":"sha256","digest":digest})
}

fn title(operation: &str) -> &str {
    match operation {
        "status" => "Status",
        "check" => "Check",
        "diff" => "Diff",
        _ => "Inspection",
    }
}

fn operation_title(operation: &str) -> &str {
    match operation {
        "push" => "Push",
        "pull" => "Pull",
        "sync" => "Sync",
        "resolve" => "Resolution",
        _ => "Mutation",
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ResultEnvelopeV1 {
    pub schema_version: u8,
    pub status: &'static str,
    pub code: &'static str,
    pub message: String,
    pub details: Map<String, Value>,
}

impl From<CommandOutcome> for ResultEnvelopeV1 {
    fn from(value: CommandOutcome) -> Self {
        Self {
            schema_version: 1,
            status: value.category.status(),
            code: value.category.code(),
            message: value.message,
            details: value.details,
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn operation_outcome<T: Serialize, B: Serialize>(
    category: ResultCategory,
    message: String,
    plan: &T,
    mode: &str,
    completion: &str,
    result: &str,
    operation_record: Option<&str>,
    baseline: &B,
) -> CommandOutcome {
    let mut details = serde_json::to_value(plan)
        .expect("typed operation plan serializes")
        .as_object()
        .expect("typed operation plan is an object")
        .clone();
    details.insert("mode".into(), mode.into());
    details.insert("completion".into(), completion.into());
    details.insert("result".into(), result.into());
    details.insert(
        "operation_record".into(),
        operation_record.map_or(
            Value::Null,
            |value| serde_json::json!({"available":true,"id":value}),
        ),
    );
    details.insert("baseline".into(), json(baseline));
    CommandOutcome {
        category,
        message,
        details,
        human_status_source_paths: None,
        human_status_guidance: None,
        human_blocker_guidance: Vec::new(),
        human_mapping_selected: false,
    }
}

pub fn render(outcome: CommandOutcome, mode: OutputMode, writer: &mut dyn Write) -> io::Result<()> {
    match mode {
        OutputMode::Human => {
            if outcome.details.get("operation").and_then(Value::as_str) == Some("status") {
                return render_human_status(
                    &outcome.details,
                    outcome.human_status_source_paths.as_deref(),
                    outcome.human_status_guidance.as_deref(),
                    writer,
                );
            }
            if render_human_mutation(&outcome, writer)? {
                return Ok(());
            }
            if render_human_concise_mapping(&outcome, writer)? {
                return Ok(());
            }
            let message = if outcome.category != ResultCategory::Success
                && !outcome.message.starts_with("Error:")
            {
                format!("Error: {}", outcome.message)
            } else {
                outcome.message.clone()
            };
            writeln!(writer, "{message}")?;
            render_human_metadata_details(&outcome.details, writer)?;
            if let Some(conflicts) = outcome.details.get("conflicts").and_then(Value::as_array) {
                for conflict in conflicts {
                    writeln!(
                        writer,
                        "{}: {} {} {}",
                        conflict
                            .get("reason")
                            .and_then(Value::as_str)
                            .unwrap_or("ownership_conflict"),
                        conflict
                            .get("first_path")
                            .and_then(Value::as_str)
                            .unwrap_or("unknown"),
                        conflict
                            .get("relation")
                            .and_then(Value::as_str)
                            .unwrap_or("conflicts_with"),
                        conflict
                            .get("second_path")
                            .and_then(Value::as_str)
                            .unwrap_or("unknown")
                    )?;
                }
            }
            if outcome.details.get("operation").and_then(Value::as_str) == Some("mapping_inspect")
                && let Some(records) = outcome.details.get("records").and_then(Value::as_array)
            {
                for record in records {
                    render_human_discovery_record(record, writer)?;
                }
            }
            if matches!(
                outcome.details.get("operation").and_then(Value::as_str),
                Some("status" | "check" | "diff")
            ) && let Some(records) = outcome.details.get("records").and_then(Value::as_array)
            {
                for record in records {
                    render_human_classification_record(record, writer)?;
                }
            }
            if matches!(
                outcome.details.get("operation").and_then(Value::as_str),
                Some("push" | "pull" | "sync" | "resolve")
            ) {
                if let Some(winner) = outcome.details.get("winner").and_then(Value::as_str) {
                    writeln!(writer, "Winner {winner}")?;
                }
                if let Some(actions) = outcome.details.get("actions").and_then(Value::as_array) {
                    for action in actions {
                        let status = action
                            .get("status")
                            .and_then(Value::as_str)
                            .unwrap_or("planned");
                        let status = if status == "unattempted"
                            && outcome.details.get("mode").and_then(Value::as_str)
                                == Some("dry_run")
                        {
                            "planned"
                        } else {
                            status
                        };
                        let source = action
                            .get("source_path")
                            .and_then(|path| path.get("display"))
                            .and_then(Value::as_str);
                        let destination = action
                            .get("destination_path")
                            .and_then(|path| path.get("display"))
                            .and_then(Value::as_str)
                            .unwrap_or("unknown");
                        let milestones = action.get("milestones").unwrap_or(&Value::Null);
                        let failure = action
                            .get("failure")
                            .and_then(Value::as_str)
                            .map(|reason| format!(" reason={reason}"))
                            .unwrap_or_default();
                        let attempted = matches!(status, "completed" | "failed" | "in_progress");
                        let evidence = if attempted {
                            format!(
                                " visible={} verified={} durable={}",
                                if milestones.get("publication").and_then(Value::as_str)
                                    == Some("visible")
                                {
                                    "yes"
                                } else {
                                    "no"
                                },
                                if milestones.get("verification").and_then(Value::as_str)
                                    == Some("verified")
                                {
                                    "yes"
                                } else {
                                    "no"
                                },
                                if milestones
                                    .get("durability_confirmed")
                                    .and_then(Value::as_bool)
                                    .unwrap_or(false)
                                {
                                    "yes"
                                } else {
                                    "no"
                                },
                            )
                        } else {
                            String::new()
                        };
                        let pull = action.get("direction").and_then(Value::as_str) == Some("pull");
                        let (origin, target) = if pull {
                            (Some(destination), source.unwrap_or(destination))
                        } else {
                            (source, destination)
                        };
                        writeln!(
                            writer,
                            "{} {} {}{}{}{}{} recovery={}",
                            status,
                            action
                                .get("kind")
                                .and_then(Value::as_str)
                                .unwrap_or("action"),
                            origin.unwrap_or(target),
                            if origin.is_some() { " -> " } else { "" },
                            if origin.is_some() { target } else { "" },
                            failure,
                            evidence,
                            milestones
                                .get("recovery")
                                .and_then(Value::as_str)
                                .unwrap_or("not_required")
                        )?;
                    }
                }
                if let Some(blockers) = outcome.details.get("blockers").and_then(Value::as_array) {
                    for (index, blocker) in blockers.iter().enumerate() {
                        writeln!(
                            writer,
                            "blocked {}",
                            blocker
                                .get("reason")
                                .and_then(Value::as_str)
                                .unwrap_or("blocking_evidence")
                        )?;
                        if let Some(Some(guidance)) = outcome.human_blocker_guidance.get(index) {
                            render_human_conflict_guidance(guidance, "  ", writer)?;
                        }
                    }
                }
                let human_blocked_directional_mutation =
                    matches!(
                        outcome.details.get("operation").and_then(Value::as_str),
                        Some("push" | "pull")
                    ) && outcome.details.get("result").and_then(Value::as_str) == Some("blocked");
                if !human_blocked_directional_mutation
                    && let Some(baseline) = outcome.details.get("baseline")
                {
                    let baseline_outcome = baseline
                        .get("outcome")
                        .and_then(Value::as_str)
                        .unwrap_or("unknown")
                        .replace('_', " ");
                    writeln!(
                        writer,
                        "Baseline {}; generation {} remains authoritative",
                        baseline_outcome,
                        baseline
                            .get("authoritative_generation")
                            .map(Value::to_string)
                            .unwrap_or_else(|| "none".into())
                    )?;
                }
                if let Some(record) = outcome.details.get("operation_record")
                    && record.get("available").and_then(Value::as_bool) == Some(true)
                {
                    writeln!(
                        writer,
                        "Operation record {} preserved",
                        record
                            .get("id")
                            .and_then(Value::as_str)
                            .unwrap_or("unknown")
                    )?;
                }
            }
            if matches!(
                outcome.details.get("operation").and_then(Value::as_str),
                Some("delete" | "retire")
            ) {
                if let Some(authority) = outcome.details.get("authority").and_then(Value::as_str) {
                    writeln!(writer, "Authority {authority}")?;
                }
                if let Some(force) = outcome.details.get("force").and_then(Value::as_bool) {
                    writeln!(
                        writer,
                        "Force authorized {}",
                        if force { "yes" } else { "no" }
                    )?;
                }
                if let Some(actions) = outcome.details.get("actions").and_then(Value::as_array) {
                    for action in actions {
                        let path = action
                            .get("target_path")
                            .or_else(|| action.get("path"))
                            .and_then(|path| path.get("display"))
                            .and_then(Value::as_str)
                            .unwrap_or("unknown");
                        writeln!(
                            writer,
                            "{} {}",
                            action
                                .get("status")
                                .and_then(Value::as_str)
                                .unwrap_or("planned"),
                            path
                        )?;
                    }
                }
            }
            if matches!(
                outcome.details.get("operation").and_then(Value::as_str),
                Some("recovery_list")
            ) && let Some(entries) = outcome.details.get("entries").and_then(Value::as_array)
            {
                for entry in entries {
                    writeln!(
                        writer,
                        "{} {} {}",
                        entry
                            .get("kind")
                            .and_then(Value::as_str)
                            .unwrap_or("unknown"),
                        entry
                            .get("availability")
                            .and_then(Value::as_str)
                            .unwrap_or("unknown"),
                        recovery_reference(entry.get("reference").unwrap_or(&Value::Null))
                    )?;
                }
            }
            if matches!(
                outcome.details.get("operation").and_then(Value::as_str),
                Some("recovery_show")
            ) && let Some(entry) = outcome.details.get("entry")
            {
                writeln!(
                    writer,
                    "{} {} {}",
                    entry
                        .get("kind")
                        .and_then(Value::as_str)
                        .unwrap_or("unknown"),
                    entry
                        .get("availability")
                        .and_then(Value::as_str)
                        .unwrap_or("unknown"),
                    recovery_reference(entry.get("reference").unwrap_or(&Value::Null))
                )?;
            }
            if matches!(
                outcome.details.get("operation").and_then(Value::as_str),
                Some("recovery_restore" | "recovery_remove")
            ) && let Some(plan) = outcome.details.get("plan")
                && let Some(actions) = plan.get("actions").and_then(Value::as_array)
            {
                for action in actions {
                    writeln!(
                        writer,
                        "{} {}",
                        action
                            .get("status")
                            .and_then(Value::as_str)
                            .unwrap_or("planned"),
                        recovery_reference(action.get("reference").unwrap_or(&Value::Null))
                    )?;
                }
            }
            if matches!(
                outcome.details.get("operation").and_then(Value::as_str),
                Some("delete" | "retire" | "recovery_restore" | "recovery_remove")
            ) {
                if let Some(baseline) = outcome.details.get("baseline") {
                    writeln!(
                        writer,
                        "Accepted state {} generation={}",
                        baseline
                            .get("outcome")
                            .and_then(Value::as_str)
                            .unwrap_or("unknown"),
                        baseline
                            .get("authoritative_generation")
                            .map(Value::to_string)
                            .unwrap_or_else(|| "none".into())
                    )?;
                }
                if let Some(visible) = outcome
                    .details
                    .get("publication_visible")
                    .and_then(Value::as_bool)
                {
                    writeln!(
                        writer,
                        "Visible {} verified={} durable={}",
                        if visible { "yes" } else { "no" },
                        outcome
                            .details
                            .get("verification")
                            .and_then(Value::as_str)
                            .unwrap_or("unknown"),
                        if outcome
                            .details
                            .get("durability_confirmed")
                            .and_then(Value::as_bool)
                            .unwrap_or(false)
                        {
                            "yes"
                        } else {
                            "no"
                        }
                    )?;
                }
                if let Some(record) = outcome.details.get("operation_record")
                    && record.get("available").and_then(Value::as_bool) == Some(true)
                {
                    writeln!(
                        writer,
                        "Operation record {} preserved",
                        record
                            .get("id")
                            .and_then(Value::as_str)
                            .unwrap_or("unknown")
                    )?;
                }
            }
            Ok(())
        }
        OutputMode::Json => {
            serde_json::to_writer(&mut *writer, &ResultEnvelopeV1::from(outcome))?;
            writeln!(writer)
        }
    }
}

fn render_human_mutation(outcome: &CommandOutcome, writer: &mut dyn Write) -> io::Result<bool> {
    let operation = outcome.details.get("operation").and_then(Value::as_str);
    let Some(result) = outcome.details.get("result").and_then(Value::as_str) else {
        return Ok(false);
    };
    let Some(direction) = human_mutation_direction(operation, &outcome.details) else {
        return Ok(false);
    };

    match result {
        "no_op" => {
            writeln!(writer, "Nothing to {direction}.")?;
            Ok(true)
        }
        "blocked" => render_human_blocked_mutation(outcome, direction, writer),
        "failed" => {
            let counts = outcome.details.get("counts").unwrap_or(&Value::Null);
            writeln!(
                writer,
                "Error: {} failed after {} of {} actions. Run: grip status before retrying.",
                human_mutation_title(direction),
                counts.get("completed").and_then(Value::as_u64).unwrap_or(0),
                counts.get("actions").and_then(Value::as_u64).unwrap_or(0),
            )?;
            Ok(true)
        }
        "planned" | "applied" if outcome.category == ResultCategory::Success => {
            render_human_successful_mutation(outcome, direction, result == "applied", writer)
        }
        _ => Ok(false),
    }
}

fn human_mutation_direction(
    operation: Option<&str>,
    details: &Map<String, Value>,
) -> Option<&'static str> {
    match operation {
        Some("push") => Some("push"),
        Some("pull") => Some("pull"),
        Some("sync") => Some("synchronize"),
        Some("resolve") => match details.get("winner").and_then(Value::as_str) {
            Some("source") => Some("push"),
            Some("destination") => Some("pull"),
            _ => None,
        },
        _ => None,
    }
}

fn human_mutation_title(direction: &str) -> &'static str {
    match direction {
        "push" => "Push",
        "pull" => "Pull",
        "synchronize" => "Sync",
        _ => "Mutation",
    }
}

fn render_human_blocked_mutation(
    outcome: &CommandOutcome,
    direction: &str,
    writer: &mut dyn Write,
) -> io::Result<bool> {
    let counts = outcome.details.get("counts").unwrap_or(&Value::Null);
    writeln!(
        writer,
        "Error: {} blocked: {} selected; {} action(s); {} blocker(s)",
        human_mutation_title(direction),
        counts.get("selected").and_then(Value::as_u64).unwrap_or(0),
        counts.get("actions").and_then(Value::as_u64).unwrap_or(0),
        counts.get("blockers").and_then(Value::as_u64).unwrap_or(0),
    )?;
    if outcome.human_blocker_guidance.is_empty() {
        writeln!(writer, "  Grip cannot safely continue. Run: grip status")?;
        return Ok(true);
    }
    let mut needs_status = false;
    for guidance in &outcome.human_blocker_guidance {
        let Some(guidance) = guidance else {
            needs_status = true;
            continue;
        };
        let (source, destination) = match guidance {
            HumanConflictGuidance::ForcePair {
                source,
                destination,
            }
            | HumanConflictGuidance::InspectDiff {
                source,
                destination,
            } => (source, destination),
        };
        writeln!(writer, "  {source} <-> {destination}")?;
        render_human_conflict_guidance(guidance, "    ", writer)?;
    }
    if needs_status {
        writeln!(writer, "  Grip cannot safely continue. Run: grip status")?;
    }
    Ok(true)
}

fn render_human_successful_mutation(
    outcome: &CommandOutcome,
    direction: &str,
    completed: bool,
    writer: &mut dyn Write,
) -> io::Result<bool> {
    let Some(actions) = outcome.details.get("actions").and_then(Value::as_array) else {
        return Ok(false);
    };
    if actions.is_empty() {
        let accepted_entries = outcome
            .details
            .get("counts")
            .and_then(|counts| counts.get("converged"))
            .and_then(Value::as_u64)
            .unwrap_or(0);
        if accepted_entries == 0 {
            return Ok(false);
        }
        let verb = if completed {
            "Established"
        } else {
            "Would establish"
        };
        writeln!(writer, "{verb} a baseline for {accepted_entries} file(s).")?;
        return Ok(true);
    }

    let verb = match direction {
        "push" => {
            if completed {
                "Pushed"
            } else {
                "Would push"
            }
        }
        "pull" => {
            if completed {
                "Pulled"
            } else {
                "Would pull"
            }
        }
        "synchronize" => {
            if completed {
                "Synchronized"
            } else {
                "Would synchronize"
            }
        }
        _ => return Ok(false),
    };
    writeln!(writer, "{verb} {} file(s):", actions.len())?;
    for action in actions {
        let source = action
            .get("source_path")
            .and_then(|path| path.get("display"))
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let destination = action
            .get("destination_path")
            .and_then(|path| path.get("display"))
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let symbol = if action.get("direction").and_then(Value::as_str) == Some("pull") {
            "<-"
        } else {
            "->"
        };
        writeln!(writer, "  {source} {symbol} {destination}")?;
    }
    Ok(true)
}

fn render_human_concise_mapping(
    outcome: &CommandOutcome,
    writer: &mut dyn Write,
) -> io::Result<bool> {
    let operation = outcome.details.get("operation").and_then(Value::as_str);
    match operation {
        Some("add") => {
            if let Some(replaced) = outcome.details.get("replaced_mapping") {
                writeln!(writer, "Mapping replaced:")?;
                render_human_named_mapping(replaced, "old", writer)?;
                if let Some(mapping) = outcome.details.get("mapping") {
                    render_human_named_mapping(mapping, "new", writer)?;
                }
            } else {
                writeln!(writer, "Mapped:")?;
                if let Some(mapping) = outcome.details.get("mapping") {
                    render_human_mapping(mapping, false, writer)?;
                }
            }
            Ok(true)
        }
        Some("remove") => {
            writeln!(writer, "Mapping removed:")?;
            if let Some(mapping) = outcome.details.get("mapping") {
                render_human_mapping(mapping, true, writer)?;
            }
            Ok(true)
        }
        Some("mapping_list") => {
            let mappings = outcome
                .details
                .get("mappings")
                .and_then(Value::as_array)
                .map(Vec::as_slice)
                .unwrap_or_default();
            writeln!(writer, "{} mapping(s):", mappings.len())?;
            for mapping in mappings {
                render_human_mapping(mapping, outcome.human_mapping_selected, writer)?;
            }
            Ok(true)
        }
        _ => Ok(false),
    }
}

fn render_human_status(
    details: &Map<String, Value>,
    source_paths: Option<&[String]>,
    guidance: Option<&[Option<HumanConflictGuidance>]>,
    writer: &mut dyn Write,
) -> io::Result<()> {
    let records = details
        .get("records")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default();
    if records.is_empty() {
        return writeln!(writer, "Status: no managed entries found.");
    }

    let mut current = 0;
    let mut conflicts = Vec::new();
    let mut push = Vec::new();
    let mut pull = Vec::new();
    let mut needs_baseline = Vec::new();
    for (index, record) in records.iter().enumerate() {
        if !record
            .get("attention")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        {
            current += 1;
        } else if record
            .get("blocking")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        {
            conflicts.push((index, record));
        } else {
            match record.get("prospective_direction").and_then(Value::as_str) {
                Some("source_to_destination") => push.push((index, record)),
                Some("destination_to_source") => pull.push((index, record)),
                _ => needs_baseline.push((index, record)),
            }
        }
    }

    write!(
        writer,
        "Status: {} {} checked; {current} current",
        records.len(),
        singular_or_plural(records.len(), "entry", "entries")
    )?;
    if conflicts.is_empty() && push.is_empty() && pull.is_empty() && needs_baseline.is_empty() {
        writeln!(writer, "; no action needed.")?;
        return Ok(());
    }
    if !push.is_empty() {
        write!(writer, "; {} to push", push.len())?;
    }
    if !pull.is_empty() {
        write!(writer, "; {} to pull", pull.len())?;
    }
    if !conflicts.is_empty() {
        write!(
            writer,
            "; {} {}",
            conflicts.len(),
            singular_or_plural(conflicts.len(), "conflict", "conflicts")
        )?;
    }
    if !needs_baseline.is_empty() {
        write!(writer, "; {} needs baseline", needs_baseline.len())?;
    }
    writeln!(writer, ".")?;

    render_human_status_section(
        "Changes to push",
        "->",
        &push,
        source_paths,
        guidance,
        writer,
    )?;
    render_human_status_section(
        "Changes to pull",
        "<-",
        &pull,
        source_paths,
        guidance,
        writer,
    )?;
    render_human_status_section(
        "Conflicts",
        "<->",
        &conflicts,
        source_paths,
        guidance,
        writer,
    )?;
    render_human_status_section(
        "Needs baseline",
        ">-<",
        &needs_baseline,
        source_paths,
        guidance,
        writer,
    )
}

fn render_human_status_section(
    title: &str,
    symbol: &str,
    records: &[(usize, &Value)],
    source_paths: Option<&[String]>,
    guidance: Option<&[Option<HumanConflictGuidance>]>,
    writer: &mut dyn Write,
) -> io::Result<()> {
    if records.is_empty() {
        return Ok(());
    }
    writeln!(writer, "\n{title}:")?;
    for (index, record) in records {
        writeln!(
            writer,
            "  {} {symbol} {}",
            source_paths
                .and_then(|paths| paths.get(*index))
                .map(String::as_str)
                .unwrap_or_else(|| classification_path(record, "source_path")),
            classification_path(record, "destination_path")
        )?;
        let source = source_paths
            .and_then(|paths| paths.get(*index))
            .map(String::as_str)
            .unwrap_or_else(|| classification_path(record, "source_path"));
        let destination = classification_path(record, "destination_path");
        render_human_status_blocker(
            record,
            source,
            destination,
            guidance
                .and_then(|guidance| guidance.get(*index))
                .and_then(Option::as_ref),
            writer,
        )?;
    }
    Ok(())
}

fn render_human_status_blocker(
    record: &Value,
    _source: &str,
    _destination: &str,
    guidance: Option<&HumanConflictGuidance>,
    writer: &mut dyn Write,
) -> io::Result<()> {
    if let Some(guidance) = guidance {
        return render_human_conflict_guidance(guidance, "    ", writer);
    }
    let Some(findings) = record
        .get("compatibility_findings")
        .and_then(Value::as_array)
    else {
        return if record
            .get("blocking")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        {
            writeln!(
                writer,
                "    Blocked: {}",
                classification_blocker_message(record)
            )
        } else {
            Ok(())
        };
    };
    let mut shown = false;
    for finding in findings.iter().filter(|finding| {
        finding
            .get("blocking")
            .and_then(Value::as_bool)
            .unwrap_or(false)
    }) {
        let message = finding
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or_else(|| classification_blocker_message(record));
        let corrective_choice = finding
            .get("corrective_choice")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty());
        write!(writer, "    Blocked: {message}")?;
        if let Some(corrective_choice) = corrective_choice {
            write!(writer, " {corrective_choice}")?;
        }
        writeln!(writer)?;
        shown = true;
    }
    if !shown
        && record
            .get("blocking")
            .and_then(Value::as_bool)
            .unwrap_or(false)
    {
        writeln!(
            writer,
            "    Blocked: {}",
            classification_blocker_message(record)
        )?;
    }
    Ok(())
}

fn render_human_force_resolution(
    source: &str,
    destination: &str,
    indent: &str,
    writer: &mut dyn Write,
) -> io::Result<()> {
    writeln!(writer, "{indent}Keep source: grip push --force {source}")?;
    writeln!(
        writer,
        "{indent}Keep destination: grip pull --force --destination {destination}"
    )
}

fn render_human_conflict_guidance(
    guidance: &HumanConflictGuidance,
    indent: &str,
    writer: &mut dyn Write,
) -> io::Result<()> {
    match guidance {
        HumanConflictGuidance::ForcePair {
            source,
            destination,
        } => render_human_force_resolution(source, destination, indent, writer),
        HumanConflictGuidance::InspectDiff { source, .. } => {
            writeln!(writer, "{indent}Run: grip diff {source}")
        }
    }
}

fn classification_blocker_message(record: &Value) -> &'static str {
    if record
        .get("reasons")
        .and_then(Value::as_array)
        .is_some_and(|reasons| {
            reasons
                .iter()
                .any(|reason| reason == "incomplete_add_publication")
        })
    {
        return "Grip could not finish recording this mapping. Rerun grip add for this mapping.";
    }
    match record.get("classification").and_then(Value::as_str) {
        Some("metadata_migration_conflict") => {
            "Grip cannot safely proceed because stored metadata migration evidence conflicts."
        }
        Some("initial_collision") => {
            "Grip cannot safely proceed because the endpoints differ without an accepted baseline."
        }
        Some("divergent_conflict") => "Grip cannot safely proceed because both endpoints changed.",
        Some("source_side_deletion" | "destination_side_deletion") => {
            "Grip cannot safely proceed because one endpoint is missing."
        }
        Some("delete_change_conflict" | "change_delete_conflict") => {
            "Grip cannot safely proceed because one endpoint is missing while the other changed."
        }
        Some("unsupported_managed") => {
            "Grip cannot safely proceed because this managed entry uses an unsupported filesystem state."
        }
        Some("unsafe_collision") => {
            "Grip cannot safely proceed because the destination has an unsafe collision."
        }
        _ => "Grip cannot safely proceed with this managed entry.",
    }
}

fn classification_path<'a>(record: &'a Value, field: &str) -> &'a str {
    record
        .get(field)
        .and_then(|path| path.get("display"))
        .and_then(Value::as_str)
        .unwrap_or("unknown")
}

fn singular_or_plural<'a>(count: usize, singular: &'a str, plural: &'a str) -> &'a str {
    if count == 1 { singular } else { plural }
}

fn render_human_metadata_details(
    details: &Map<String, Value>,
    writer: &mut dyn Write,
) -> io::Result<()> {
    if let Some(dimensions) = details.get("metadata_dimensions").and_then(Value::as_array) {
        for dimension in dimensions {
            writeln!(
                writer,
                "metadata_dimension: {}",
                dimension.as_str().unwrap_or("unknown")
            )?;
        }
    }
    if let Some(evidence) = details.get("evidence").and_then(Value::as_object) {
        for (field, value) in evidence {
            writeln!(
                writer,
                "metadata_evidence: {field} {}",
                value
                    .get("state")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown")
            )?;
        }
    }
    if let Some(findings) = details
        .get("compatibility_findings")
        .and_then(Value::as_array)
    {
        for finding in findings {
            writeln!(
                writer,
                "compatibility: {} {} {} blocking={} corrective_choice={}",
                finding
                    .get("endpoint")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown"),
                finding
                    .get("field")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown"),
                finding
                    .get("reason")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown"),
                finding
                    .get("blocking")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
                finding
                    .get("corrective_choice")
                    .and_then(Value::as_str)
                    .unwrap_or("unavailable")
            )?;
        }
    }
    if let Some(mappings) = details
        .get("endpoint_capabilities")
        .and_then(Value::as_array)
    {
        for mapping in mappings {
            let source = mapping
                .get("mapping_source")
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            if let Some(profiles) = mapping.get("profiles").and_then(Value::as_array) {
                for profile in profiles {
                    writeln!(
                        writer,
                        "endpoint_capability: mapping={} endpoint={} filesystem={} case_sensitive={} mtime_precision_ns={}",
                        source,
                        profile
                            .get("endpoint")
                            .and_then(Value::as_str)
                            .unwrap_or("unknown"),
                        evidence_display(profile.get("filesystem_type")),
                        evidence_display(profile.get("case_sensitive")),
                        evidence_display(profile.get("mtime_precision_nanoseconds")),
                    )?;
                }
            }
        }
    }
    if let Some(reference) = details.get("recovery_authority").and_then(Value::as_str) {
        writeln!(writer, "recovery_authority: {reference}")?;
    }
    if let Some(verification) = details.get("verification").and_then(Value::as_str) {
        writeln!(writer, "verification: {verification}")?;
    }
    if let Some(generation) = details.get("accepted_generation").and_then(Value::as_u64) {
        writeln!(writer, "accepted_generation: {generation}")?;
    }
    Ok(())
}

fn recovery_reference(reference: &Value) -> String {
    match reference.get("kind").and_then(Value::as_str) {
        Some("payload") => format!(
            "payload:{}:{}",
            reference["operation_id"].as_str().unwrap_or("unknown"),
            reference["action_index"].as_u64().unwrap_or(0)
        ),
        Some("registry") => format!(
            "registry:sha256:{}",
            reference["digest"].as_str().unwrap_or("unknown")
        ),
        Some("accepted_state") => format!(
            "state:generation:{}:sha256:{}",
            reference["generation"].as_u64().unwrap_or(0),
            reference["digest"].as_str().unwrap_or("unknown")
        ),
        Some("operation") => format!(
            "operation:{}",
            reference["operation_id"].as_str().unwrap_or("unknown")
        ),
        _ => "unknown".into(),
    }
}

fn render_human_classification_record(record: &Value, writer: &mut dyn Write) -> io::Result<()> {
    let classification = record
        .get("classification")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let source = record
        .get("source_path")
        .and_then(|path| path.get("display"))
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let direction = record
        .get("prospective_direction")
        .and_then(Value::as_str)
        .unwrap_or("none");
    writeln!(
        writer,
        "{classification} {source} direction={direction} attention={} blocking={}",
        yes_no(record.get("attention")),
        yes_no(record.get("blocking"))
    )?;
    if let Some(reasons) = record.get("reasons").and_then(Value::as_array)
        && !reasons.is_empty()
    {
        let value = reasons
            .iter()
            .filter_map(Value::as_str)
            .collect::<Vec<_>>()
            .join(",");
        writeln!(writer, "  reasons {value}")?;
    }
    if let Some(dimensions) = record.get("changed_dimensions").and_then(Value::as_object) {
        for label in [
            "source_to_baseline",
            "destination_to_baseline",
            "source_to_destination",
        ] {
            match dimensions.get(label) {
                Some(Value::Null) | None => writeln!(writer, "  {label} unavailable")?,
                Some(Value::Array(values)) if values.is_empty() => {
                    writeln!(writer, "  {label} none")?
                }
                Some(Value::Array(values)) => {
                    let value = values
                        .iter()
                        .filter_map(Value::as_str)
                        .collect::<Vec<_>>()
                        .join(",");
                    writeln!(writer, "  {label} {value}")?;
                }
                Some(_) => writeln!(writer, "  {label} unavailable")?,
            }
        }
    }
    if let Some(findings) = record
        .get("compatibility_findings")
        .and_then(Value::as_array)
    {
        for finding in findings {
            writeln!(
                writer,
                "  compatibility endpoint={} field={} reason={} blocking={} required={} corrective_choice={}",
                finding
                    .get("endpoint")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown"),
                finding
                    .get("field")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown"),
                finding
                    .get("reason")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown"),
                yes_no(finding.get("blocking")),
                finding
                    .get("required")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown"),
                finding
                    .get("corrective_choice")
                    .and_then(Value::as_str)
                    .unwrap_or("unavailable")
            )?;
        }
    }
    if let Some(profiles) = record
        .get("endpoint_capabilities")
        .and_then(Value::as_array)
    {
        for profile in profiles {
            writeln!(
                writer,
                "  endpoint={} filesystem={} case_sensitive={} mtime_precision_ns={}",
                profile
                    .get("endpoint")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown"),
                evidence_display(profile.get("filesystem_type")),
                evidence_display(profile.get("case_sensitive")),
                evidence_display(profile.get("mtime_precision_nanoseconds"))
            )?;
        }
    }
    Ok(())
}

fn evidence_display(value: Option<&Value>) -> String {
    let Some(value) = value else {
        return "unavailable".into();
    };
    match value.get("state").and_then(Value::as_str) {
        Some("observed") => value
            .get("value")
            .map_or_else(|| "observed".into(), Value::to_string),
        Some(state) => state.into(),
        None => "unavailable".into(),
    }
}

fn yes_no(value: Option<&Value>) -> &'static str {
    if value.and_then(Value::as_bool).unwrap_or(false) {
        "yes"
    } else {
        "no"
    }
}

fn render_human_discovery_record(record: &Value, writer: &mut dyn Write) -> io::Result<()> {
    let category = record
        .get("category")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let mapping = record
        .get("mapping_source")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let relative = record
        .get("relative_path")
        .and_then(|path| path.get("display"))
        .and_then(Value::as_str)
        .unwrap_or(".");
    let destination = record
        .get("destination_path")
        .and_then(|path| path.get("display"))
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let kind = record
        .get("node_kind")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    if let Some(reason) = record.get("reason").and_then(Value::as_str) {
        writeln!(
            writer,
            "{category} {mapping} {relative} -> {destination} {kind} {reason}"
        )
    } else {
        writeln!(
            writer,
            "{category} {mapping} {relative} -> {destination} {kind}"
        )
    }
}

fn render_human_mapping(
    mapping: &Value,
    include_kind: bool,
    writer: &mut dyn Write,
) -> io::Result<()> {
    let declared = mapping.get("declared").unwrap_or(mapping);
    let kind = declared
        .get("kind")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let source = declared
        .get("source")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let destination = declared
        .get("destination")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    if include_kind {
        writeln!(writer, " {kind} {source} -> {destination}")
    } else {
        writeln!(writer, " {source} -> {destination}")
    }
}

fn render_human_named_mapping(
    mapping: &Value,
    name: &str,
    writer: &mut dyn Write,
) -> io::Result<()> {
    let declared = mapping.get("declared").unwrap_or(mapping);
    let source = declared
        .get("source")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let destination = declared
        .get("destination")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    writeln!(writer, "  {name}: {source} -> {destination}")
}

pub fn emit_diagnostic(verbosity: u8, event: &str, writer: &mut dyn Write) -> io::Result<()> {
    if verbosity > 0 {
        writeln!(writer, "diagnostic: {event}")
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata::model::{
        CompatibilityFinding, CompatibilityReason, EndpointRole, Evidence, MetadataDimension,
        XattrFingerprint,
    };

    #[test]
    fn status_renderer_partitions_every_human_status_group() {
        let record =
            |source: &str, destination: &str, attention: bool, blocking: bool, direction: &str| {
                serde_json::json!({
                    "source_path": {"display": source},
                    "destination_path": {"display": destination},
                    "attention": attention,
                    "blocking": blocking,
                    "prospective_direction": direction,
                })
            };
        let details = serde_json::json!({"records": [
            record("current", "~/current", false, false, "none"),
            record("push", "~/push", true, false, "source_to_destination"),
            record("pull", "~/pull", true, false, "destination_to_source"),
            record("conflict", "~/conflict", true, true, "none"),
            record("deleted", "~/deleted", true, false, "none"),
            record("migration", "~/migration", true, false, "none"),
        ]})
        .as_object()
        .unwrap()
        .clone();
        let mut output = Vec::new();
        let source_paths = vec![
            "current".into(),
            "main.py".into(),
            "../shared/config.yml".into(),
            "../README.md".into(),
            "new-file".into(),
            "reconcile".into(),
        ];

        render_human_status(&details, Some(&source_paths), None, &mut output).unwrap();

        assert_eq!(
            String::from_utf8(output).unwrap(),
            "Status: 6 entries checked; 1 current; 1 to push; 1 to pull; 1 conflict; 2 needs baseline.\n\nChanges to push:\n  main.py -> ~/push\n\nChanges to pull:\n  ../shared/config.yml <- ~/pull\n\nConflicts:\n  ../README.md <-> ~/conflict\n    Blocked: Grip cannot safely proceed with this managed entry.\n\nNeeds baseline:\n  new-file >-< ~/deleted\n  reconcile >-< ~/migration\n"
        );
    }

    #[test]
    fn project_result_exposes_safe_identity_and_each_rebinding_outcome() {
        for state in [
            RebindingOutcome::Uninitialized,
            RebindingOutcome::Bound,
            RebindingOutcome::RebindEligible,
            RebindingOutcome::RebindBlocked,
        ] {
            let value = serde_json::to_value(ProjectResultDetails {
                root: crate::discovery::model::SafePath::from_path(std::path::Path::new(
                    "/safe/project",
                )),
                state,
                prior_root: None,
            })
            .unwrap();
            assert_eq!(value["root"]["display"], "/safe/project");
            assert!(value["state"].is_string());
        }
    }

    #[test]
    fn mapping_result_keeps_declared_and_resolved_values_separate() {
        let root = tempfile::tempdir().unwrap();
        let project = root.path().join("project");
        let home = root.path().join("home");
        std::fs::create_dir(&project).unwrap();
        std::fs::create_dir(&home).unwrap();
        std::fs::write(project.join("source"), "payload").unwrap();
        let mapping = crate::mapping::PortableMapping::parse(
            crate::mapping::MappingKind::File,
            std::ffi::OsStr::new("source"),
            std::ffi::OsStr::new("~/destination"),
        )
        .unwrap()
        .resolve(&project, &home, "test")
        .unwrap();
        let value = serde_json::to_value(MappingResultDetails::from(&mapping)).unwrap();
        assert_eq!(value["declared"]["source"], "source");
        assert_eq!(value["declared"]["destination"], "~/destination");
        assert_eq!(
            value["resolved"]["source"]["display"],
            std::fs::canonicalize(project.join("source"))
                .unwrap()
                .display()
                .to_string()
        );
        assert_ne!(value["declared"]["source"], value["resolved"]["source"]);
    }

    #[test]
    fn metadata_result_v1_retains_evidence_and_authority_without_raw_xattr_values() {
        let details = MetadataResultDetails {
            metadata_dimensions: vec![
                MetadataDimension::PermissionMode,
                MetadataDimension::ExtendedAttribute,
            ],
            evidence: std::collections::BTreeMap::from([
                (
                    "owner".into(),
                    Evidence::Observed {
                        value: "501".into(),
                    },
                ),
                ("acl".into(), Evidence::Absent),
                (
                    "group".into(),
                    Evidence::Unauthorized {
                        reason: "group membership not proven".into(),
                    },
                ),
            ]),
            compatibility_findings: vec![CompatibilityFinding {
                endpoint: EndpointRole::Destination,
                path_display: "/safe/path".into(),
                path_raw_hex: None,
                field: MetadataDimension::Owner,
                required: "501".into(),
                evidence_state: "unauthorized".into(),
                reason: CompatibilityReason::Unauthorized,
                message: "owner transition is not authorized".into(),
                corrective_choice: "pre-align ownership or select the other authority".into(),
                blocking: true,
            }],
            extended_attributes: vec![XattrFingerprint {
                name: b"user.test".to_vec(),
                length: 12,
                algorithm: "sha256".into(),
                digest: "a".repeat(64),
            }],
            recovery_authority: Some("payload:push-1:0".into()),
            verification: "verified".into(),
            durability_confirmed: true,
            accepted_generation: Some(7),
        };
        let envelope = ResultEnvelopeV1::from(CommandOutcome::metadata(
            ResultCategory::Success,
            "metadata inspected",
            &details,
        ));
        let encoded = serde_json::to_string(&envelope).unwrap();
        assert_eq!(envelope.schema_version, 1);
        assert_eq!(envelope.details["evidence"]["acl"]["state"], "absent");
        assert_eq!(envelope.details["accepted_generation"], 7);
        assert_eq!(envelope.details["durability_confirmed"], true);
        assert_eq!(envelope.details["recovery_authority"], "payload:push-1:0");
        assert!(!encoded.contains("raw_value"));
        assert!(!encoded.contains("secret-xattr-value"));
    }
    #[test]
    fn json_envelope_has_consistent_status_and_code() {
        let mut output = Vec::new();
        render(CommandOutcome::success("ok"), OutputMode::Json, &mut output).unwrap();
        let value: Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(value["status"], "ok");
        assert_eq!(value["code"], "ok");
    }

    struct FailingWriter;
    impl Write for FailingWriter {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(io::Error::other("closed"))
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    #[test]
    fn renderer_propagates_output_failure() {
        assert!(
            render(
                CommandOutcome::success("ok"),
                OutputMode::Json,
                &mut FailingWriter
            )
            .is_err()
        );
    }

    #[test]
    fn discovery_renderer_propagates_output_failure() {
        let inventory = crate::discovery::model::DiscoveryInventory::new(
            crate::discovery::model::DiscoveryScope::All,
            Vec::new(),
        );
        assert!(
            render(
                CommandOutcome::discovery(&inventory),
                OutputMode::Json,
                &mut FailingWriter
            )
            .is_err()
        );
    }

    #[test]
    fn mapping_failure_has_stable_structured_details() {
        let error = GripError::mapping(
            "mapping_show",
            "mapping_not_found",
            vec!["/source".into()],
            "Mapping not found",
        );
        let outcome = CommandOutcome::failure(&error);
        assert_eq!(outcome.details["operation"], "mapping_show");
        assert_eq!(outcome.details["reason"], "mapping_not_found");
        assert_eq!(outcome.details["paths"][0], "/source");
    }

    #[test]
    fn publication_mapping_failures_use_the_operational_failure_envelope() {
        for reason in ["publication_failure", "registry_recovery_failure"] {
            let error = GripError::mapping(
                "mapping_update",
                reason,
                vec!["/grip/config.toml".into()],
                "Registry update failed",
            );
            let outcome = CommandOutcome::failure(&error);
            assert_eq!(outcome.category, ResultCategory::InternalError);
            assert_eq!(outcome.details["reason"], reason);
            let envelope = ResultEnvelopeV1::from(outcome);
            assert_eq!(envelope.code, "operational_failure");
            assert_eq!(envelope.status, "error");
            assert_eq!(envelope.details["operation"], "mapping_update");
        }
    }

    #[test]
    fn registry_io_mapping_failure_retains_operational_category_and_details() {
        let error = GripError::RegistryIo("could not read config.toml".into())
            .for_mapping_operation("mapping_list")
            .with_paths_if_empty(vec!["/grip/config.toml".into()]);
        let envelope = ResultEnvelopeV1::from(CommandOutcome::failure(&error));
        assert_eq!(envelope.code, "operational_failure");
        assert_eq!(envelope.details["operation"], "mapping_list");
        assert_eq!(envelope.details["reason"], "invalid_registry");
        assert_eq!(envelope.details["paths"][0], "/grip/config.toml");
    }

    #[test]
    fn unsupported_schema_mapping_failure_retains_category_and_details() {
        let error = GripError::UnsupportedSchema("unsupported registry schema version 2".into())
            .for_mapping_operation("mapping_show")
            .with_paths_if_empty(vec!["/grip/config.toml".into()]);
        let envelope = ResultEnvelopeV1::from(CommandOutcome::failure(&error));
        assert_eq!(envelope.code, "unsupported_schema");
        assert_eq!(envelope.details["operation"], "mapping_show");
        assert_eq!(envelope.details["reason"], "invalid_registry");
        assert_eq!(envelope.details["paths"][0], "/grip/config.toml");
    }

    #[test]
    fn feature_eight_lifecycle_failure_exposes_independent_evidence_fields() {
        let error = GripError::Lifecycle {
            operation: "recovery_restore".into(),
            reason: "restore_verification_failed".into(),
            category: ResultCategory::InternalError,
            publication_visible: true,
            verification: "failed".into(),
            durability_confirmed: false,
            message: "restore failed".into(),
        };
        let envelope = ResultEnvelopeV1::from(CommandOutcome::failure(&error));
        assert_eq!(envelope.code, "operational_failure");
        assert_eq!(envelope.details["operation"], "recovery_restore");
        assert_eq!(envelope.details["reason"], "restore_verification_failed");
        assert_eq!(envelope.details["publication_visible"], true);
        assert_eq!(envelope.details["verification"], "failed");
        assert_eq!(envelope.details["durability_confirmed"], false);
    }

    #[test]
    fn typed_operation_result_wraps_operation_record_identity() {
        let outcome = operation_outcome(
            ResultCategory::Success,
            "complete".into(),
            &serde_json::json!({"operation":"delete","plan_id":"a","actions":[],"blockers":[]}),
            "execute",
            "complete",
            "applied",
            Some("delete-1"),
            &serde_json::json!({"authoritative_generation":2}),
        );
        assert_eq!(outcome.details["operation_record"]["available"], true);
        assert_eq!(outcome.details["operation_record"]["id"], "delete-1");
        assert_eq!(outcome.details["baseline"]["authoritative_generation"], 2);
    }

    #[test]
    fn missing_peer_blocker_wording_does_not_claim_a_change() {
        let missing_peer = serde_json::json!({"classification": "source_side_deletion"});
        let changed_peer = serde_json::json!({"classification": "delete_change_conflict"});
        assert_eq!(
            classification_blocker_message(&missing_peer),
            "Grip cannot safely proceed because one endpoint is missing."
        );
        assert_eq!(
            classification_blocker_message(&changed_peer),
            "Grip cannot safely proceed because one endpoint is missing while the other changed."
        );
    }
}
