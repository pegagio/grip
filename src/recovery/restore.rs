//! Exact bound-target recovery restoration.

use crate::discovery::model::NodeKind;
use crate::error::GripError;
use crate::observation::model::SupportedState;
use crate::project::ProjectPaths;
use crate::recovery::model::{
    RecoveryAvailability, RecoveryKind, RecoveryRef, RestoreAction, RestorePlan,
};
use sha2::{Digest, Sha256};
use std::path::{Component, Path, PathBuf};

#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestoreFault {
    Preservation,
    Staging,
    Publication,
    Verification,
}

pub fn plan(home: &ProjectPaths, reference: &RecoveryRef) -> Result<RestorePlan, GripError> {
    if !reference.has_recoverable_bytes() {
        return Err(GripError::InvalidConfiguration(
            "operation recovery references cannot be restored".into(),
        ));
    }
    let entry = crate::recovery::inventory::show(home, reference)?;
    let mut blockers = Vec::new();
    if entry.availability != RecoveryAvailability::Available {
        blockers.push(format!("recovery_is_{:?}", entry.availability).to_lowercase());
    }
    if entry.restore_eligibility != "eligible" {
        blockers.push(entry.restore_eligibility.clone());
    }
    let target = entry.bound_path.clone();
    let mut expected_current = serde_json::Value::Null;
    let mut displacement_recovery = "not_required".to_owned();
    let mut authority_evidence = serde_json::json!({});
    if entry.kind == RecoveryKind::Payload && target.is_none() {
        blockers.push("bound_target_missing".into());
    }
    if entry.kind == RecoveryKind::Payload
        && let Some(target_value) = target.as_deref()
    {
        let located = crate::recovery::inventory::locate_manifest(home, reference)?;
        let target_path = PathBuf::from(target_value);
        if let Err(error) = validate_target(&target_path) {
            blockers.push(format!("unsafe_target: {error}"));
        } else {
            match std::fs::symlink_metadata(&target_path) {
                Ok(metadata) if metadata.file_type().is_symlink() => {
                    blockers.push("unsafe_target".into());
                }
                Ok(_) if located.payload.expected_post_evidence.is_null() => {
                    blockers.push("occupied_target_without_post_evidence".into());
                }
                Ok(_) => {
                    let post = serde_json::from_value::<SupportedState>(
                        located.payload.expected_post_evidence.clone(),
                    )
                    .map_err(|error| {
                        GripError::CorruptState(format!("invalid post payload evidence: {error}"))
                    })?;
                    match crate::observation::fingerprint::inspect(&target_path, post.node_kind) {
                        Ok((actual, _)) if actual == post => {
                            expected_current = serde_json::to_value(actual).map_err(|error| {
                                GripError::Internal(format!(
                                    "could not encode restore target evidence: {error}"
                                ))
                            })?;
                            displacement_recovery = "required".into();
                        }
                        _ => blockers.push("occupied_target_changed".into()),
                    }
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => blockers.push(format!("target_unavailable: {error}")),
            }
        }
    }
    match entry.kind {
        RecoveryKind::Payload => {
            authority_evidence = payload_authority_evidence(home, reference, target.as_deref())?;
        }
        RecoveryKind::Registry => {
            authority_evidence = registry_compatibility(home, reference, &mut blockers)?;
        }
        RecoveryKind::AcceptedState => {
            authority_evidence = state_compatibility(home, reference, &mut blockers)?;
        }
        RecoveryKind::Operation => {}
    }
    let projection = serde_json::to_vec(&(
        reference,
        &target,
        entry.kind,
        &expected_current,
        &displacement_recovery,
        &authority_evidence,
        &blockers,
    ))
    .map_err(|error| GripError::Internal(format!("could not encode restore plan: {error}")))?;
    Ok(RestorePlan {
        operation: "recovery_restore",
        plan_id: format!("{:x}", Sha256::digest(projection)),
        reference: reference.clone(),
        target: target.clone(),
        kind: entry.kind,
        expected_current,
        displacement_recovery,
        authority_evidence,
        actions: vec![RestoreAction {
            index: 0,
            reference: reference.clone(),
            target,
            status: "unattempted".into(),
            milestones: checkpoint("not_attempted", false),
            failure: None,
        }],
        blockers,
        status: "unattempted".into(),
        operation_record: None,
    })
}

fn payload_authority_evidence(
    home: &ProjectPaths,
    reference: &RecoveryRef,
    target: Option<&str>,
) -> Result<serde_json::Value, GripError> {
    let located = crate::recovery::inventory::locate_manifest(home, reference)?;
    let portable_identity =
        located.payload.identity.as_ref().ok_or_else(|| {
            GripError::CorruptState("payload recovery identity is missing".into())
        })?;
    let identity = crate::state::runtime_identity_from_portable(home, portable_identity)?;
    let expected_target = match located.payload.endpoint_role {
        Some(crate::state::EndpointRoleV1::Source) => identity.source_path(),
        Some(crate::state::EndpointRoleV1::Destination) => identity.destination_path(),
        _ => {
            return Err(GripError::CorruptState(
                "payload recovery has no valid bound side".into(),
            ));
        }
    };
    let recovery_bytes = crate::mutation::filesystem::read_private_file(
        &located.directory.join("recovery-v2.json"),
    )?;
    let recovery = crate::mutation::recovery::decode_mutation_recovery_v2(&recovery_bytes)?;
    if recovery.payload.identity != *portable_identity
        || Some(recovery.payload.endpoint_role) != located.payload.endpoint_role
        || recovery.payload.private_ref != located.payload.private_ref
    {
        return Err(GripError::CorruptState(
            "mutation recovery and manifest bindings disagree".into(),
        ));
    }
    if target.map(Path::new) != Some(expected_target.as_path()) {
        return Err(GripError::CorruptState(
            "payload recovery target does not match its managed identity".into(),
        ));
    }
    let registry = crate::registry::publication::load(home, false)?;
    if !registry.registry.mappings().iter().any(|mapping| {
        identity.mapping == crate::observation::model::ResolvedMapping::from(mapping)
    }) {
        return Err(GripError::InvalidConfiguration(
            "payload recovery mapping is not present in the live registry".into(),
        ));
    }
    let state = crate::state::publication::load(home)?;
    Ok(serde_json::json!({
        "registry_sha256": format!("{:x}", Sha256::digest(&registry.bytes)),
        "accepted_generation": state.accepted.generation,
        "mapping": identity.mapping,
    }))
}

fn registry_compatibility(
    home: &ProjectPaths,
    reference: &RecoveryRef,
    blockers: &mut Vec<String>,
) -> Result<serde_json::Value, GripError> {
    let located = crate::recovery::inventory::locate_manifest(home, reference)?;
    let bytes = crate::mutation::filesystem::read_private_file(&located.payload_path)?;
    let RecoveryRef::Registry { digest } = reference else {
        unreachable!()
    };
    if format!("{:x}", Sha256::digest(&bytes)) != *digest {
        return Err(GripError::CorruptState(
            "registry recovery digest mismatch".into(),
        ));
    }
    let recovered = recovered_registry(home, &bytes)?;
    let state = crate::state::publication::load(home)?;
    for identity in state.accepted.complete_baselines.keys() {
        if !recovered.mappings().iter().any(|mapping| {
            identity.mapping == crate::observation::model::ResolvedMapping::from(mapping)
        }) {
            blockers.push("recovered_registry_incompatible_with_accepted_state".into());
        }
    }
    if blockers.is_empty()
        && crate::registry::publication::validate_recovered_compatibility(
            home,
            &recovered,
            &bytes,
            &state.accepted,
        )
        .is_err()
    {
        blockers.push("recovered_registry_live_payload_incompatible".into());
    }
    let post_digest = located
        .payload
        .expected_post_evidence
        .get("sha256")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| GripError::CorruptState("registry recovery lacks post digest".into()))?;
    if let Ok(current) = crate::registry::publication::load(home, false)
        && format!("{:x}", Sha256::digest(&current.bytes)) != post_digest
    {
        blockers.push("current_registry_differs_from_recorded_post_state".into());
    }
    Ok(serde_json::json!({
        "candidate_sha256": digest,
        "accepted_generation": state.accepted.generation,
        "expected_post_sha256": post_digest,
    }))
}

