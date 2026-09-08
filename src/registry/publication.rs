//! Safe loading and atomic publication of the user-authored registry.

use super::{Registry, decode, decode_mappings, encode};
use crate::error::GripError;
use crate::home::GripHome;
use crate::path_policy::{self, PathEvidence};
use crate::state::lock::PublicationLock;
use rustix::fs::{CWD, Mode, OFlags, RenameFlags, open, renameat_with};
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt, PermissionsExt, symlink};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static TEMP_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FileIdentity {
    device: u64,
    inode: u64,
    mode: u32,
}

/// An accepted registry plus the evidence needed to reject stale publication.
#[derive(Debug, Clone)]
pub struct RegistrySnapshot {
    pub registry: Registry,
    pub bytes: Vec<u8>,
    pub mode: u32,
    identity: FileIdentity,
    evidence: Vec<PathEvidence>,
}

impl RegistrySnapshot {
    /// Return missing destination parents captured during validated registry loading.
    pub fn missing_destination_parents(&self) -> Vec<crate::push::plan::ParentRequirement> {
        let mut requirements = Vec::new();
        for mapping in &self.registry.mappings {
            if let Some(evidence) = self
                .evidence
                .iter()
                .find(|evidence| !evidence.source && evidence.canonical == mapping.destination)
            {
                requirements.extend(evidence.missing_destination_parents().into_iter().map(
                    |path| crate::push::plan::ParentRequirement {
                        mapping: crate::observation::model::MappingSnapshot::from(mapping),
                        path,
                    },
                ));
            }
        }
        requirements
    }
}

