//! Stable source and destination observation orchestration.

pub mod fingerprint;
pub mod model;

use crate::discovery::model::{NodeKind, RecordCategory};
use crate::error::GripError;
use crate::mapping::{Mapping, MappingKind};
use crate::project::ProjectPaths;
use crate::registry::publication::RegistrySnapshot;
use crate::state::AcceptedState;
use model::{EntryIdentity, Membership, Observation, ObservedEntry, ResolvedMapping, Selection};
use std::collections::BTreeMap;

/// Build a stable observation and join it with retained accepted identities.
pub fn inspect(
    home: &ProjectPaths,
    registry: &RegistrySnapshot,
    accepted: &AcceptedState,
    selection: &Selection,
) -> Result<Observation, GripError> {
    let inventory = crate::discovery::inspect(home, registry, accepted, None)?;
    let first = inspect_once(&registry.registry, accepted, selection, &inventory)?;
    let second = inspect_once(&registry.registry, accepted, selection, &inventory)?;
    crate::registry::publication::revalidate_readonly(home, registry, "classification")?;
    if first != second {
        return Err(GripError::discovery_operational(
            "classification",
            "stale_observation_evidence",
            Vec::new(),
            "observation evidence changed before classification completed",
        ));
    }
    Ok(second)
}

/// Inspect a proposed add registry while retaining the active registry as the revalidation
/// authority. The candidate is never published merely to obtain baseline evidence.
pub fn inspect_candidate(
    home: &ProjectPaths,
    candidate: &crate::registry::ResolvedRegistry,
    candidate_descriptor_bytes: &[u8],
    expected: &RegistrySnapshot,
    accepted: &AcceptedState,
    selection: &Selection,
) -> Result<Observation, GripError> {
    let inventory = crate::discovery::inspect_candidate(
        home,
        candidate,
        candidate_descriptor_bytes,
        expected,
        accepted,
    )?;
    let first = inspect_once(candidate, accepted, selection, &inventory)?;
    let second = inspect_once(candidate, accepted, selection, &inventory)?;
    crate::registry::publication::revalidate_readonly(home, expected, "classification")?;
    if first != second {
        return Err(GripError::discovery_operational(
            "classification",
            "stale_observation_evidence",
            Vec::new(),
            "candidate observation evidence changed before classification completed",
        ));
    }
    Ok(second)
}

