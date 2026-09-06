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
            let source = inspect_supported(&mapping.source)?;
            let destination = inspect_supported(&mapping.destination)?;
            if source.is_some()
                || destination.is_some()
                || accepted.baselines.contains_key(&identity)
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
                source_diagnostic: None,
                destination_diagnostic: None,
                unsupported: Vec::new(),
                blocking: false,
            });
        match record.category {
            RecordCategory::Eligible => {
                entry.membership = Membership::Active;
                if let Some((state, diagnostic)) =
                    inspect_identity(&identity, true, &mut relative_inspectors)?
                {
                    entry.source = Some(state);
                    entry.source_diagnostic = Some(diagnostic);
                }
                if let Some((state, diagnostic)) =
                    inspect_identity(&identity, false, &mut relative_inspectors)?
                {
                    entry.destination = Some(state);
                    entry.destination_diagnostic = Some(diagnostic);
                }
            }
            RecordCategory::Ignored => entry.membership = Membership::Ignored,
            RecordCategory::DestinationOnly => {
                if entry.membership != Membership::Ignored {
                    entry.membership = Membership::DestinationOnly;
                }
                if matches!(record.node_kind, NodeKind::File | NodeKind::Directory)
                    && let Some((state, diagnostic)) =
                        inspect_identity(&identity, false, &mut relative_inspectors)?
                {
                    entry.destination = Some(state);
                    entry.destination_diagnostic = Some(diagnostic);
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

    for identity in accepted.baselines.keys() {
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
                source_diagnostic: None,
                destination_diagnostic: None,
                unsupported: Vec::new(),
                blocking: false,
            });
        if !current.contains_key(&identity.mapping) {
            entry.membership = Membership::Untracked;
        }
    }

    observed.retain(|identity, entry| {
        selection.includes(identity)
            && !(matches!(selection, Selection::All | Selection::Mapping(_))
                && entry.membership == Membership::Ignored
                && !accepted.baselines.contains_key(identity))
    });
    Ok(observed)
}

fn inspect_supported(
    path: &std::path::Path,
) -> Result<Option<(model::SupportedState, model::DiagnosticEvidence)>, GripError> {
    let metadata = match crate::discovery::filesystem::metadata_at_path(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(GripError::from_io("could not inspect observed path", error)),
    };
    let kind = metadata.classify(metadata.stat.st_dev as u64);
    if !matches!(kind, NodeKind::File | NodeKind::Directory) {
        return Ok(None);
    }
    fingerprint::inspect(path, kind).map(Some)
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