/// Validate a recovered registry against live endpoints and accepted state without publishing it.
pub fn validate_recovered_compatibility(
    home: &GripHome,
    registry: &Registry,
    bytes: &[u8],
    accepted: &crate::state::AcceptedState,
) -> Result<(), GripError> {
    let evidence = inspect_mappings(registry.mappings(), "recovery_restore")?;
    let _ = (home, bytes, evidence);
    for (identity, baseline) in &accepted.baselines {
        for path in [identity.source_path(), identity.destination_path()] {
            match std::fs::symlink_metadata(&path) {
                Ok(metadata) if metadata.file_type().is_symlink() => {
                    return Err(GripError::InvalidConfiguration(
                        "recovered registry resolves an accepted identity through a symbolic link"
                            .into(),
                    ));
                }
                Ok(_) => {
                    crate::observation::fingerprint::inspect(&path, baseline.node_kind)?;
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => {
                    return Err(GripError::from_io(
                        "could not inspect recovered registry payload compatibility",
                        error,
                    ));
                }
            }
        }
    }
    for (identity, baseline) in &accepted.complete_baselines {
        for path in [identity.source_path(), identity.destination_path()] {
            match std::fs::symlink_metadata(&path) {
                Ok(metadata) if metadata.file_type().is_symlink() => {
                    return Err(GripError::InvalidConfiguration(
                        "recovered registry resolves an accepted identity through a symbolic link"
                            .into(),
                    ));
                }
                Ok(_) => {
                    crate::observation::fingerprint::inspect_complete(&path, baseline.node_kind)?;
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => {
                    return Err(GripError::from_io(
                        "could not inspect recovered registry payload compatibility",
                        error,
                    ));
                }
            }
        }
    }
    Ok(())
}

/// Held registry publication guard used to enforce registry-before-state ordering.
pub struct RegistryGuard {
    _lock: PublicationLock,
}

/// Acquire the bounded registry publication lock for a compound state transaction.
pub fn acquire_guard(home: &GripHome, operation: &str) -> Result<RegistryGuard, GripError> {
    PublicationLock::acquire(&home.path().join(".registry.lock"))
        .map(|lock| RegistryGuard { _lock: lock })
        .map_err(|error| match error {
            GripError::StateContention => GripError::mapping(
                operation,
                "registry_contention",
                vec![home.path().join(".registry.lock").display().to_string()],
                "registry publication is already in progress",
            ),
            other => other,
        })
}

struct AcceptedFile {
    bytes: Vec<u8>,
    identity: FileIdentity,
}

struct StagedFile {
    path: PathBuf,
    file: File,
    device: u64,
    inode: u64,
    expected_mode: u32,
}

#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PublicationFault {
    BeforeRecoveryRename,
    CorruptStagedCandidate,
    SubstituteRecoveryStagingWithSymlink,
    SubstituteRegistryStagingWithDirectory,
    SubstituteRegistryStagingWithFile,
    SubstituteRegistryStagingWithSymlink,
    BeforeFinalRevalidation,
    BeforeRegistryRename,
    RegistryRename,
    DirectorySync,
}

fn mapping_error(reason: &str, message: impl Into<String>) -> GripError {
    GripError::mapping("mapping_update", reason, Vec::new(), &message.into())
}

fn publication_error(message: impl Into<String>, visible: bool) -> GripError {
    mapping_error("publication_failure", message).with_publication_visible(visible)
}

fn accepted_identity(metadata: &fs::Metadata) -> FileIdentity {
    FileIdentity {
        device: metadata.dev(),
        inode: metadata.ino(),
        mode: metadata.permissions().mode() & 0o7777,
    }
}

fn accepted_metadata_is_safe(metadata: &fs::Metadata, writable: bool) -> bool {
    let mode = metadata.permissions().mode() & 0o7777;
    !metadata.file_type().is_symlink()
        && metadata.is_file()
        && metadata.uid() == rustix::process::geteuid().as_raw()
        && mode & 0o400 != 0
        && mode & 0o022 == 0
        && (!writable || mode & 0o200 != 0)
}

fn unsafe_registry_error(message: &str) -> GripError {
    mapping_error("unsafe_registry_mode", message)
}

fn ensure_owned_dir(path: &Path) -> Result<(), GripError> {
    match fs::symlink_metadata(path) {
        Ok(metadata)
            if metadata.is_dir()
                && !metadata.file_type().is_symlink()
                && metadata.uid() == rustix::process::geteuid().as_raw()
                && metadata.permissions().mode() & 0o777 == 0o700 =>
        {
            Ok(())
        }
        Ok(_) => Err(GripError::CorruptState(format!(
            "unsafe registry recovery directory {}",
            path.display()
        ))),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let mut builder = fs::DirBuilder::new();
            builder.mode(0o700);
            builder.create(path).map_err(|error| {
                publication_error(
                    format!("could not create registry recovery directory: {error}"),
                    false,
                )
            })?;
            fs::set_permissions(path, fs::Permissions::from_mode(0o700)).map_err(|error| {
                publication_error(
                    format!("could not secure registry recovery directory: {error}"),
                    false,
                )
            })
        }
        Err(error) => Err(publication_error(
            format!("could not inspect registry recovery directory: {error}"),
            false,
        )),
    }
}