fn inspect_once(
    registry: &crate::registry::ResolvedRegistry,
    accepted: &AcceptedState,
    selection: &Selection,
    inventory: &crate::discovery::model::DiscoveryInventory,
) -> Result<Observation, GripError> {
    let current: BTreeMap<ResolvedMapping, &Mapping> = registry
        .mappings()
        .iter()
        .map(|mapping| (ResolvedMapping::from(mapping), mapping))
        .collect();
    let mut observed = Observation::new();
    let mut relative_inspectors = BTreeMap::new();
    let mut complete_inspectors = BTreeMap::new();

    for mapping in registry.mappings() {
        if mapping.kind == MappingKind::File
            && !file_mapping_is_discovery_backed(mapping, inventory)
        {
            let identity = EntryIdentity::new(ResolvedMapping::from(mapping), Vec::new())
                .expect("file mapping identity is valid");
            let source_complete = inspect_complete_supported(&mapping.source)?;
            let destination_outcome = inspect_destination_leaf(&mapping.destination)?;
            let (destination_complete, destination_link, destination_blocking) =
                match destination_outcome {
                    fingerprint::RelativeCompleteObservation::Missing => (None, None, None),
                    fingerprint::RelativeCompleteObservation::Supported(complete) => {
                        (Some(*complete), None, None)
                    }
                    fingerprint::RelativeCompleteObservation::DestinationLeafLink(link) => {
                        (None, Some(link), None)
                    }
                    fingerprint::RelativeCompleteObservation::Blocking { reason } => {
                        (None, None, Some(reason))
                    }
                };
            let source = source_complete.as_ref().map(legacy_from_complete);
            let destination = destination_complete.as_ref().map(legacy_from_complete);
            if source.is_some()
                || destination.is_some()
                || destination_link.is_some()
                || destination_blocking.is_some()
                || accepted.complete_baselines.contains_key(&identity)
            {
                observed.insert(
                    identity.clone(),
                    ObservedEntry {
                        identity,
                        membership: if source.is_some() {
                            Membership::Active
                        } else {
                            Membership::DestinationOnly
                        },
                        source: source.as_ref().map(|value| value.0.clone()),
                        destination: destination.as_ref().map(|value| value.0.clone()),
                        source_complete,
                        destination_complete,
                        destination_link,
                        metadata_findings: Vec::new(),
                        endpoint_capabilities: Vec::new(),
                        source_diagnostic: source.map(|value| value.1),
                        destination_diagnostic: destination.map(|value| value.1),
                        unsupported: destination_blocking
                            .into_iter()
                            .map(|reason| format!("destination:{reason}"))
                            .collect(),
                        blocking: destination_blocking.is_some(),
                    },
                );
            }
        }
    }

    for record in &inventory.records {
        let mapping = registry
            .mappings()
            .iter()
            .find(|mapping| {
                mapping.source == record.mapping_source && mapping.kind == record.mapping_kind
            })
            .expect("discovery record belongs to accepted mapping");
        let relative = record
            .relative_path
            .as_ref()
            .map_or_else(Vec::new, |path| path.raw_bytes().to_vec());
        let identity = EntryIdentity::new(ResolvedMapping::from(mapping), relative)
            .map_err(|message| GripError::CorruptState(message.into()))?;
        let entry = observed
            .entry(identity.clone())
            .or_insert_with(|| ObservedEntry {
                identity: identity.clone(),
                membership: Membership::Active,
                source: None,
                destination: None,
                source_complete: None,
                destination_complete: None,
                destination_link: None,
                metadata_findings: Vec::new(),
                endpoint_capabilities: Vec::new(),
                source_diagnostic: None,
                destination_diagnostic: None,
                unsupported: Vec::new(),
                blocking: false,
            });
        match record.category {
            RecordCategory::Eligible => {
                entry.membership = Membership::Active;
                observe_complete_identity(entry, true, record.node_kind, &mut complete_inspectors)?;
                observe_complete_identity(
                    entry,
                    false,
                    record.node_kind,
                    &mut complete_inspectors,
                )?;
            }
            RecordCategory::Ignored => {
                entry.membership = Membership::Ignored;
                if accepted.complete_baselines.contains_key(&identity) {
                    observe_complete_identity(
                        entry,
                        true,
                        record.node_kind,
                        &mut complete_inspectors,
                    )?;
                    observe_complete_identity(
                        entry,
                        false,
                        record.node_kind,
                        &mut complete_inspectors,
                    )?;
                }
            }
            RecordCategory::DestinationOnly => {
                if entry.membership != Membership::Ignored {
                    entry.membership = Membership::DestinationOnly;
                }
                if matches!(record.node_kind, NodeKind::File | NodeKind::Directory) {
                    observe_complete_identity(
                        entry,
                        false,
                        record.node_kind,
                        &mut complete_inspectors,
                    )?;
                }
            }
            RecordCategory::UnsupportedSource => {
                entry.blocking = true;
                entry
                    .unsupported
                    .push(record.reason.unwrap_or("unsupported_source").into());
            }
            RecordCategory::UnsafeDestinationCollision => {
                entry.blocking = true;
                entry.unsupported.push(format!(
                    "destination:{}",
                    record.reason.unwrap_or("wrong_node_kind")
                ));
                if let Some(relation) = record.relation {
                    entry.unsupported.push(format!("relation:{relation}"));
                }
            }
        }
    }

    for identity in accepted.complete_baselines.keys() {
        let entry = observed
            .entry(identity.clone())
            .or_insert_with(|| ObservedEntry {
                identity: identity.clone(),
                membership: if current.contains_key(&identity.mapping) {
                    Membership::Active
                } else {
                    Membership::Untracked
                },
                source: None,
                destination: None,
                source_complete: None,
                destination_complete: None,
                destination_link: None,
                metadata_findings: Vec::new(),
                endpoint_capabilities: Vec::new(),
                source_diagnostic: None,
                destination_diagnostic: None,
                unsupported: Vec::new(),
                blocking: false,
            });
        if !current.contains_key(&identity.mapping) {
            entry.membership = Membership::Untracked;
            if entry.source.is_none()
                && let Some((state, diagnostic)) =
                    inspect_identity(identity, true, &mut relative_inspectors)?
            {
                entry.source = Some(state);
                entry.source_diagnostic = Some(diagnostic);
            }
            if entry.destination.is_none()
                && let Some((state, diagnostic)) =
                    inspect_identity(identity, false, &mut relative_inspectors)?
            {
                entry.destination = Some(state);
                entry.destination_diagnostic = Some(diagnostic);
            }
        }
    }

    let mut capability_cache: BTreeMap<
        ResolvedMapping,
        Vec<crate::metadata::model::EndpointCapabilityProfile>,
    > = BTreeMap::new();
    for entry in observed.values_mut() {
        append_xattr_findings(entry);
        append_bsd_flag_findings(entry);
        let mapping = entry.identity.mapping.clone();
        entry.endpoint_capabilities = if let Some(profiles) = capability_cache.get(&mapping) {
            profiles.clone()
        } else {
            let profiles = endpoint_profiles(&entry.identity)?;
            capability_cache.insert(mapping, profiles.clone());
            profiles
        };
        append_ownership_findings(entry)?;
    }
    let selected_mapping = selection.mapping().cloned();
    observed.retain(|identity, entry| {
        let selected_recursive_blocker = selected_mapping.as_ref() == Some(&identity.mapping)
            && entry
                .unsupported
                .iter()
                .any(|reason| reason == "destination:recursive_member_topology");
        (selection.includes(identity) || selected_recursive_blocker)
            && !(matches!(selection, Selection::All | Selection::Mapping(_))
                && entry.membership == Membership::Ignored
                && !accepted.complete_baselines.contains_key(identity))
    });
    Ok(observed)
}