fn state_compatibility(
    home: &ProjectPaths,
    reference: &RecoveryRef,
    blockers: &mut Vec<String>,
) -> Result<serde_json::Value, GripError> {
    let located = crate::recovery::inventory::locate_manifest(home, reference)?;
    let bytes = crate::mutation::filesystem::read_private_file(&located.payload_path)?;
    let recovered_v4 = crate::state::decode_v4(&bytes)?;
    let recovered_runtime = crate::state::runtime_from_accepted_v4(home, &recovered_v4)?;
    let recovered = runtime_accepted_state(recovered_runtime);
    let RecoveryRef::AcceptedState { generation, digest } = reference else {
        unreachable!()
    };
    if recovered_v4.generation != *generation || format!("{:x}", Sha256::digest(&bytes)) != *digest
    {
        return Err(GripError::CorruptState(
            "state recovery identity mismatch".into(),
        ));
    }
    let registry = crate::registry::publication::load(home, false)?;
    for identity in recovered.complete_baselines.keys() {
        if !registry.registry.mappings().iter().any(|mapping| {
            identity.mapping == crate::observation::model::ResolvedMapping::from(mapping)
        }) {
            blockers.push("recovered_state_references_missing_mapping".into());
        }
    }
    if blockers.is_empty()
        && crate::observation::inspect(
            home,
            &registry,
            &recovered,
            &crate::observation::model::Selection::All,
        )
        .is_err()
    {
        blockers.push("recovered_state_live_payload_incompatible".into());
    }
    let post_generation = located
        .payload
        .expected_post_evidence
        .get("generation")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| GripError::CorruptState("state recovery lacks post generation".into()))?;
    let post_digest = located
        .payload
        .expected_post_evidence
        .get("sha256")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| GripError::CorruptState("state recovery lacks post digest".into()))?;
    if let Ok(current) = crate::state::publication::load(home) {
        let current_digest = current
            .bytes
            .as_deref()
            .map(|value| format!("{:x}", Sha256::digest(value)));
        if current.accepted.generation != Some(post_generation)
            || current_digest.as_deref() != Some(post_digest)
        {
            blockers.push("current_state_differs_from_recorded_post_state".into());
        }
    }
    Ok(serde_json::json!({
        "candidate_generation": generation,
        "candidate_sha256": digest,
        "registry_sha256": format!("{:x}", Sha256::digest(&registry.bytes)),
        "expected_post_generation": post_generation,
        "expected_post_sha256": post_digest,
    }))
}

