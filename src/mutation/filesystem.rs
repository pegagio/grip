//! Narrow direction-neutral filesystem mutation primitives.

use crate::error::GripError;
use crate::mutation::model::MutationDirection;
use crate::observation::model::SupportedState;
use rustix::fs::{
    AtFlags, Mode, OFlags, RenameFlags, fsync, mkdirat, openat, renameat_with, unlinkat,
};
use sha2::{Digest, Sha256};
use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static STAGE_COUNTER: AtomicU64 = AtomicU64::new(0);

/// An attempt-owned verified sibling staging file.
#[derive(Debug)]
pub struct StagedFile {
    path: PathBuf,
    parent: crate::discovery::filesystem::Directory,
    name: Vec<u8>,
    destination_name: Vec<u8>,
    file: File,
    device: u64,
    inode: u64,
    published: bool,
}

impl StagedFile {
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for StagedFile {
    fn drop(&mut self) {
        if !self.published
            && self.parent.metadata(&self.name).is_ok_and(|metadata| {
                metadata.stat.st_dev as u64 == self.device && metadata.stat.st_ino == self.inode
            })
        {
            let _ = unlinkat(
                self.parent.as_fd(),
                OsStr::from_bytes(&self.name),
                AtFlags::empty(),
            );
        }
    }
}

/// Create and verify one absent directory without recursive creation.
pub fn create_directory(path: &Path) -> Result<(), GripError> {
    create_directory_with_collision(path, false)
}

pub(crate) fn create_recovery_directory(path: &Path) -> Result<(), GripError> {
    create_directory_with_collision(path, true)
}

fn create_directory_with_collision(
    path: &Path,
    collision_is_corrupt: bool,
) -> Result<(), GripError> {
    let (parent, name) = open_parent(path)?;
    if parent.metadata(&name).is_ok() {
        return Err(if collision_is_corrupt {
            GripError::CorruptState("recovery action directory already exists".into())
        } else {
            stale("mutation target appeared before directory creation")
        });
    }
    mkdirat(
        parent.as_fd(),
        OsStr::from_bytes(&name),
        Mode::from_raw_mode(0o700),
    )
    .map_err(|error| {
        if collision_is_corrupt && error == rustix::io::Errno::EXIST {
            GripError::CorruptState("recovery action directory collision".into())
        } else {
            GripError::from_io("could not create mutation target directory", error.into())
        }
    })?;
    let created = parent.open_child_directory(&name).map_err(|error| {
        GripError::mutation_side_effect(
            "verification_failure",
            true,
            "failed",
            false,
            format!("could not verify mutation target directory: {error}"),
        )
    })?;
    if created.root_metadata().st_mode & 0o777 != 0o700 {
        return Err(GripError::mutation_side_effect(
            "verification_failure",
            true,
            "failed",
            false,
            "created mutation target directory is not private",
        ));
    }
    sync_open_directory(&parent).map_err(|error| {
        GripError::mutation_side_effect(
            "publication_failure",
            true,
            "verified",
            false,
            error.to_string(),
        )
    })
}

/// Read one current-user-owned private file through no-following descriptors.
pub(crate) fn read_private_file(path: &Path) -> Result<Vec<u8>, GripError> {
    let (parent, name) = open_parent(path)?;
    let path_metadata = parent
        .metadata(&name)
        .map_err(|error| GripError::from_io("could not inspect private file", error))?;
    let descriptor = parent
        .open_child_file(&name)
        .map_err(|error| GripError::from_io("could not open private file safely", error))?;
    let mut file = File::from(descriptor);
    let metadata = file
        .metadata()
        .map_err(|error| GripError::from_io("could not inspect opened private file", error))?;
    if !metadata.is_file()
        || metadata.uid() != rustix::process::geteuid().as_raw()
        || metadata.permissions().mode() & 0o7777 != 0o600
        || metadata.dev() != path_metadata.stat.st_dev as u64
        || metadata.ino() != path_metadata.stat.st_ino
    {
        return Err(GripError::CorruptState(
            "private file identity, ownership, or mode is unsafe".into(),
        ));
    }
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .map_err(|error| GripError::from_io("could not read private file", error))?;
    Ok(bytes)
}

/// Stream a no-follow source into an exclusive verified destination sibling.
pub fn stage_file(
    source: &Path,
    destination: &Path,
    expected: &SupportedState,
) -> Result<StagedFile, GripError> {
    stage_file_for(MutationDirection::Push, source, destination, expected)
}

/// Stream an operation origin into an exclusive verified target sibling.
pub fn stage_file_for(
    direction: MutationDirection,
    origin: &Path,
    target: &Path,
    expected: &SupportedState,
) -> Result<StagedFile, GripError> {
    stage_file_with_mode(direction, origin, target, expected, None)
}

/// Stage a private recovery copy while validating the source against its captured evidence.
pub(crate) fn stage_private_file(
    source: &Path,
    destination: &Path,
    expected: &SupportedState,
) -> Result<StagedFile, GripError> {
    stage_file_with_mode(
        MutationDirection::Push,
        source,
        destination,
        expected,
        Some(0o600),
    )
}

fn stage_file_with_mode(
    direction: MutationDirection,
    source: &Path,
    destination: &Path,
    expected: &SupportedState,
    target_mode: Option<u32>,
) -> Result<StagedFile, GripError> {
    let (parent, destination_name) = open_parent(destination)?;
    let (source_parent, source_name) = open_parent(source)?;
    let source_fd = openat(
        source_parent.as_fd(),
        OsStr::from_bytes(&source_name),
        OFlags::RDONLY | OFlags::CLOEXEC | OFlags::NOFOLLOW | OFlags::NONBLOCK,
        Mode::empty(),
    )
    .map_err(|error| GripError::from_io("could not open mutation origin safely", error.into()))?;
    let mut source_file = File::from(source_fd);
    let source_before = source_file
        .metadata()
        .map_err(|error| GripError::from_io("could not inspect mutation origin", error))?;
    validate_regular(source, &source_before)?;
    let name = format!(
        ".grip-stage-{}-{}",
        std::process::id(),
        STAGE_COUNTER.fetch_add(1, Ordering::Relaxed)
    );
    let name = name.into_bytes();
    let path = destination
        .parent()
        .expect("destination has parent")
        .join(OsStr::from_bytes(&name));
    let descriptor = openat(
        parent.as_fd(),
        OsStr::from_bytes(&name),
        OFlags::RDWR | OFlags::CLOEXEC | OFlags::NOFOLLOW | OFlags::CREATE | OFlags::EXCL,
        Mode::from_raw_mode(0o600),
    )
    .map_err(|error| GripError::from_io("could not create target staging file", error.into()))?;
    let file = File::from(descriptor);
    let metadata = file
        .metadata()
        .map_err(|error| GripError::from_io("could not inspect target staging file", error))?;
    let mut staged = StagedFile {
        path,
        parent,
        name,
        destination_name,
        file,
        device: metadata.dev(),
        inode: metadata.ino(),
        published: false,
    };
    let result = (|| {
        let mut digest = Sha256::new();
        let mut length = 0_u64;
        let mut buffer = [0_u8; 64 * 1024];
        loop {
            let read = source_file
                .read(&mut buffer)
                .map_err(|error| GripError::from_io("could not read mutation origin", error))?;
            if read == 0 {
                break;
            }
            staged
                .file
                .write_all(&buffer[..read])
                .map_err(|error| GripError::from_io("could not write target staging", error))?;
            digest.update(&buffer[..read]);
            length += read as u64;
        }
        let source_after = source_file
            .metadata()
            .map_err(|error| GripError::from_io("could not revalidate mutation origin", error))?;
        if source_before.dev() != source_after.dev()
            || source_before.ino() != source_after.ino()
            || source_before.mode() != source_after.mode()
            || source_before.len() != source_after.len()
            || source_before.mtime() != source_after.mtime()
            || source_before.mtime_nsec() != source_after.mtime_nsec()
        {
            return Err(stale_for(
                direction,
                "origin changed while staging mutation content",
            ));
        }
        let actual_digest = format!("{:x}", digest.finalize());
        let expected_content = expected.content.as_ref().ok_or_else(|| {
            GripError::Internal("planned file action has no content evidence".into())
        })?;
        if length != expected_content.length || actual_digest != expected_content.digest {
            return Err(stale_for(
                direction,
                "origin content differs from the planned evidence",
            ));
        }
        let mode = target_mode.unwrap_or(
            u32::from_str_radix(
                expected.permission_mode.as_deref().ok_or_else(|| {
                    GripError::Internal("planned file action has no mode evidence".into())
                })?,
                8,
            )
            .map_err(|_| GripError::Internal("planned file mode is invalid".into()))?,
        );
        staged
            .file
            .set_permissions(fs::Permissions::from_mode(mode))
            .and_then(|_| staged.file.sync_all())
            .map_err(|error| GripError::from_io("could not prepare target staging", error))?;
        staged
            .file
            .seek(SeekFrom::Start(0))
            .map_err(|error| GripError::from_io("could not rewind target staging", error))?;
        let mut staged_digest = Sha256::new();
        let copied = std::io::copy(
            &mut staged.file,
            &mut staged_digest_writer(&mut staged_digest),
        )
        .map_err(|error| GripError::from_io("could not verify target staging", error))?;
        if copied != length || format!("{:x}", staged_digest.finalize()) != actual_digest {
            return Err(GripError::CorruptState(
                "target staging verification failed".into(),
            ));
        }
        verify_staged_identity(&staged)
    })();
    result.map(|_| staged)
}

fn staged_digest_writer<'a>(digest: &'a mut Sha256) -> impl Write + 'a {
    struct DigestWriter<'a>(&'a mut Sha256);
    impl Write for DigestWriter<'_> {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0.update(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    DigestWriter(digest)
}