/// Return whether this file mapping has an eligible discovery record whose complete endpoint
/// observation will be collected later in the same inspection pass.
fn file_mapping_is_discovery_backed(
    mapping: &Mapping,
    inventory: &crate::discovery::model::DiscoveryInventory,
) -> bool {
    inventory.records.iter().any(|record| {
        record.category == RecordCategory::Eligible
            && record.mapping_kind == MappingKind::File
            && record.mapping_source == mapping.source
            && record.relative_path.is_none()
    })
}

fn append_bsd_flag_findings(entry: &mut ObservedEntry) {
    let source = entry
        .source_complete
        .as_ref()
        .map(|observed| observed.unsupported_bsd_flags.as_slice())
        .unwrap_or_default();
    let destination = entry
        .destination_complete
        .as_ref()
        .map(|observed| observed.unsupported_bsd_flags.as_slice())
        .unwrap_or_default();
    let differing = source != destination;
    for (role, flags, path) in [
        (
            crate::metadata::model::EndpointRole::Source,
            source,
            entry.identity.source_path(),
        ),
        (
            crate::metadata::model::EndpointRole::Destination,
            destination,
            entry.identity.destination_path(),
        ),
    ] {
        for flag in flags {
            entry
                .metadata_findings
                .push(crate::metadata::model::CompatibilityFinding {
                    endpoint: role,
                    path_display: crate::discovery::model::SafePath::from_path(&path).display,
                    path_raw_hex: None,
                    field: crate::metadata::model::MetadataDimension::BsdFlags,
                    required: flag.clone(),
                    evidence_state: "unsupported".into(),
                    reason: crate::metadata::model::CompatibilityReason::ProtectedFlag,
                    message: if differing {
                        "unsupported or protected BSD flag differs between peers".into()
                    } else {
                        "unsupported or protected BSD flag is present equally on both peers".into()
                    },
                    corrective_choice:
                        "align or remove the unsupported flag outside Grip, or exclude this entry"
                            .into(),
                    blocking: differing,
                });
        }
    }
    entry.blocking |= differing && (!source.is_empty() || !destination.is_empty());
    entry.metadata_findings.sort_by(|first, second| {
        first
            .endpoint
            .cmp(&second.endpoint)
            .then(first.path_display.cmp(&second.path_display))
            .then(first.field.cmp(&second.field))
            .then(first.required.cmp(&second.required))
    });
}

