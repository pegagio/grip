//! Stable source and destination observation orchestration.

pub mod fingerprint;
pub mod model;

use crate::discovery::model::{NodeKind, RecordCategory};
use crate::error::GripError;
use crate::home::GripHome;
use crate::mapping::{Mapping, MappingKind};
use crate::registry::publication::RegistrySnapshot;
use crate::state::AcceptedState;
use model::{EntryIdentity, MappingSnapshot, Membership, Observation, ObservedEntry, Selection};
use std::collections::BTreeMap;

/// Build a stable observation and join it with retained accepted identities.
pub fn inspect(
    home: &GripHome,
    registry: &RegistrySnapshot,
    accepted: &AcceptedState,
    selection: &Selection,
) -> Result<Observation, GripError> {
    let inventory = crate::discovery::inspect(home, registry, None)?;
    let first = inspect_once(registry, accepted, selection, &inventory)?;
    let second = inspect_once(registry, accepted, selection, &inventory)?;
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

fn inspect_once(
    registry: &RegistrySnapshot,
    accepted: &AcceptedState,
    selection: &Selection,
    inventory: &crate::discovery::model::DiscoveryInventory,
) -> Result<Observation, GripError> {
    let current: BTreeMap<MappingSnapshot, &Mapping> = registry
        .registry
        .mappings()
        .iter()
        .map(|mapping| (MappingSnapshot::from(mapping), mapping))
        .collect();
    let mut observed = Observation::new();
    let mut relative_inspectors = BTreeMap::new();

    for mapping in registry.registry.mappings() {
        if mapping.kind == MappingKind::File {
            let identity = EntryIdentity::new(MappingSnapshot::from(mapping), Vec::new())
                .expect("file mapping identity is valid");
            let source_complete = inspect_complete_supported(&mapping.source)?;
            let destination_complete = inspect_complete_supported(&mapping.destination)?;
            let source = source_complete.as_ref().map(legacy_from_complete);
            let destination = destination_complete.as_ref().map(legacy_from_complete);
            if source.is_some()
                || destination.is_some()
                || accepted.baselines.contains_key(&identity)
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
                        metadata_findings: Vec::new(),
                        endpoint_capabilities: Vec::new(),
                        source_diagnostic: source.map(|value| value.1),
                        destination_diagnostic: destination.map(|value| value.1),
                        unsupported: Vec::new(),
                        blocking: false,
                    },
                );
            }
        }
    }

    for record in &inventory.records {
        let mapping = registry
            .registry
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
        let identity = EntryIdentity::new(MappingSnapshot::from(mapping), relative)
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
                entry.source_complete = inspect_complete_identity(&identity, true)?;
                entry.destination_complete = inspect_complete_identity(&identity, false)?;
                if let Some(complete) = &entry.source_complete {
                    let (state, diagnostic) = legacy_from_complete(complete);
                    entry.source = Some(state);
                    entry.source_diagnostic = Some(diagnostic);
                }
                if let Some(complete) = &entry.destination_complete {
                    let (state, diagnostic) = legacy_from_complete(complete);
                    entry.destination = Some(state);
                    entry.destination_diagnostic = Some(diagnostic);
                }
            }
            RecordCategory::Ignored => {
                entry.membership = Membership::Ignored;
                if accepted.baselines.contains_key(&identity)
                    || accepted.complete_baselines.contains_key(&identity)
                {
                    entry.source_complete = inspect_complete_identity(&identity, true)?;
                    entry.destination_complete = inspect_complete_identity(&identity, false)?;
                    if let Some(complete) = &entry.source_complete {
                        let (state, diagnostic) = legacy_from_complete(complete);
                        entry.source = Some(state);
                        entry.source_diagnostic = Some(diagnostic);
                    }
                    if let Some(complete) = &entry.destination_complete {
                        let (state, diagnostic) = legacy_from_complete(complete);
                        entry.destination = Some(state);
                        entry.destination_diagnostic = Some(diagnostic);
                    }
                }
            }
            RecordCategory::DestinationOnly => {
                if entry.membership != Membership::Ignored {
                    entry.membership = Membership::DestinationOnly;
                }
                if matches!(record.node_kind, NodeKind::File | NodeKind::Directory) {
                    entry.destination_complete = inspect_complete_identity(&identity, false)?;
                    if let Some(complete) = &entry.destination_complete {
                        let (state, diagnostic) = legacy_from_complete(complete);
                        entry.destination = Some(state);
                        entry.destination_diagnostic = Some(diagnostic);
                    }
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
            }
        }
    }

    for identity in accepted
        .baselines
        .keys()
        .chain(accepted.complete_baselines.keys())
    {
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
        MappingSnapshot,
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
    observed.retain(|identity, entry| {
        selection.includes(identity)
            && !(matches!(selection, Selection::All | Selection::Mapping(_))
                && entry.membership == Membership::Ignored
                && !accepted.baselines.contains_key(identity)
                && !accepted.complete_baselines.contains_key(identity))
    });
    Ok(observed)
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

fn inspect_complete_identity(
    identity: &EntryIdentity,
    source: bool,
) -> Result<Option<model::CompleteObservedState>, GripError> {
    let path = if source {
        identity.source_path()
    } else {
        identity.destination_path()
    };
    inspect_complete_supported(&path)
}

fn inspect_identity(
    identity: &EntryIdentity,
    source: bool,
    inspectors: &mut BTreeMap<(MappingSnapshot, bool), fingerprint::RelativeInspector>,
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