pub fn execute(home: &ProjectPaths, expected: &RestorePlan) -> Result<RestorePlan, GripError> {
    execute_with_fault(home, expected, None)
}

#[doc(hidden)]
pub fn execute_with_fault(
    home: &ProjectPaths,
    expected: &RestorePlan,
    fault: Option<RestoreFault>,
) -> Result<RestorePlan, GripError> {
    let _guard = crate::state::mutation_lock::MutationLock::acquire(home, "recovery_restore")?;
    let rebuilt = plan(home, &expected.reference)?;
    if rebuilt.plan_id != expected.plan_id || !rebuilt.blockers.is_empty() {
        return Err(GripError::InvalidConfiguration(
            "recovery restore plan changed before execution".into(),
        ));
    }
    match expected.kind {
        RecoveryKind::Payload => restore_payload(home, expected, fault),
        RecoveryKind::Registry => restore_registry(home, expected, fault),
        RecoveryKind::AcceptedState => restore_state(home, expected, fault),
        RecoveryKind::Operation => unreachable!(),
    }
}

fn restore_registry(
    home: &ProjectPaths,
    expected: &RestorePlan,
    fault: Option<RestoreFault>,
) -> Result<RestorePlan, GripError> {
    let located = crate::recovery::inventory::locate_manifest(home, &expected.reference)?;
    let bytes = crate::mutation::filesystem::read_private_file(&located.payload_path)?;
    let digest = format!("{:x}", Sha256::digest(&bytes));
    let RecoveryRef::Registry {
        digest: expected_digest,
    } = &expected.reference
    else {
        unreachable!()
    };
    if &digest != expected_digest {
        return Err(GripError::CorruptState(
            "registry recovery digest mismatch".into(),
        ));
    }
    let recovered_registry = recovered_registry(home, &bytes)?;
    let state = crate::state::publication::load(home)?;
    for identity in state.accepted.complete_baselines.keys() {
        if !recovered_registry.mappings().iter().any(|mapping| {
            identity.mapping == crate::observation::model::ResolvedMapping::from(mapping)
        }) {
            return Err(GripError::InvalidConfiguration(
                "recovered registry is incompatible with live accepted state".into(),
            ));
        }
    }
    let post = located
        .payload
        .expected_post_evidence
        .get("sha256")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| {
            GripError::CorruptState("registry recovery lacks post-transition evidence".into())
        })?;
    let mut receipt = crate::operation::publication::initialize_typed(
        home,
        "recovery_restore",
        &expected.plan_id,
        expected,
        1,
    )?;
    let mut result = expected.clone();
    result.status = "in_progress".into();
    result.actions[0].status = "in_progress".into();
    result.actions[0].milestones = checkpoint("not_attempted", false);
    if let Err(error) =
        receipt.checkpoint_action(0, "in_progress", result.actions[0].milestones.clone(), None)
    {
        return restore_fail(
            &mut receipt,
            &mut result,
            "operation_checkpoint",
            error,
            false,
            "not_attempted",
            false,
        );
    }
    if fault == Some(RestoreFault::Publication) {
        return restore_fail(
            &mut receipt,
            &mut result,
            "registry_publication",
            GripError::Internal("injected registry restore publication failure".into()),
            false,
            "not_attempted",
            false,
        );
    }
    if let Err(error) = crate::registry::publication::restore_exact(home, &bytes, post) {
        return restore_fail(
            &mut receipt,
            &mut result,
            "registry_publication",
            error,
            false,
            "not_attempted",
            false,
        );
    }
    result.actions[0].status = "completed".into();
    result.actions[0].milestones = checkpoint("visible", true);
    if let Err(error) =
        receipt.checkpoint_action(0, "completed", result.actions[0].milestones.clone(), None)
    {
        return restore_fail(
            &mut receipt,
            &mut result,
            "operation_checkpoint",
            error,
            true,
            "verified",
            true,
        );
    }
    if let Err(error) = receipt.checkpoint_summary(
        "completed",
        serde_json::json!({"outcome":"registry_restored"}),
        "prepared",
        None,
    ) {
        return restore_fail(
            &mut receipt,
            &mut result,
            "operation_checkpoint",
            error,
            true,
            "verified",
            true,
        );
    }
    result.status = "completed".into();
    result.operation_record = Some(receipt.operation_id().into());
    Ok(result)
}