/// Publish a verified staging file only if the destination remains absent.
pub fn publish_addition(staged: &mut StagedFile, destination: &Path) -> Result<(), GripError> {
    ensure_destination_binding(staged, destination)?;
    verify_staged_identity(staged)?;
    renameat_with(
        staged.parent.as_fd(),
        OsStr::from_bytes(&staged.name),
        staged.parent.as_fd(),
        OsStr::from_bytes(&staged.destination_name),
        RenameFlags::NOREPLACE,
    )
    .map_err(|error| {
        GripError::mutation_side_effect(
            "publication_failure",
            false,
            "not_attempted",
            false,
            format!("could not publish addition without replacement: {error}"),
        )
    })?;
    staged.published = true;
    sync_open_directory(&staged.parent).map_err(|error| {
        GripError::mutation_side_effect(
            "publication_failure",
            true,
            "not_attempted",
            false,
            error.to_string(),
        )
    })
}

/// Atomically publish a verified staging file over an expected mutation target.
pub fn publish_replacement(staged: &mut StagedFile, destination: &Path) -> Result<(), GripError> {
    ensure_destination_binding(staged, destination)?;
    verify_staged_identity(staged)?;
    renameat_with(
        staged.parent.as_fd(),
        OsStr::from_bytes(&staged.name),
        staged.parent.as_fd(),
        OsStr::from_bytes(&staged.destination_name),
        RenameFlags::empty(),
    )
    .map_err(|error| {
        GripError::mutation_side_effect(
            "publication_failure",
            false,
            "not_attempted",
            false,
            format!("could not publish target replacement: {error}"),
        )
    })?;
    staged.published = true;
    sync_open_directory(&staged.parent).map_err(|error| {
        GripError::mutation_side_effect(
            "publication_failure",
            true,
            "not_attempted",
            false,
            error.to_string(),
        )
    })
}