fn read_accepted(home: &GripHome, writable: bool) -> Result<AcceptedFile, GripError> {
    let path = home.path().join("config.toml");
    let path_metadata = fs::symlink_metadata(&path).map_err(|error| match error.kind() {
        std::io::ErrorKind::NotFound => {
            GripError::InvalidConfiguration("config.toml is unavailable".into())
        }
        _ => GripError::RegistryIo(format!("could not inspect config.toml: {error}")),
    })?;
    if !accepted_metadata_is_safe(&path_metadata, writable) {
        return Err(unsafe_registry_error(
            "config.toml must be a readable current-user-owned non-symlink regular file with a safe permission mode",
        ));
    }
    let path_identity = accepted_identity(&path_metadata);
    let descriptor = open(
        &path,
        OFlags::RDONLY | OFlags::CLOEXEC | OFlags::NOFOLLOW | OFlags::NONBLOCK,
        Mode::empty(),
    )
    .map_err(|error| match error {
        rustix::io::Errno::NOENT => {
            GripError::InvalidConfiguration("config.toml is unavailable".into())
        }
        rustix::io::Errno::LOOP => unsafe_registry_error(
            "config.toml must be a current-user-owned non-symlink regular file with a safe permission mode",
        ),
        _ => match fs::symlink_metadata(&path) {
            Ok(current)
                if !accepted_metadata_is_safe(&current, writable)
                    || accepted_identity(&current) != path_identity =>
            {
                unsafe_registry_error("config.toml changed to an unsafe node during inspection")
            }
            _ => GripError::RegistryIo(format!("could not open config.toml: {error}")),
        },
    })?;
    let mut file = File::from(descriptor);
    let metadata = file.metadata().map_err(|error| {
        GripError::RegistryIo(format!("could not inspect config.toml: {error}"))
    })?;
    let identity = accepted_identity(&metadata);
    if identity != path_identity || !accepted_metadata_is_safe(&metadata, writable) {
        return Err(unsafe_registry_error(
            "config.toml changed during inspection or is not a readable current-user-owned regular file with a safe permission mode",
        ));
    }
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .map_err(|error| GripError::RegistryIo(format!("could not read config.toml: {error}")))?;
    Ok(AcceptedFile { bytes, identity })
}

/// Read and completely validate the accepted registry and its safety metadata.
pub fn load(home: &GripHome, writable: bool) -> Result<RegistrySnapshot, GripError> {
    let accepted = read_accepted(home, writable)?;
    let text = std::str::from_utf8(&accepted.bytes)
        .map_err(|_| GripError::InvalidConfiguration("config.toml must be UTF-8 TOML".into()))?;
    let mappings = decode_mappings(text)?;
    let evidence = inspect_mappings(&mappings, "registry_validate")?;
    let (endpoint_pairs, remainder) = evidence.as_chunks::<2>();
    debug_assert!(remainder.is_empty());
    let canonical_mappings: Vec<_> = mappings
        .iter()
        .zip(endpoint_pairs)
        .map(|(mapping, endpoints)| {
            crate::mapping::Mapping::new(
                mapping.kind,
                endpoints[0].canonical.clone(),
                endpoints[1].canonical.clone(),
            )
        })
        .collect();
    let canonical_registry = Registry::new(canonical_mappings.clone())?;
    for (stored, canonical) in mappings.iter().zip(&canonical_mappings) {
        for (stored_path, canonical_path) in [
            (&stored.source, &canonical.source),
            (&stored.destination, &canonical.destination),
        ] {
            if stored_path != canonical_path {
                return Err(GripError::mapping(
                    "registry_validate",
                    "unsafe_ancestry",
                    vec![
                        stored_path.display().to_string(),
                        canonical_path.display().to_string(),
                    ],
                    "registry mapping paths must use their canonical identities",
                ));
            }
        }
    }
    Ok(RegistrySnapshot {
        registry: canonical_registry,
        bytes: accepted.bytes,
        mode: accepted.identity.mode,
        identity: accepted.identity,
        evidence,
    })
}

/// Revalidate an accepted snapshot without acquiring a publication lock or writing state.
pub fn revalidate_readonly(
    home: &GripHome,
    expected: &RegistrySnapshot,
    operation: &str,
) -> Result<(), GripError> {
    for evidence in &expected.evidence {
        path_policy::revalidate(evidence, operation).map_err(|error| {
            GripError::discovery_operational(
                operation,
                "stale_discovery_evidence",
                vec![evidence.canonical.display().to_string()],
                &error.to_string(),
            )
        })?;
    }
    let current = read_accepted(home, false).map_err(|error| {
        GripError::discovery_operational(
            operation,
            "stale_discovery_evidence",
            vec![home.path().join("config.toml").display().to_string()],
            &error.to_string(),
        )
    })?;
    if current.bytes != expected.bytes || current.identity != expected.identity {
        return Err(GripError::discovery_operational(
            operation,
            "stale_discovery_evidence",
            vec![home.path().join("config.toml").display().to_string()],
            "Accepted registry changed during inspection",
        ));
    }
    Ok(())
}