fn restore_state(
    home: &ProjectPaths,
    expected: &RestorePlan,
    fault: Option<RestoreFault>,
) -> Result<RestorePlan, GripError> {
    let located = crate::recovery::inventory::locate_manifest(home, &expected.reference)?;
    let bytes = crate::mutation::filesystem::read_private_file(&located.payload_path)?;
    let recovered_v4 = crate::state::decode_v4(&bytes)?;
    let recovered_runtime = crate::state::runtime_from_accepted_v4(home, &recovered_v4)?;
    let recovered = runtime_accepted_state(recovered_runtime);
    let RecoveryRef::AcceptedState { generation, digest } = &expected.reference else {
        unreachable!()
    };
    if recovered_v4.generation != *generation || format!("{:x}", Sha256::digest(&bytes)) != *digest
    {
        return Err(GripError::CorruptState(
            "state recovery identity mismatch".into(),
        ));
    }
    let post_generation = located
        .payload
        .expected_post_evidence
        .get("generation")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| GripError::CorruptState("state recovery lacks post generation".into()))?;
    let post_digest = located
        .payload
        .expected_post_evidence
        .get("sha256")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| GripError::CorruptState("state recovery lacks post digest".into()))?;
    let registry = crate::registry::publication::load(home, false)?;
    for identity in recovered.complete_baselines.keys() {
        if !registry.registry.mappings().iter().any(|mapping| {
            identity.mapping == crate::observation::model::ResolvedMapping::from(mapping)
        }) {
            return Err(GripError::InvalidConfiguration(
                "recovered state references a mapping absent from the live registry".into(),
            ));
        }
    }
    crate::observation::inspect(
        home,
        &registry,
        &recovered,
        &crate::observation::model::Selection::All,
    )?;
    let mut receipt = crate::operation::publication::initialize_typed(
        home,
        "recovery_restore",
        &expected.plan_id,
        expected,
        1,
    )?;
    let mut result = expected.clone();
    result.status = "in_progress".into();
    result.actions[0].status = "in_progress".into();
    result.actions[0].milestones = checkpoint("not_attempted", false);
    if let Err(error) =
        receipt.checkpoint_action(0, "in_progress", result.actions[0].milestones.clone(), None)
    {
        return restore_fail(
            &mut receipt,
            &mut result,
            "operation_checkpoint",
            error,
            false,
            "not_attempted",
            false,
        );
    }
    if fault == Some(RestoreFault::Publication) {
        return restore_fail(
            &mut receipt,
            &mut result,
            "state_publication",
            GripError::Internal("injected state restore publication failure".into()),
            false,
            "not_attempted",
            false,
        );
    }
    if let Err(error) =
        crate::state::publication::restore_exact(home, &bytes, post_generation, post_digest)
    {
        return restore_fail(
            &mut receipt,
            &mut result,
            "state_publication",
            error,
            false,
            "not_attempted",
            false,
        );
    }
    result.actions[0].status = "completed".into();
    result.actions[0].milestones = checkpoint("visible", true);
    if let Err(error) =
        receipt.checkpoint_action(0, "completed", result.actions[0].milestones.clone(), None)
    {
        return restore_fail(
            &mut receipt,
            &mut result,
            "operation_checkpoint",
            error,
            true,
            "verified",
            true,
        );
    }
    if let Err(error) = receipt.checkpoint_summary(
        "completed",
        serde_json::json!({"outcome":"state_restored","authoritative_generation":generation}),
        "prepared",
        None,
    ) {
        return restore_fail(
            &mut receipt,
            &mut result,
            "operation_checkpoint",
            error,
            true,
            "verified",
            true,
        );
    }
    result.status = "completed".into();
    result.operation_record = Some(receipt.operation_id().into());
    Ok(result)
}