/// Verify the final supported mutation target state.
pub fn verify_target(target: &Path, expected: &SupportedState) -> Result<(), GripError> {
    let (parent, name) = open_parent(target)?;
    let (actual, _) =
        crate::observation::fingerprint::inspect_open_child(&parent, &name, expected.node_kind)?;
    if &actual != expected {
        return Err(GripError::mutation_side_effect(
            "verification_failure",
            true,
            "failed",
            true,
            "published mutation target failed supported-state verification",
        ));
    }
    Ok(())
}

/// Compatibility wrapper for the Feature 005 push API.
pub fn verify_destination(destination: &Path, expected: &SupportedState) -> Result<(), GripError> {
    verify_target(destination, expected)
}

/// Remove one verified supported entry relative to its opened parent without following links.
pub fn remove_verified(target: &Path, expected: &SupportedState) -> Result<bool, GripError> {
    let (parent, name) = revalidate_removal(target, expected)?;
    unlinkat(
        parent.as_fd(),
        OsStr::from_bytes(&name),
        if expected.node_kind == crate::discovery::model::NodeKind::Directory {
            AtFlags::REMOVEDIR
        } else {
            AtFlags::empty()
        },
    )
    .map_err(|error| GripError::from_io("could not remove verified target", error.into()))?;
    let durable = fsync(parent.as_fd()).is_ok();
    match parent.metadata(&name) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(durable),
        Ok(_) => Err(GripError::Internal(
            "deletion target remains visible after removal".into(),
        )),
        Err(error) => Err(GripError::from_io(
            "could not verify deletion target absence",
            error,
        )),
    }
}