fn endpoint_profiles(
    identity: &EntryIdentity,
) -> Result<Vec<crate::metadata::model::EndpointCapabilityProfile>, GripError> {
    let mut profiles = Vec::new();
    for (role, root) in [
        (
            crate::metadata::model::EndpointRole::Source,
            &identity.mapping.source,
        ),
        (
            crate::metadata::model::EndpointRole::Destination,
            &identity.mapping.destination,
        ),
    ] {
        let mut capability_path = root.as_path();
        // A final destination link has no supported endpoint capability of its own. Use its
        // already-required parent directory without opening or resolving the link target.
        if role == crate::metadata::model::EndpointRole::Destination
            && crate::discovery::filesystem::metadata_at_path(capability_path).is_ok_and(
                |metadata| metadata.classify(metadata.stat.st_dev as u64) == NodeKind::Symlink,
            )
        {
            capability_path = capability_path.parent().ok_or_else(|| {
                GripError::InvalidConfiguration("destination link has no parent directory".into())
            })?;
        }
        let metadata = loop {
            match crate::discovery::filesystem::metadata_at_path(capability_path) {
                Ok(value) => break value,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    capability_path = capability_path.parent().ok_or_else(|| {
                        GripError::InvalidConfiguration(
                            "endpoint capability root has no existing ancestor".into(),
                        )
                    })?;
                }
                Err(error) => {
                    return Err(GripError::from_io(
                        "could not inspect endpoint capability root",
                        error,
                    ));
                }
            }
        };
        let mut flags =
            rustix::fs::OFlags::RDONLY | rustix::fs::OFlags::CLOEXEC | rustix::fs::OFlags::NOFOLLOW;
        if metadata.stat.st_mode & libc::S_IFMT == libc::S_IFDIR {
            flags |= rustix::fs::OFlags::DIRECTORY;
        }
        let file = std::fs::File::from(
            rustix::fs::open(capability_path, flags, rustix::fs::Mode::empty()).map_err(
                |error| GripError::from_io("could not open endpoint capability root", error.into()),
            )?,
        );
        profiles.push(
            crate::metadata::macos::endpoint_capability_profile(
                &file,
                role,
                crate::discovery::model::SafePath::from_path(root).display,
            )
            .map_err(|error| {
                GripError::from_io("could not inspect endpoint capabilities", error)
            })?,
        );
    }
    Ok(profiles)
}