fn recovered_registry(
    home: &ProjectPaths,
    bytes: &[u8],
) -> Result<crate::registry::ResolvedRegistry, GripError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| GripError::CorruptState("recovered descriptor is not UTF-8".into()))?;
    let descriptor = crate::registry::decode_descriptor(text)?;
    let project_root = home.project_root()?;
    let destination_home = home.destination_home().ok_or_else(|| {
        GripError::InvalidConfiguration("project destination home is unavailable".into())
    })?;
    let mappings = descriptor
        .resolve(project_root, destination_home, "recovery_restore")?
        .iter()
        .map(crate::mapping::ResolvedMapping::ownership_mapping)
        .collect();
    crate::registry::ResolvedRegistry::new(mappings)
}

fn runtime_accepted_state(state: crate::state::AcceptedStateV3) -> crate::state::AcceptedState {
    crate::state::AcceptedState {
        generation: Some(state.generation),
        complete_baselines: state.baselines,
        accepted_bytes: state.accepted_bytes,
    }
}

fn restore_payload(
    home: &ProjectPaths,
    expected: &RestorePlan,
    fault: Option<RestoreFault>,
) -> Result<RestorePlan, GripError> {
    let located = crate::recovery::inventory::locate_manifest(home, &expected.reference)?;
    let target =
        PathBuf::from(expected.target.as_deref().ok_or_else(|| {
            GripError::CorruptState("payload recovery has no bound target".into())
        })?);
    validate_target(&target)?;
    let prior: SupportedState = serde_json::from_value(located.payload.prior_evidence.clone())
        .map_err(|error| {
            GripError::CorruptState(format!("invalid prior payload evidence: {error}"))
        })?;
    prior
        .validate()
        .map_err(|message| GripError::CorruptState(message.into()))?;
    let complete_recovery = load_complete_recovery(&located, &expected.reference, &target, home)?;
    let post = if located.payload.expected_post_evidence.is_null() {
        None
    } else {
        Some(
            serde_json::from_value::<SupportedState>(
                located.payload.expected_post_evidence.clone(),
            )
            .map_err(|error| {
                GripError::CorruptState(format!("invalid post payload evidence: {error}"))
            })?,
        )
    };
    let occupied = match std::fs::symlink_metadata(&target) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() {
                return Err(GripError::InvalidConfiguration(
                    "restore target is a symbolic link".into(),
                ));
            }
            let post = post.as_ref().ok_or_else(|| {
                GripError::InvalidConfiguration(
                    "occupied restore target has no recorded post-action evidence".into(),
                )
            })?;
            let actual = crate::observation::fingerprint::inspect(&target, post.node_kind)?.0;
            if &actual != post {
                return Err(GripError::InvalidConfiguration(
                    "occupied restore target differs from recorded post-action evidence".into(),
                ));
            }
            Some(actual)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => {
            return Err(GripError::from_io(
                "could not inspect restore target",
                error,
            ));
        }
    };
    let in_place_complete_restore = occupied.as_ref().is_some_and(|current| {
        complete_recovery.is_some()
            && current.node_kind == prior.node_kind
            && current.content == prior.content
    });
    let mut receipt = crate::operation::publication::initialize_typed(
        home,
        "recovery_restore",
        &expected.plan_id,
        expected,
        1,
    )?;
    let mut result = expected.clone();
    result.status = "in_progress".into();
    result.actions[0].status = "in_progress".into();
    result.actions[0].milestones = checkpoint("not_attempted", false);
    if let Err(error) =
        receipt.checkpoint_action(0, "in_progress", result.actions[0].milestones.clone(), None)
    {
        return restore_fail(
            &mut receipt,
            &mut result,
            "operation_checkpoint",
            error,
            false,
            "not_attempted",
            false,
        );
    }
    if let Some(current) = occupied.as_ref() {
        if fault == Some(RestoreFault::Preservation) {
            return restore_fail(
                &mut receipt,
                &mut result,
                "preservation",
                GripError::Internal("injected restore preservation failure".into()),
                false,
                "not_attempted",
                false,
            );
        }
        let identity = match located
            .payload
            .identity
            .as_ref()
            .ok_or_else(|| GripError::CorruptState("payload recovery identity is missing".into()))
            .and_then(|identity| crate::state::runtime_identity_from_portable(home, identity))
        {
            Ok(value) => value,
            Err(error) => {
                return restore_fail(
                    &mut receipt,
                    &mut result,
                    "preservation",
                    error,
                    false,
                    "not_attempted",
                    false,
                );
            }
        };
        let displaced = if in_place_complete_restore {
            let current_complete =
                match crate::observation::fingerprint::inspect_complete(&target, current.node_kind)
                {
                    Ok(value) => value.state,
                    Err(error) => {
                        return restore_fail(
                            &mut receipt,
                            &mut result,
                            "preservation",
                            error,
                            false,
                            "not_attempted",
                            false,
                        );
                    }
                };
            match crate::mutation::recovery::preserve_complete(
                &receipt,
                0,
                &identity,
                &target,
                current,
                &current_complete,
                &complete_recovery.as_ref().unwrap().0.payload.prior_state,
            ) {
                Ok(value) => value,
                Err(error) => {
                    return restore_fail(
                        &mut receipt,
                        &mut result,
                        "preservation",
                        error,
                        false,
                        "not_attempted",
                        false,
                    );
                }
            }
        } else {
            match crate::mutation::recovery::preserve(&receipt, 0, &identity, &target, current) {
                Ok(value) => value,
                Err(error) => {
                    return restore_fail(
                        &mut receipt,
                        &mut result,
                        "preservation",
                        error,
                        false,
                        "not_attempted",
                        false,
                    );
                }
            }
        };
        result.actions[0].milestones.recovery = "preserved".into();
        result.actions[0].milestones.recovery_ref = Some(displaced.relative_ref);
        if !in_place_complete_restore {
            if let Err(error) = crate::mutation::filesystem::remove_verified(&target, current) {
                return restore_fail(
                    &mut receipt,
                    &mut result,
                    "displacement",
                    error,
                    false,
                    "not_attempted",
                    false,
                );
            }
            result.actions[0].milestones.publication = "displaced".into();
        }
    }
    if in_place_complete_restore {
        let (complete, origin) = complete_recovery.as_ref().unwrap();
        if let Err(error) = crate::metadata::macos::apply_recovery_metadata_paths(
            origin,
            &target,
            complete.payload.prior_state.node_kind,
            &complete.payload.prior_state.metadata,
        ) {
            return restore_fail(
                &mut receipt,
                &mut result,
                "metadata_application",
                GripError::from_io("could not restore complete metadata", error),
                true,
                "not_attempted",
                false,
            );
        }
        result.actions[0].milestones.publication = "visible".into();
        if fault == Some(RestoreFault::Verification) {
            return restore_fail(
                &mut receipt,
                &mut result,
                "verification",
                GripError::Internal("injected restore verification failure".into()),
                true,
                "failed",
                true,
            );
        }
        if let Err(error) = verify_restored_target(&target, &prior, complete_recovery.as_ref()) {
            return restore_fail(
                &mut receipt,
                &mut result,
                "verification",
                error,
                true,
                "failed",
                true,
            );
        }
    } else {
        match prior.node_kind {
            NodeKind::File => {
                if fault == Some(RestoreFault::Staging) {
                    return restore_fail(
                        &mut receipt,
                        &mut result,
                        "staging",
                        GripError::Internal("injected restore staging failure".into()),
                        false,
                        "not_attempted",
                        false,
                    );
                }
                let mut staged = match crate::mutation::filesystem::stage_file_for(
                    crate::mutation::model::MutationDirection::Push,
                    &located.payload_path,
                    &target,
                    &prior,
                ) {
                    Ok(value) => value,
                    Err(error) => {
                        return restore_fail(
                            &mut receipt,
                            &mut result,
                            "staging",
                            error,
                            false,
                            "not_attempted",
                            false,
                        );
                    }
                };
                result.actions[0].milestones.staging = "verified".into();
                if fault == Some(RestoreFault::Publication) {
                    return restore_fail(
                        &mut receipt,
                        &mut result,
                        "publication",
                        GripError::Internal("injected restore publication failure".into()),
                        false,
                        "not_attempted",
                        false,
                    );
                }
                if let Err(error) =
                    crate::mutation::filesystem::publish_addition(&mut staged, &target)
                {
                    return restore_fail(
                        &mut receipt,
                        &mut result,
                        "publication",
                        error,
                        false,
                        "not_attempted",
                        false,
                    );
                }
                result.actions[0].milestones.publication = "visible".into();
                if let Some((complete, origin)) = complete_recovery.as_ref()
                    && let Err(error) = crate::metadata::macos::apply_recovery_metadata_paths(
                        origin,
                        &target,
                        complete.payload.prior_state.node_kind,
                        &complete.payload.prior_state.metadata,
                    )
                {
                    return restore_fail(
                        &mut receipt,
                        &mut result,
                        "metadata_application",
                        GripError::from_io("could not restore complete metadata", error),
                        true,
                        "not_attempted",
                        false,
                    );
                }
                if fault == Some(RestoreFault::Verification) {
                    return restore_fail(
                        &mut receipt,
                        &mut result,
                        "verification",
                        GripError::Internal("injected restore verification failure".into()),
                        true,
                        "failed",
                        true,
                    );
                }
                if let Err(error) =
                    verify_restored_target(&target, &prior, complete_recovery.as_ref())
                {
                    return restore_fail(
                        &mut receipt,
                        &mut result,
                        "verification",
                        error,
                        true,
                        "failed",
                        true,
                    );
                }
            }
            NodeKind::Directory => {
                if let Err(error) = crate::mutation::filesystem::create_directory(&target) {
                    return restore_fail(
                        &mut receipt,
                        &mut result,
                        "publication",
                        error,
                        false,
                        "not_attempted",
                        false,
                    );
                }
                result.actions[0].milestones.publication = "visible".into();
                if let Some((complete, origin)) = complete_recovery.as_ref()
                    && let Err(error) = crate::metadata::macos::apply_recovery_metadata_paths(
                        origin,
                        &target,
                        complete.payload.prior_state.node_kind,
                        &complete.payload.prior_state.metadata,
                    )
                {
                    return restore_fail(
                        &mut receipt,
                        &mut result,
                        "metadata_application",
                        GripError::from_io("could not restore complete metadata", error),
                        true,
                        "not_attempted",
                        false,
                    );
                }
                if fault == Some(RestoreFault::Verification) {
                    return restore_fail(
                        &mut receipt,
                        &mut result,
                        "verification",
                        GripError::Internal("injected restore verification failure".into()),
                        true,
                        "failed",
                        true,
                    );
                }
                if let Err(error) =
                    verify_restored_target(&target, &prior, complete_recovery.as_ref())
                {
                    return restore_fail(
                        &mut receipt,
                        &mut result,
                        "verification",
                        error,
                        true,
                        "failed",
                        true,
                    );
                }
            }
            _ => {
                return restore_fail(
                    &mut receipt,
                    &mut result,
                    "publication",
                    GripError::CorruptState("recovery describes unsupported payload state".into()),
                    false,
                    "not_attempted",
                    false,
                );
            }
        }
    }
    result.actions[0].status = "completed".into();
    result.actions[0].milestones.publication = "visible".into();
    result.actions[0].milestones.verification = "verified".into();
    result.actions[0].milestones.durability_confirmed = true;
    if let Err(error) =
        receipt.checkpoint_action(0, "completed", result.actions[0].milestones.clone(), None)
    {
        return restore_fail(
            &mut receipt,
            &mut result,
            "operation_checkpoint",
            error,
            true,
            "verified",
            true,
        );
    }
    if let Err(error) = receipt.checkpoint_summary(
        "completed",
        serde_json::json!({"outcome":"not_accepted"}),
        "prepared",
        None,
    ) {
        return restore_fail(
            &mut receipt,
            &mut result,
            "operation_checkpoint",
            error,
            true,
            "verified",
            true,
        );
    }
    result.status = "completed".into();
    result.operation_record = Some(receipt.operation_id().into());
    Ok(result)
}