fn inspect_mappings(
    mappings: &[crate::mapping::Mapping],
    operation: &str,
) -> Result<Vec<PathEvidence>, GripError> {
    let mut evidence = Vec::with_capacity(mappings.len() * 2);
    for mapping in mappings {
        evidence.push(path_policy::inspect_durable_endpoint(
            &mapping.source,
            mapping.kind,
            true,
            operation,
        )?);
        evidence.push(path_policy::inspect_durable_endpoint(
            &mapping.destination,
            mapping.kind,
            false,
            operation,
        )?);
    }
    Ok(evidence)
}

fn stage_bytes(directory: &Path, prefix: &str, bytes: &[u8]) -> Result<StagedFile, GripError> {
    let temporary = directory.join(format!(
        ".{prefix}.tmp-{}-{}",
        std::process::id(),
        TEMP_ID.fetch_add(1, Ordering::Relaxed)
    ));
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&temporary)
        .map_err(|error| {
            publication_error(format!("could not stage registry data: {error}"), false)
        })?;
    let metadata = file.metadata().map_err(|error| {
        publication_error(
            format!("could not inspect staged registry data: {error}"),
            false,
        )
    })?;
    let mut staged = StagedFile {
        path: temporary,
        file,
        device: metadata.dev(),
        inode: metadata.ino(),
        expected_mode: 0o600,
    };
    if let Err(error) = staged
        .file
        .write_all(bytes)
        .and_then(|_| staged.file.sync_all())
    {
        cleanup_staged(&staged);
        return Err(publication_error(
            format!("could not sync staged registry data: {error}"),
            false,
        ));
    }
    Ok(staged)
}

fn verify_staged_bytes(staged: &mut StagedFile, expected: &[u8]) -> Result<Vec<u8>, GripError> {
    staged.file.seek(SeekFrom::Start(0)).map_err(|error| {
        publication_error(
            format!("could not rewind staged registry data: {error}"),
            false,
        )
    })?;
    let mut bytes = Vec::new();
    staged.file.read_to_end(&mut bytes).map_err(|error| {
        publication_error(
            format!("could not verify staged registry data: {error}"),
            false,
        )
    })?;
    if bytes != expected {
        return Err(publication_error(
            "staged registry byte verification failed",
            false,
        ));
    }
    Ok(bytes)
}

fn set_staged_mode(staged: &mut StagedFile, mode: u32) -> Result<(), GripError> {
    staged
        .file
        .set_permissions(fs::Permissions::from_mode(mode))
        .and_then(|_| staged.file.sync_all())
        .map_err(|error| {
            publication_error(
                format!("could not secure staged registry data: {error}"),
                false,
            )
        })?;
    staged.expected_mode = mode;
    Ok(())
}

fn staged_path_matches(staged: &StagedFile, require_mode: bool) -> bool {
    fs::symlink_metadata(&staged.path).is_ok_and(|metadata| {
        metadata.is_file()
            && !metadata.file_type().is_symlink()
            && metadata.uid() == rustix::process::geteuid().as_raw()
            && metadata.dev() == staged.device
            && metadata.ino() == staged.inode
            && (!require_mode || metadata.permissions().mode() & 0o7777 == staged.expected_mode)
    })
}

fn verify_staged_path(staged: &StagedFile) -> Result<(), GripError> {
    let metadata = staged.file.metadata().map_err(|error| {
        publication_error(
            format!("could not inspect staged registry descriptor: {error}"),
            false,
        )
    })?;
    if !metadata.is_file()
        || metadata.uid() != rustix::process::geteuid().as_raw()
        || metadata.dev() != staged.device
        || metadata.ino() != staged.inode
        || metadata.permissions().mode() & 0o7777 != staged.expected_mode
        || !staged_path_matches(staged, true)
    {
        return Err(publication_error(
            "staged registry pathname no longer identifies the attempt-owned file",
            false,
        ));
    }
    Ok(())
}