fn append_ownership_findings(entry: &mut ObservedEntry) -> Result<(), GripError> {
    for (role, target, current, required) in [
        (
            crate::metadata::model::EndpointRole::Destination,
            entry.identity.destination_path(),
            entry.destination_complete.as_ref(),
            entry.source_complete.as_ref(),
        ),
        (
            crate::metadata::model::EndpointRole::Source,
            entry.identity.source_path(),
            entry.source_complete.as_ref(),
            entry.destination_complete.as_ref(),
        ),
    ] {
        let Some(required) = required else { continue };
        let authorization = if let Some(current) = current {
            crate::metadata::macos::ownership_authorization_for(
                current.state.metadata.uid,
                current.state.metadata.gid,
                required.state.metadata.uid,
                required.state.metadata.gid,
            )
        } else {
            ownership_authorization_at_ancestor(
                &target,
                required.state.metadata.uid,
                required.state.metadata.gid,
            )
        }
        .map_err(|error| GripError::from_io("could not inspect ownership authorization", error))?;
        if !matches!(
            authorization,
            crate::metadata::model::Evidence::Observed { value: true }
        ) {
            entry
                .metadata_findings
                .push(crate::metadata::model::CompatibilityFinding {
                    endpoint: role,
                    path_display: crate::discovery::model::SafePath::from_path(&target).display,
                    path_raw_hex: None,
                    field: crate::metadata::model::MetadataDimension::Owner,
                    required: format!(
                        "uid={} gid={}",
                        required.state.metadata.uid, required.state.metadata.gid
                    ),
                    evidence_state: "unauthorized".into(),
                    reason: crate::metadata::model::CompatibilityReason::Unauthorized,
                    message:
                        "invoking process cannot prove the required numeric ownership transition"
                            .into(),
                    corrective_choice:
                        "pre-align numeric ownership outside Grip or choose a direction requiring no ownership change"
                            .into(),
                    blocking: true,
                });
        }
    }
    entry.metadata_findings.sort_by(|first, second| {
        first
            .endpoint
            .cmp(&second.endpoint)
            .then(first.path_display.cmp(&second.path_display))
            .then(first.required.cmp(&second.required))
    });
    Ok(())
}

fn ownership_authorization_at_ancestor(
    target: &std::path::Path,
    required_uid: u32,
    required_gid: u32,
) -> std::io::Result<crate::metadata::model::Evidence<bool>> {
    let mut capability_path = target;
    let metadata = loop {
        match crate::discovery::filesystem::metadata_at_path(capability_path) {
            Ok(value) => break value,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                capability_path = capability_path.parent().ok_or_else(|| {
                    std::io::Error::other("ownership target has no existing ancestor")
                })?;
            }
            Err(error) => return Err(error),
        }
    };
    crate::metadata::macos::ownership_authorization_for(
        metadata.stat.st_uid,
        metadata.stat.st_gid,
        required_uid,
        required_gid,
    )
}

fn append_xattr_findings(entry: &mut ObservedEntry) {
    for (role, observed, path) in [
        (
            crate::metadata::model::EndpointRole::Source,
            entry.source_complete.as_ref(),
            entry.identity.source_path(),
        ),
        (
            crate::metadata::model::EndpointRole::Destination,
            entry.destination_complete.as_ref(),
            entry.identity.destination_path(),
        ),
    ] {
        let Some(observed) = observed else { continue };
        for name in &observed.excluded_xattrs {
            entry
                .metadata_findings
                .push(crate::metadata::model::CompatibilityFinding {
                    endpoint: role,
                    path_display: crate::discovery::model::SafePath::from_path(&path).display,
                    path_raw_hex: None,
                    field: crate::metadata::model::MetadataDimension::ExtendedAttribute,
                    required: crate::discovery::model::SafePath::from_bytes(name).display,
                    evidence_state: "observed".into(),
                    reason: crate::metadata::model::CompatibilityReason::ExcludedXattr,
                    message: "extended attribute is explicitly excluded from synchronization"
                        .into(),
                    corrective_choice:
                        "no action is required; Grip leaves this attribute unmanaged".into(),
                    blocking: false,
                });
        }
        for name in &observed.unknown_xattrs {
            entry.blocking = true;
            entry
                .metadata_findings
                .push(crate::metadata::model::CompatibilityFinding {
                    endpoint: role,
                    path_display: crate::discovery::model::SafePath::from_path(&path).display,
                    path_raw_hex: None,
                    field: crate::metadata::model::MetadataDimension::ExtendedAttribute,
                    required: crate::discovery::model::SafePath::from_bytes(name).display,
                    evidence_state: "unsupported".into(),
                    reason: crate::metadata::model::CompatibilityReason::UnknownXattr,
                    message: "unknown extended attribute blocks the selected mutation scope".into(),
                    corrective_choice:
                        "remove the unknown attribute explicitly or exclude this entry from the operation"
                            .into(),
                    blocking: true,
                });
        }
    }
    entry.metadata_findings.sort_by(|first, second| {
        first
            .endpoint
            .cmp(&second.endpoint)
            .then(first.path_display.cmp(&second.path_display))
            .then(first.required.cmp(&second.required))
    });
}