fn load_complete_recovery(
    located: &crate::recovery::inventory::LocatedRecovery,
    reference: &RecoveryRef,
    target: &Path,
    home: &ProjectPaths,
) -> Result<Option<(crate::recovery::model::RecoveryMetadataV3, PathBuf)>, GripError> {
    let path = located.directory.join("metadata-v3.json");
    match std::fs::symlink_metadata(&path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => {}
        Ok(_) => {
            return Err(GripError::CorruptState(
                "complete recovery metadata is not a regular file".into(),
            ));
        }
        Err(error) => {
            return Err(GripError::from_io(
                "could not inspect complete recovery metadata",
                error,
            ));
        }
    }
    let bytes = crate::mutation::filesystem::read_private_file(&path)?;
    let metadata = crate::recovery::model::decode_metadata_v3(&bytes)?;
    let RecoveryRef::Payload {
        operation_id,
        action_index,
    } = reference
    else {
        return Err(GripError::CorruptState(
            "complete payload recovery has a non-payload reference".into(),
        ));
    };
    if metadata.payload.operation_id != *operation_id
        || metadata.payload.action_index != *action_index
    {
        return Err(GripError::CorruptState(
            "complete recovery metadata does not match its public reference".into(),
        ));
    }
    let identity = crate::state::runtime_identity_from_portable(home, &metadata.payload.identity)?;
    if target != identity.source_path() && target != identity.destination_path() {
        return Err(GripError::CorruptState(
            "complete recovery metadata does not match its bound target".into(),
        ));
    }
    let payload_ref = metadata.payload.payload_ref.as_deref().ok_or_else(|| {
        GripError::CorruptState("complete recovery metadata has no private metadata object".into())
    })?;
    let origin = located.directory.join(payload_ref);
    if !origin.is_file() {
        return Err(GripError::CorruptState(
            "complete recovery private metadata object is unavailable".into(),
        ));
    }
    Ok(Some((metadata, origin)))
}

