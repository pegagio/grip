//! Pure three-way synchronization classification.

pub mod model;

use crate::observation::model::{Membership, ObservedEntry, SupportedState};
use model::{ChangedDimension, ChangedDimensions, Classification, ClassificationRecord, Direction};

pub fn changed_dimensions(
    first: Option<&SupportedState>,
    second: Option<&SupportedState>,
) -> Option<Vec<ChangedDimension>> {
    let (Some(first), Some(second)) = (first, second) else {
        return None;
    };
    if first.node_kind != second.node_kind {
        return Some(vec![ChangedDimension::NodeKind]);
    }
    let mut result = Vec::new();
    if first
        .content
        .as_ref()
        .map(|value| (&value.algorithm, &value.digest))
        != second
            .content
            .as_ref()
            .map(|value| (&value.algorithm, &value.digest))
    {
        result.push(ChangedDimension::Content);
    }
    if first.permission_mode != second.permission_mode {
        result.push(ChangedDimension::PermissionMode);
    }
    Some(result)
}

pub fn classify(entry: &ObservedEntry, baseline: Option<&SupportedState>) -> ClassificationRecord {
    let source = entry.source.as_ref();
    let destination = entry.destination.as_ref();
    let unsafe_destination = entry
        .unsupported
        .iter()
        .any(|reason| reason.starts_with("destination:") || reason == "wrong_node_kind");
    let classification = if entry.membership == Membership::Untracked {
        Classification::UntrackedPendingRetirement
    } else if entry.membership == Membership::Ignored && baseline.is_some() {
        Classification::NewlyIgnoredPendingRetirement
    } else if entry.blocking && unsafe_destination {
        Classification::UnsafeCollision
    } else if entry.blocking {
        Classification::UnsupportedManaged
    } else if let Some(accepted) = baseline {
        classify_with_baseline(source, destination, accepted)
    } else {
        classify_without_baseline(source, destination)
    };
    let (direction, attention, blocking, default_reason) = properties(classification);
    let mut reasons = entry.unsupported.clone();
    if reasons.is_empty() && entry.membership == Membership::Ignored && baseline.is_none() {
        reasons.push("ignored".into());
    }
    if reasons.is_empty() && !default_reason.is_empty() {
        reasons.push(default_reason.into());
    }
    ClassificationRecord {
        identity: entry.identity.clone(),
        classification,
        mapping_kind: entry.identity.mapping.kind,
        mapping_source: entry.identity.mapping.source.clone(),
        relative_path: entry.identity.relative_safe_path(),
        source_path: crate::discovery::model::SafePath::from_path(&entry.identity.source_path()),
        destination_path: crate::discovery::model::SafePath::from_path(
            &entry.identity.destination_path(),
        ),
        source: entry.source.clone(),
        destination: entry.destination.clone(),
        baseline: baseline.cloned(),
        prospective_direction: direction,
        changed_dimensions: ChangedDimensions {
            source_to_baseline: changed_dimensions(source, baseline),
            destination_to_baseline: changed_dimensions(destination, baseline),
            source_to_destination: changed_dimensions(source, destination),
        },
        attention,
        blocking: blocking || entry.blocking,
        reasons,
    }
}

fn classify_without_baseline(
    source: Option<&SupportedState>,
    destination: Option<&SupportedState>,
) -> Classification {
    match (source, destination) {
        (Some(_), None) => Classification::SourceAddition,
        (Some(source), Some(destination)) if source == destination => Classification::InitialMatch,
        (Some(_), Some(_)) => Classification::InitialCollision,
        (None, Some(_)) | (None, None) => Classification::DestinationOnlyUnmanaged,
    }
}

fn classify_with_baseline(
    source: Option<&SupportedState>,
    destination: Option<&SupportedState>,
    baseline: &SupportedState,
) -> Classification {
    match (source, destination) {
        (None, None) => Classification::ConvergedDeletion,
        (None, Some(destination)) if destination == baseline => Classification::SourceSideDeletion,
        (None, Some(_)) => Classification::DeleteChangeConflict,
        (Some(source), None) if source == baseline => Classification::DestinationSideDeletion,
        (Some(_), None) => Classification::ChangeDeleteConflict,
        (Some(source), Some(destination)) if source == baseline && destination == baseline => {
            Classification::Synchronized
        }
        (Some(source), Some(destination)) if source != baseline && destination == baseline => {
            Classification::SourceOnlyChange
        }
        (Some(source), Some(destination)) if source == baseline && destination != baseline => {
            Classification::DestinationOnlyChange
        }
        (Some(source), Some(destination)) if source == destination => {
            Classification::ConvergedTwoSidedChange
        }
        (Some(_), Some(_)) => Classification::DivergentConflict,
    }
}

fn properties(classification: Classification) -> (Direction, bool, bool, &'static str) {
    match classification {
        Classification::Synchronized => (Direction::None, false, false, ""),
        Classification::DestinationOnlyUnmanaged => {
            (Direction::None, false, false, "unmanaged_destination")
        }
        Classification::SourceAddition => (
            Direction::SourceToDestination,
            true,
            false,
            "baseline_uninitialized",
        ),
        Classification::InitialMatch => (Direction::None, true, false, "baseline_uninitialized"),
        Classification::InitialCollision => (Direction::None, true, true, "initial_collision"),
        Classification::SourceOnlyChange => (
            Direction::SourceToDestination,
            true,
            false,
            "source_changed",
        ),
        Classification::DestinationOnlyChange => (
            Direction::DestinationToSource,
            true,
            false,
            "destination_changed",
        ),
        Classification::ConvergedTwoSidedChange => {
            (Direction::None, true, false, "baseline_refresh_available")
        }
        Classification::DivergentConflict => (Direction::None, true, true, "divergent_change"),
        Classification::SourceSideDeletion => (
            Direction::SourceToDestination,
            true,
            false,
            "source_deleted",
        ),
        Classification::DestinationSideDeletion => (
            Direction::DestinationToSource,
            true,
            false,
            "destination_deleted",
        ),
        Classification::DeleteChangeConflict => {
            (Direction::None, true, true, "delete_change_conflict")
        }
        Classification::ChangeDeleteConflict => {
            (Direction::None, true, true, "change_delete_conflict")
        }
        Classification::ConvergedDeletion => (Direction::None, true, false, "converged_deletion"),
        Classification::NewlyIgnoredPendingRetirement => {
            (Direction::None, true, false, "newly_ignored")
        }
        Classification::UntrackedPendingRetirement => {
            (Direction::None, true, false, "mapping_removed")
        }
        Classification::UnsupportedManaged => (Direction::None, true, true, "unsupported_managed"),
        Classification::UnsafeCollision => (Direction::None, true, true, "unsafe_collision"),
    }
}