fn legacy_from_complete(
    observed: &model::CompleteObservedState,
) -> (model::SupportedState, model::DiagnosticEvidence) {
    (
        model::SupportedState {
            node_kind: observed.state.node_kind,
            content: observed.state.content.clone(),
            permission_mode: (observed.state.node_kind == NodeKind::File)
                .then(|| observed.state.metadata.permission_mode.clone()),
        },
        model::DiagnosticEvidence {
            modified_seconds: observed.state.metadata.modified_time.seconds,
            modified_nanoseconds: i64::from(observed.state.metadata.modified_time.nanoseconds),
        },
    )
}

fn inspect_complete_supported(
    path: &std::path::Path,
) -> Result<Option<model::CompleteObservedState>, GripError> {
    let metadata = match crate::discovery::filesystem::metadata_at_path(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(GripError::from_io("could not inspect observed path", error)),
    };
    let kind = metadata.classify(metadata.stat.st_dev as u64);
    if !matches!(kind, NodeKind::File | NodeKind::Directory) {
        return Ok(None);
    }
    fingerprint::inspect_complete(path, kind).map(Some)
}

fn inspect_destination_leaf(
    path: &std::path::Path,
) -> Result<fingerprint::RelativeCompleteObservation, GripError> {
    let metadata = match crate::discovery::filesystem::metadata_at_path(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(fingerprint::RelativeCompleteObservation::Missing);
        }
        Err(error) => return Err(GripError::from_io("could not inspect observed path", error)),
    };
    if metadata.classify(metadata.stat.st_dev as u64) == NodeKind::Symlink {
        return Ok(
            fingerprint::RelativeCompleteObservation::DestinationLeafLink(
                fingerprint::destination_leaf_link(metadata),
            ),
        );
    }
    match inspect_complete_supported(path)? {
        Some(complete) => Ok(fingerprint::RelativeCompleteObservation::Supported(
            Box::new(complete),
        )),
        None => Ok(fingerprint::RelativeCompleteObservation::Blocking {
            reason: "wrong_node_kind",
        }),
    }
}

fn observe_complete_identity(
    entry: &mut ObservedEntry,
    source: bool,
    expected_kind: NodeKind,
    inspectors: &mut BTreeMap<(ResolvedMapping, bool), Option<fingerprint::RelativeInspector>>,
) -> Result<(), GripError> {
    let identity = &entry.identity;
    let outcome = if identity.mapping.kind == MappingKind::Tree {
        let root = if source {
            &identity.mapping.source
        } else {
            &identity.mapping.destination
        };
        let inspector = match inspectors.entry((identity.mapping.clone(), source)) {
            std::collections::btree_map::Entry::Occupied(entry) => entry.into_mut(),
            std::collections::btree_map::Entry::Vacant(entry) => {
                let value = match crate::discovery::filesystem::metadata_at_path(root) {
                    Ok(_) => Some(fingerprint::RelativeInspector::open(root)?),
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
                    Err(error) => {
                        return Err(GripError::from_io(
                            "could not inspect mapped tree root",
                            error,
                        ));
                    }
                };
                entry.insert(value)
            }
        };
        match inspector {
            Some(inspector) => {
                inspector.inspect_complete(&identity.relative_path, expected_kind)?
            }
            None => fingerprint::RelativeCompleteObservation::Missing,
        }
    } else {
        let path = if source {
            identity.source_path()
        } else {
            identity.destination_path()
        };
        if source {
            match inspect_complete_supported(&path)? {
                Some(complete) => {
                    fingerprint::RelativeCompleteObservation::Supported(Box::new(complete))
                }
                None => fingerprint::RelativeCompleteObservation::Missing,
            }
        } else {
            inspect_destination_leaf(&path)?
        }
    };
    match outcome {
        fingerprint::RelativeCompleteObservation::Missing => {}
        fingerprint::RelativeCompleteObservation::Supported(complete) => {
            let (state, diagnostic) = legacy_from_complete(&complete);
            if source {
                entry.source = Some(state);
                entry.source_diagnostic = Some(diagnostic);
                entry.source_complete = Some(*complete);
            } else {
                entry.destination = Some(state);
                entry.destination_diagnostic = Some(diagnostic);
                entry.destination_complete = Some(*complete);
            }
        }
        fingerprint::RelativeCompleteObservation::DestinationLeafLink(evidence) => {
            if source {
                entry.blocking = true;
                entry.unsupported.push("symlink".into());
            } else {
                entry.destination_link = Some(evidence);
            }
        }
        fingerprint::RelativeCompleteObservation::Blocking { reason } => {
            entry.blocking = true;
            entry.unsupported.push(if source {
                reason.into()
            } else {
                format!("destination:{reason}")
            });
        }
    }
    Ok(())
}