fn verify_restored_target(
    target: &Path,
    prior: &SupportedState,
    complete: Option<&(crate::recovery::model::RecoveryMetadataV3, PathBuf)>,
) -> Result<(), GripError> {
    crate::mutation::filesystem::verify_target(target, prior)?;
    if let Some((metadata, _)) = complete {
        let observed = crate::observation::fingerprint::inspect_complete(
            target,
            metadata.payload.prior_state.node_kind,
        )?;
        if observed.state != metadata.payload.prior_state {
            return Err(GripError::CorruptState(
                "restored complete state does not match Recovery Metadata V2".into(),
            ));
        }
    }
    Ok(())
}

fn validate_target(path: &Path) -> Result<(), GripError> {
    if !path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir | Component::CurDir))
    {
        return Err(GripError::CorruptState(
            "recovery target path is unsafe".into(),
        ));
    }
    let parent = path
        .parent()
        .ok_or_else(|| GripError::CorruptState("recovery target has no parent".into()))?;
    let metadata = std::fs::symlink_metadata(parent)
        .map_err(|error| GripError::from_io("restore target parent is unavailable", error))?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(GripError::InvalidConfiguration(
            "restore target ancestry is unsafe".into(),
        ));
    }
    Ok(())
}

fn checkpoint(
    publication: &str,
    durable: bool,
) -> crate::operation::model::ActionCheckpointEvidenceV2 {
    crate::operation::model::ActionCheckpointEvidenceV2 {
        revalidation: "passed".into(),
        recovery: "preserved".into(),
        recovery_ref: None,
        staging: if durable { "verified" } else { "not_attempted" }.into(),
        publication: publication.into(),
        verification: if durable { "verified" } else { "not_attempted" }.into(),
        durability_confirmed: durable,
    }
}