fn cleanup_staged(staged: &StagedFile) {
    if staged_path_matches(staged, false) {
        let _ = fs::remove_file(&staged.path);
    }
}

fn substitute_staging(staged: &StagedFile, fault: PublicationFault) -> Result<(), GripError> {
    fs::remove_file(&staged.path).map_err(|error| {
        publication_error(
            format!("could not inject staged-path substitution: {error}"),
            false,
        )
    })?;
    match fault {
        PublicationFault::SubstituteRecoveryStagingWithSymlink
        | PublicationFault::SubstituteRegistryStagingWithSymlink => {
            symlink("config.toml", &staged.path)
        }
        PublicationFault::SubstituteRegistryStagingWithDirectory => fs::create_dir(&staged.path),
        PublicationFault::SubstituteRegistryStagingWithFile => OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&staged.path)
            .map(drop),
        _ => unreachable!("substitution helper requires a substitution fault"),
    }
    .map_err(|error| {
        publication_error(
            format!("could not inject staged-path substitution: {error}"),
            false,
        )
    })
}

fn publish_recovery_file(
    directory: &Path,
    target: &Path,
    bytes: &[u8],
    fault: Option<PublicationFault>,
) -> Result<(), GripError> {
    reject_unexpected_staging(directory)?;
    let mut staged = stage_bytes(directory, "config", bytes)?;
    let result = (|| {
        verify_staged_bytes(&mut staged, bytes)?;
        set_staged_mode(&mut staged, 0o600)?;
        if fault == Some(PublicationFault::SubstituteRecoveryStagingWithSymlink) {
            substitute_staging(&staged, fault.unwrap())?;
        }
        if fault == Some(PublicationFault::BeforeRecoveryRename) {
            return Err(mapping_error(
                "registry_recovery_failure",
                "injected pre-rename registry recovery failure",
            ));
        }
        verify_staged_path(&staged)?;
        renameat_with(CWD, &staged.path, CWD, target, RenameFlags::NOREPLACE).map_err(|error| {
            mapping_error(
                "registry_recovery_failure",
                format!("could not publish registry recovery: {error}"),
            )
        })?;
        File::open(directory)
            .and_then(|file| file.sync_all())
            .map_err(|error| {
                mapping_error(
                    "registry_recovery_failure",
                    format!("could not sync registry recovery directory: {error}"),
                )
            })
    })();
    if result.is_err() {
        cleanup_staged(&staged);
    }
    result
}

fn read_recovery_file(path: &Path) -> Result<Vec<u8>, GripError> {
    let descriptor = open(
        path,
        OFlags::RDONLY | OFlags::CLOEXEC | OFlags::NOFOLLOW,
        Mode::empty(),
    )
    .map_err(|error| {
        mapping_error(
            "registry_recovery_failure",
            format!("could not open registry recovery safely: {error}"),
        )
    })?;
    let mut file = File::from(descriptor);
    let metadata = file.metadata().map_err(|error| {
        mapping_error(
            "registry_recovery_failure",
            format!("could not inspect registry recovery: {error}"),
        )
    })?;
    if !metadata.is_file()
        || metadata.uid() != rustix::process::geteuid().as_raw()
        || metadata.permissions().mode() & 0o777 != 0o600
    {
        return Err(mapping_error(
            "registry_recovery_failure",
            "registry recovery generation is unsafe",
        ));
    }
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).map_err(|error| {
        mapping_error(
            "registry_recovery_failure",
            format!("could not read registry recovery: {error}"),
        )
    })?;
    Ok(bytes)
}