/// Revalidate a removal target and a fresh empty-directory view without changing it.
pub fn verify_removal_ready(target: &Path, expected: &SupportedState) -> Result<(), GripError> {
    revalidate_removal(target, expected).map(|_| ())
}

fn revalidate_removal(
    target: &Path,
    expected: &SupportedState,
) -> Result<(crate::discovery::filesystem::Directory, Vec<u8>), GripError> {
    let (parent, name) = open_parent(target)?;
    let (actual, _) =
        crate::observation::fingerprint::inspect_open_child(&parent, &name, expected.node_kind)?;
    if &actual != expected {
        return Err(GripError::discovery_operational(
            "delete",
            "stale_deletion_evidence",
            vec![target.display().to_string()],
            "deletion target changed before removal",
        ));
    }
    if expected.node_kind == crate::discovery::model::NodeKind::Directory {
        let child = parent.open_child_directory(&name).map_err(|error| {
            GripError::from_io("could not open deletion directory safely", error)
        })?;
        if !child
            .child_names()
            .map_err(|error| GripError::from_io("could not enumerate deletion directory", error))?
            .is_empty()
        {
            return Err(GripError::InvalidConfiguration(
                "managed directory contains an unplanned descendant".into(),
            ));
        }
    }
    Ok((parent, name))
}

/// Remove one current-user-owned private recovery file after exact byte-count verification.
pub(crate) fn remove_private_file(path: &Path, expected_len: u64) -> Result<bool, GripError> {
    let (parent, name) = open_parent(path)?;
    let metadata = parent
        .metadata(&name)
        .map_err(|error| GripError::from_io("could not inspect private recovery bytes", error))?;
    let descriptor = parent
        .open_child_file(&name)
        .map_err(|error| GripError::from_io("could not open private recovery bytes", error))?;
    let opened = File::from(descriptor);
    let opened_metadata = opened
        .metadata()
        .map_err(|error| GripError::from_io("could not inspect opened recovery bytes", error))?;
    if !opened_metadata.is_file()
        || opened_metadata.uid() != rustix::process::geteuid().as_raw()
        || opened_metadata.permissions().mode() & 0o7777 != 0o600
        || opened_metadata.dev() != metadata.stat.st_dev as u64
        || opened_metadata.ino() != metadata.stat.st_ino
        || opened_metadata.len() != expected_len
    {
        return Err(GripError::CorruptState(
            "private recovery bytes do not match cleanup evidence".into(),
        ));
    }
    unlinkat(parent.as_fd(), OsStr::from_bytes(&name), AtFlags::empty()).map_err(|error| {
        GripError::from_io("could not remove private recovery bytes", error.into())
    })?;
    fsync(parent.as_fd())
        .map_err(|error| GripError::from_io("could not sync recovery directory", error.into()))?;
    match parent.metadata(&name) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(true),
        Ok(_) => Err(GripError::CorruptState(
            "private recovery bytes remain visible after cleanup".into(),
        )),
        Err(error) => Err(GripError::from_io(
            "could not verify recovery-byte absence",
            error,
        )),
    }
}

