//! Typed classification records and summaries.

use crate::discovery::model::SafePath;
use crate::observation::model::{EntryIdentity, PathSpace, SupportedState};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Classification {
    MetadataMigrationReady,
    MetadataMigrationConflict,
    SourceAddition,
    InitialMatch,
    InitialCollision,
    DestinationOnlyUnmanaged,
    Synchronized,
    SourceOnlyChange,
    DestinationOnlyChange,
    ConvergedTwoSidedChange,
    DivergentConflict,
    SourceSideDeletion,
    DestinationSideDeletion,
    DeleteChangeConflict,
    ChangeDeleteConflict,
    ConvergedDeletion,
    UnsupportedManaged,
    UnsafeCollision,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    SourceToDestination,
    DestinationToSource,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangedDimension {
    NodeKind,
    Content,
    PermissionMode,
    Owner,
    Group,
    ModificationTime,
    ExtendedAttribute,
    AccessControlList,
    BsdFlags,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ChangedDimensions {
    pub source_to_baseline: Option<Vec<ChangedDimension>>,
    pub destination_to_baseline: Option<Vec<ChangedDimension>>,
    pub source_to_destination: Option<Vec<ChangedDimension>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ClassificationRecord {
    #[serde(skip)]
    pub identity: EntryIdentity,
    pub classification: Classification,
    pub mapping_kind: crate::mapping::MappingKind,
    pub mapping_source: std::path::PathBuf,
    pub relative_path: Option<SafePath>,
    pub source_path: SafePath,
    pub destination_path: SafePath,
    pub source: Option<SupportedState>,
    pub destination: Option<SupportedState>,
    pub baseline: Option<SupportedState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_complete: Option<crate::metadata::model::SupportedEntryStateV3>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_complete: Option<crate::metadata::model::SupportedEntryStateV3>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline_complete: Option<crate::metadata::model::SupportedEntryStateV3>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub compatibility_findings: Vec<crate::metadata::model::CompatibilityFinding>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub endpoint_capabilities: Vec<crate::metadata::model::EndpointCapabilityProfile>,
    pub prospective_direction: Direction,
    pub changed_dimensions: ChangedDimensions,
    pub attention: bool,
    pub blocking: bool,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct ClassificationCounts {
    pub metadata_migration_ready: usize,
    pub metadata_migration_conflict: usize,
    pub source_addition: usize,
    pub initial_match: usize,
    pub initial_collision: usize,
    pub destination_only_unmanaged: usize,
    pub synchronized: usize,
    pub source_only_change: usize,
    pub destination_only_change: usize,
    pub converged_two_sided_change: usize,
    pub divergent_conflict: usize,
    pub source_side_deletion: usize,
    pub destination_side_deletion: usize,
    pub delete_change_conflict: usize,
    pub change_delete_conflict: usize,
    pub converged_deletion: usize,
    pub unsupported_managed: usize,
    pub unsafe_collision: usize,
}

impl ClassificationCounts {
    pub fn record(&mut self, classification: Classification) {
        match classification {
            Classification::MetadataMigrationReady => self.metadata_migration_ready += 1,
            Classification::MetadataMigrationConflict => self.metadata_migration_conflict += 1,
            Classification::SourceAddition => self.source_addition += 1,
            Classification::InitialMatch => self.initial_match += 1,
            Classification::InitialCollision => self.initial_collision += 1,
            Classification::DestinationOnlyUnmanaged => self.destination_only_unmanaged += 1,
            Classification::Synchronized => self.synchronized += 1,
            Classification::SourceOnlyChange => self.source_only_change += 1,
            Classification::DestinationOnlyChange => self.destination_only_change += 1,
            Classification::ConvergedTwoSidedChange => self.converged_two_sided_change += 1,
            Classification::DivergentConflict => self.divergent_conflict += 1,
            Classification::SourceSideDeletion => self.source_side_deletion += 1,
            Classification::DestinationSideDeletion => self.destination_side_deletion += 1,
            Classification::DeleteChangeConflict => self.delete_change_conflict += 1,
            Classification::ChangeDeleteConflict => self.change_delete_conflict += 1,
            Classification::ConvergedDeletion => self.converged_deletion += 1,
            Classification::UnsupportedManaged => self.unsupported_managed += 1,
            Classification::UnsafeCollision => self.unsafe_collision += 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ClassificationScope {
    pub kind: String,
    pub path_space: PathSpace,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<SafePath>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mapping_source: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ClassificationResult {
    pub operation: String,
    pub completion: &'static str,
    pub state: &'static str,
    pub scope: ClassificationScope,
    pub counts: ClassificationCounts,
    pub attention_count: usize,
    pub blocking_count: usize,
    pub records: Vec<ClassificationRecord>,
}

impl ClassificationResult {
    pub fn new(
        operation: &str,
        scope: ClassificationScope,
        mut records: Vec<ClassificationRecord>,
    ) -> Self {
        records.sort_by(|first, second| {
            first
                .identity
                .cmp(&second.identity)
                .then(first.classification.cmp(&second.classification))
        });
        let mut counts = ClassificationCounts::default();
        for record in &records {
            counts.record(record.classification);
        }
        let attention_count = records.iter().filter(|record| record.attention).count();
        let blocking_count = records.iter().filter(|record| record.blocking).count();
        Self {
            operation: operation.into(),
            completion: "complete",
            state: if attention_count == 0 {
                "clean"
            } else {
                "attention_required"
            },
            scope,
            counts,
            attention_count,
            blocking_count,
            records,
        }
    }
}