fn retain_recovery(
    home: &GripHome,
    bytes: &[u8],
    expected_post: &[u8],
    fault: Option<PublicationFault>,
) -> Result<PathBuf, GripError> {
    let state = home.path().join("state");
    let recovery = state.join("recovery");
    let registry = recovery.join("registry");
    for directory in [&state, &recovery, &registry] {
        ensure_owned_dir(directory)?;
    }
    let digest = format!("{:x}", Sha256::digest(bytes));
    let generation = registry.join(format!("sha256-{digest}"));
    ensure_owned_dir(&generation)?;
    let target = generation.join("config.toml");
    let created = !target.exists();
    if target.exists() {
        if read_recovery_file(&target)? != bytes {
            return Err(mapping_error(
                "registry_recovery_failure",
                "registry recovery generation conflicts with accepted bytes",
            ));
        }
    } else {
        publish_recovery_file(&generation, &target, bytes, fault)?;
    }
    let recovered = read_recovery_file(&target)?;
    if Sha256::digest(&recovered) != Sha256::digest(bytes) || recovered != bytes {
        return Err(mapping_error(
            "registry_recovery_failure",
            "registry recovery verification failed",
        ));
    }
    let manifest_path = generation.join("manifest.json");
    if created && !manifest_path.exists() {
        let manifest = crate::recovery::model::RecoveryEnvelopeV1::new(
            crate::recovery::model::RecoveryManifestPayloadV1 {
                reference: crate::recovery::model::RecoveryRef::Registry {
                    digest: digest.clone(),
                },
                kind: crate::recovery::model::RecoveryKind::Registry,
                created_at: recovery_timestamp(),
                origin_operation: None,
                origin_transition: "registry_publication".into(),
                managed_identity: None,
                bound_side: None,
                bound_target: None,
                prior_evidence: serde_json::json!({"sha256": digest}),
                expected_post_evidence: serde_json::json!({"sha256": format!("{:x}", Sha256::digest(expected_post))}),
                byte_count: bytes.len() as u64,
                payload_ref: "config.toml".into(),
            },
        )?;
        crate::operation::publication::publish_new_component(
            &generation,
            "manifest.json",
            &crate::recovery::model::encode(&manifest)?,
        )?;
    }
    Ok(target)
}

fn recovery_timestamp() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| format!("{}Z", duration.as_secs()))
        .unwrap_or_else(|_| "0Z".into())
}

fn reject_unexpected_staging(directory: &Path) -> Result<(), GripError> {
    for entry in fs::read_dir(directory).map_err(|error| {
        publication_error(
            format!("could not inspect staging directory: {error}"),
            false,
        )
    })? {
        let entry = entry.map_err(|error| {
            publication_error(format!("could not inspect staging entry: {error}"), false)
        })?;
        if entry
            .file_name()
            .to_string_lossy()
            .starts_with(".config.tmp-")
        {
            return Err(GripError::CorruptState(format!(
                "unexpected registry staging node {}",
                entry.path().display()
            )));
        }
    }
    Ok(())
}

fn verify_final_evidence(
    home: &GripHome,
    expected: &RegistrySnapshot,
    candidate_evidence: &[PathEvidence],
) -> Result<(), GripError> {
    for evidence in expected.evidence.iter().chain(candidate_evidence) {
        path_policy::revalidate(evidence, "mapping_update")
            .map_err(|error| mapping_error("stale_path_evidence", error.to_string()))?;
    }
    let current = read_accepted(home, true).map_err(|error| match error {
        GripError::Mapping { .. } => mapping_error("stale_path_evidence", error.to_string()),
        GripError::InvalidConfiguration(_) => mapping_error("stale_registry", error.to_string()),
        other => other,
    })?;
    if current.bytes != expected.bytes || current.identity != expected.identity {
        return Err(mapping_error(
            "stale_registry",
            "accepted registry changed before publication",
        ));
    }
    Ok(())
}