fn verify_staged_identity(staged: &StagedFile) -> Result<(), GripError> {
    let descriptor = staged
        .file
        .metadata()
        .map_err(|error| GripError::from_io("could not inspect target staging", error))?;
    let path = staged
        .parent
        .metadata(&staged.name)
        .map_err(|error| GripError::from_io("could not inspect target staging path", error))?;
    if descriptor.dev() != staged.device
        || descriptor.ino() != staged.inode
        || path.stat.st_dev as u64 != staged.device
        || path.stat.st_ino != staged.inode
        || path.file_type() != rustix::fs::FileType::RegularFile
    {
        return Err(GripError::CorruptState(
            "target staging path no longer identifies the attempt-owned file".into(),
        ));
    }
    Ok(())
}

fn validate_regular(path: &Path, metadata: &fs::Metadata) -> Result<(), GripError> {
    let path_metadata = crate::discovery::filesystem::metadata_at_path(path)
        .map_err(|error| GripError::from_io("could not revalidate mutation origin", error))?;
    if !metadata.is_file()
        || metadata.file_type().is_symlink()
        || metadata.nlink() != 1
        || metadata.dev() != path_metadata.stat.st_dev as u64
        || metadata.ino() != path_metadata.stat.st_ino
        || path_metadata.classify(path_metadata.stat.st_dev as u64)
            != crate::discovery::model::NodeKind::File
    {
        return Err(GripError::Internal(
            "mutation origin is no longer an ordinary supported file".into(),
        ));
    }
    Ok(())
}

fn sync_open_directory(
    directory: &crate::discovery::filesystem::Directory,
) -> Result<(), GripError> {
    fsync(directory.as_fd()).map_err(|error| {
        GripError::from_io("could not sync mutation target directory", error.into())
    })
}

fn ensure_destination_binding(staged: &StagedFile, destination: &Path) -> Result<(), GripError> {
    let expected = destination
        .file_name()
        .map(OsStr::as_bytes)
        .ok_or_else(|| GripError::Internal("destination file has no final component".into()))?;
    if expected != staged.destination_name || destination.parent() != staged.path.parent() {
        return Err(GripError::Internal(
            "staging file is bound to a different destination".into(),
        ));
    }
    Ok(())
}

fn open_parent(
    path: &Path,
) -> Result<(crate::discovery::filesystem::Directory, Vec<u8>), GripError> {
    if !path.is_absolute() {
        return Err(GripError::Internal(
            "mutation filesystem path must be absolute".into(),
        ));
    }
    let mut components = path.components().peekable();
    if components.next() != Some(Component::RootDir) {
        return Err(GripError::Internal(
            "mutation filesystem path must start at root".into(),
        ));
    }
    let mut directory = crate::discovery::filesystem::Directory::open(Path::new("/"))
        .map_err(|error| GripError::from_io("could not open filesystem root", error))?;
    let mut final_name = None;
    while let Some(component) = components.next() {
        let Component::Normal(name) = component else {
            return Err(GripError::Internal(
                "mutation filesystem path contains unsafe components".into(),
            ));
        };
        if components.peek().is_none() {
            final_name = Some(name.as_bytes().to_vec());
        } else {
            directory = directory
                .open_child_directory(name.as_bytes())
                .map_err(|error| {
                    GripError::from_io("could not open mutation path ancestry safely", error)
                })?;
        }
    }
    let name = final_name.ok_or_else(|| {
        GripError::Internal("mutation filesystem path has no final component".into())
    })?;
    Ok((directory, name))
}

fn stale(message: &str) -> GripError {
    stale_for(MutationDirection::Push, message)
}

fn stale_for(direction: MutationDirection, message: &str) -> GripError {
    let reason = match direction {
        MutationDirection::Push => "stale_push_evidence",
        MutationDirection::Pull => "stale_pull_evidence",
    };
    GripError::discovery_operational(direction.operation(), reason, Vec::new(), message)
}