#[allow(clippy::too_many_arguments)]
fn restore_fail(
    receipt: &mut crate::operation::publication::OperationReceipt,
    plan: &mut RestorePlan,
    phase: &str,
    error: GripError,
    publication_visible: bool,
    verification: &str,
    durability_confirmed: bool,
) -> Result<RestorePlan, GripError> {
    plan.status = if publication_visible {
        "partial"
    } else {
        "failed"
    }
    .into();
    if let Some(action) = plan.actions.first_mut() {
        action.status = "failed".into();
        action.failure = Some(error.to_string());
        action.milestones.publication = if publication_visible {
            "visible"
        } else {
            "not_visible"
        }
        .into();
        action.milestones.verification = verification.into();
        action.milestones.durability_confirmed = durability_confirmed;
        let _ = receipt.checkpoint_action(
            action.index,
            "failed",
            action.milestones.clone(),
            Some(error.to_string()),
        );
    }
    let authority = serde_json::json!({
        "outcome":"not_accepted",
        "publication_visible":publication_visible,
        "verification":verification,
        "durability_confirmed":durability_confirmed,
    });
    let _ = receipt.checkpoint_summary(
        "failed",
        authority.clone(),
        "prepared",
        Some(format!("{phase}: {error}")),
    );
    let mut details = serde_json::Map::new();
    details.insert("mode".into(), "execute".into());
    details.insert("completion".into(), plan.status.clone().into());
    details.insert("result".into(), "failed".into());
    details.insert(
        "plan".into(),
        serde_json::to_value(&*plan).unwrap_or(serde_json::Value::Null),
    );
    details.insert("authority".into(), authority);
    details.insert(
        "failure".into(),
        serde_json::json!({"phase":phase,"message":error.to_string()}),
    );
    Err(GripError::operation_lifecycle(
        "recovery_restore",
        &format!("{phase}_failure"),
        crate::error::ResultCategory::InternalError,
        receipt.operation_id(),
        details,
        publication_visible,
        verification,
        durability_confirmed,
        format!("recovery restore failed during {phase}: {error}"),
    ))
}