fn validate_candidate_evidence(
    expected: &RegistrySnapshot,
    candidate: &Registry,
    evidence: &[PathEvidence],
) -> Result<(), GripError> {
    let added = candidate
        .mappings()
        .iter()
        .filter(|mapping| !expected.registry.mappings().contains(mapping))
        .collect::<Vec<_>>();
    let (endpoint_pairs, remainder) = evidence.as_chunks::<2>();
    let matches_candidate = remainder.is_empty()
        && endpoint_pairs.len() == added.len()
        && added
            .iter()
            .zip(endpoint_pairs)
            .all(|(mapping, endpoints)| {
                endpoints[0].source
                    && !endpoints[1].source
                    && endpoints[0].kind == mapping.kind
                    && endpoints[1].kind == mapping.kind
                    && endpoints[0].canonical == mapping.source
                    && endpoints[1].canonical == mapping.destination
            });
    if !matches_candidate {
        return Err(mapping_error(
            "stale_path_evidence",
            "candidate endpoint evidence does not match the proposed registry",
        ));
    }
    Ok(())
}

/// Publish a complete candidate only when the accepted evidence is unchanged.
pub fn publish(
    home: &GripHome,
    expected: &RegistrySnapshot,
    candidate: &Registry,
) -> Result<(), GripError> {
    publish_with_fault(home, expected, candidate, None)
}

/// Restore exact, validated Registry V1 bytes while the caller holds the mutation lock.
pub(crate) fn restore_exact(
    home: &GripHome,
    bytes: &[u8],
    expected_post_digest: &str,
) -> Result<(), GripError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| GripError::CorruptState("recovered registry is not UTF-8".into()))?;
    let candidate = decode(text)?;
    inspect_mappings(candidate.mappings(), "recovery_restore")?;
    if let Ok(current) = load(home, false)
        && format!("{:x}", Sha256::digest(&current.bytes)) != expected_post_digest
    {
        return Err(mapping_error(
            "restore_authority_changed",
            "current valid registry differs from recorded post-transition authority",
        ));
    }
    let _lock = PublicationLock::acquire(&home.path().join(".registry.lock"))?;
    let mut staged = stage_bytes(home.path(), "config", bytes)?;
    verify_staged_bytes(&mut staged, bytes)?;
    set_staged_mode(&mut staged, 0o600)?;
    fs::rename(&staged.path, home.path().join("config.toml"))
        .map_err(|error| GripError::from_io("could not restore registry", error))?;
    File::open(home.path())
        .and_then(|directory| directory.sync_all())
        .map_err(|error| GripError::from_io("could not sync restored registry", error))?;
    if load(home, false)?.bytes != bytes {
        return Err(GripError::CorruptState(
            "restored registry verification failed".into(),
        ));
    }
    Ok(())
}

/// Publish a complete candidate using endpoint evidence captured from submitted add paths.
pub fn publish_with_evidence(
    home: &GripHome,
    expected: &RegistrySnapshot,
    candidate: &Registry,
    candidate_evidence: &[PathEvidence],
) -> Result<(), GripError> {
    publish_candidate(home, expected, candidate, candidate_evidence, None)
}

#[doc(hidden)]
pub fn publish_with_fault(
    home: &GripHome,
    expected: &RegistrySnapshot,
    candidate: &Registry,
    fault: Option<PublicationFault>,
) -> Result<(), GripError> {
    let added = candidate
        .mappings()
        .iter()
        .filter(|mapping| !expected.registry.mappings().contains(mapping))
        .cloned()
        .collect::<Vec<_>>();
    let candidate_evidence = inspect_mappings(&added, "mapping_update")
        .map_err(|error| mapping_error("stale_path_evidence", error.to_string()))?;
    publish_candidate(home, expected, candidate, &candidate_evidence, fault)
}