fn inspect_identity(
    identity: &EntryIdentity,
    source: bool,
    inspectors: &mut BTreeMap<(ResolvedMapping, bool), fingerprint::RelativeInspector>,
) -> Result<Option<(model::SupportedState, model::DiagnosticEvidence)>, GripError> {
    let path = if source {
        identity.source_path()
    } else {
        identity.destination_path()
    };
    let metadata = match crate::discovery::filesystem::metadata_at_path(&path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(GripError::from_io("could not inspect observed path", error)),
    };
    let kind = metadata.classify(metadata.stat.st_dev as u64);
    if !matches!(kind, NodeKind::File | NodeKind::Directory) {
        return Ok(None);
    }
    if identity.mapping.kind == MappingKind::Tree {
        let root = if source {
            &identity.mapping.source
        } else {
            &identity.mapping.destination
        };
        let inspector = match inspectors.entry((identity.mapping.clone(), source)) {
            std::collections::btree_map::Entry::Occupied(entry) => entry.into_mut(),
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(fingerprint::RelativeInspector::open(root)?)
            }
        };
        inspector.inspect(&identity.relative_path, kind).map(Some)
    } else {
        fingerprint::inspect(&path, kind).map(Some)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery::model::{DiscoveryInventory, DiscoveryRecord, DiscoveryScope, SafePath};
    use std::path::PathBuf;

    fn file_mapping() -> Mapping {
        Mapping::new(
            MappingKind::File,
            PathBuf::from("/source/file"),
            PathBuf::from("/destination/file"),
        )
    }

    fn inventory(category: RecordCategory, mapping_kind: MappingKind) -> DiscoveryInventory {
        DiscoveryInventory::new(
            DiscoveryScope::All,
            vec![DiscoveryRecord {
                category,
                mapping_kind,
                mapping_source: PathBuf::from("/source/file"),
                relative_path: None,
                source_path: Some(SafePath::from_path(std::path::Path::new("/source/file"))),
                destination_path: SafePath::from_path(std::path::Path::new("/destination/file")),
                node_kind: NodeKind::File,
                reason: None,
                relation: None,
                blocking: false,
            }],
        )
    }

    #[test]
    fn eligible_file_discovery_record_reuses_the_pass_observation() {
        assert!(file_mapping_is_discovery_backed(
            &file_mapping(),
            &inventory(RecordCategory::Eligible, MappingKind::File)
        ));
    }

    #[test]
    fn noneligible_or_nonfile_record_keeps_direct_file_observation_fallback() {
        assert!(!file_mapping_is_discovery_backed(
            &file_mapping(),
            &inventory(RecordCategory::UnsupportedSource, MappingKind::File)
        ));
        assert!(!file_mapping_is_discovery_backed(
            &file_mapping(),
            &inventory(RecordCategory::Eligible, MappingKind::Tree)
        ));
    }
}