fn publish_candidate(
    home: &GripHome,
    expected: &RegistrySnapshot,
    candidate: &Registry,
    candidate_evidence: &[PathEvidence],
    fault: Option<PublicationFault>,
) -> Result<(), GripError> {
    validate_candidate_evidence(expected, candidate, candidate_evidence)?;
    let lock = PublicationLock::acquire(&home.path().join(".registry.lock")).map_err(|error| {
        if matches!(error, GripError::StateContention) {
            mapping_error(
                "registry_contention",
                "registry publication is already in progress",
            )
        } else {
            error
        }
    })?;
    reject_unexpected_staging(home.path())?;
    verify_final_evidence(home, expected, candidate_evidence)?;
    let bytes = encode(candidate)?;
    retain_recovery(home, &expected.bytes, &bytes, fault)?;
    let mut staged = stage_bytes(home.path(), "config", &bytes)?;
    let result = (|| {
        if fault == Some(PublicationFault::CorruptStagedCandidate) {
            staged
                .file
                .set_len(0)
                .and_then(|_| staged.file.write_all(b"corrupt staged registry"))
                .and_then(|_| staged.file.sync_all())
                .map_err(|error| {
                    publication_error(
                        format!("could not inject staged corruption: {error}"),
                        false,
                    )
                })?;
        }
        let staged_bytes = verify_staged_bytes(&mut staged, &bytes)?;
        let decoded = std::str::from_utf8(&staged_bytes)
            .map_err(|_| publication_error("staged registry is not UTF-8", false))
            .and_then(|text| {
                decode(text).map_err(|error| {
                    publication_error(format!("staged registry is invalid: {error}"), false)
                })
            })?;
        if decoded != *candidate {
            return Err(publication_error(
                "staged registry is not semantically equal to the candidate",
                false,
            ));
        }
        set_staged_mode(&mut staged, expected.mode)?;
        if matches!(
            fault,
            Some(
                PublicationFault::SubstituteRegistryStagingWithDirectory
                    | PublicationFault::SubstituteRegistryStagingWithFile
                    | PublicationFault::SubstituteRegistryStagingWithSymlink
            )
        ) {
            substitute_staging(&staged, fault.unwrap())?;
        }
        if fault == Some(PublicationFault::BeforeFinalRevalidation) {
            return Err(publication_error(
                "injected pre-final-revalidation publication failure",
                false,
            ));
        }
        verify_final_evidence(home, expected, candidate_evidence)?;
        if fault == Some(PublicationFault::BeforeRegistryRename)
            || fault == Some(PublicationFault::RegistryRename)
        {
            return Err(publication_error("injected registry rename failure", false));
        }
        verify_staged_path(&staged)?;
        fs::rename(&staged.path, home.path().join("config.toml")).map_err(|error| {
            publication_error(format!("could not publish registry data: {error}"), false)
        })?;
        if fault == Some(PublicationFault::DirectorySync) {
            return Err(publication_error(
                "registry is visible but directory durability could not be confirmed",
                true,
            ));
        }
        File::open(home.path())
            .and_then(|file| file.sync_all())
            .map_err(|error| {
                publication_error(
                    format!(
                        "registry is visible but directory durability could not be confirmed: {error}"
                    ),
                    true,
                )
            })
    })();
    if result.is_err() {
        cleanup_staged(&staged);
    }
    drop(lock);
    result
}

#[cfg(test)]
mod discovery_tests {
    use super::*;

    fn empty_home() -> (tempfile::TempDir, GripHome) {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(
            root.path().join("config.toml"),
            "schema_version = 1\nmappings = []\n",
        )
        .unwrap();
        let home = crate::home::select(Some(root.path().as_os_str().to_owned()), None).unwrap();
        (root, home)
    }

    #[test]
    fn readonly_revalidation_does_not_acquire_the_registry_lock() {
        let (root, home) = empty_home();
        let snapshot = load(&home, false).unwrap();
        std::fs::write(root.path().join(".registry.lock"), "occupied").unwrap();
        revalidate_readonly(&home, &snapshot, "mapping_inspect").unwrap();
        assert_eq!(
            std::fs::read(root.path().join(".registry.lock")).unwrap(),
            b"occupied"
        );
    }

    #[test]
    fn readonly_revalidation_detects_registry_byte_drift() {
        let (root, home) = empty_home();
        let snapshot = load(&home, false).unwrap();
        std::fs::write(
            root.path().join("config.toml"),
            "schema_version = 1\n\nmappings = []\n",
        )
        .unwrap();
        assert!(revalidate_readonly(&home, &snapshot, "mapping_inspect").is_err());
    }
}
