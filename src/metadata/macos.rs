//! Narrow macOS filesystem adapter for descriptor-bound metadata operations.

use crate::metadata::model::{
    AccessControlEntry, AclEntryFlag, AclEntryKind, AclPermission, AclState, BsdFlag,
    MetadataState, ModificationTime, XattrFingerprint,
};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs::File;
use std::io;

#[cfg(target_os = "macos")]
use std::os::fd::AsRawFd;

#[cfg(target_os = "macos")]
const ACL_TYPE_EXTENDED: libc::c_int = 0x0000_0100;

#[cfg(target_os = "macos")]
#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    fn acl_get_fd_np(fd: libc::c_int, acl_type: libc::c_int) -> *mut libc::c_void;
    fn acl_free(object: *mut libc::c_void) -> libc::c_int;
    fn acl_size(acl: *mut libc::c_void) -> libc::ssize_t;
    fn acl_copy_ext(
        buffer: *mut libc::c_void,
        acl: *mut libc::c_void,
        size: libc::ssize_t,
    ) -> libc::ssize_t;
    fn acl_get_entry(
        acl: *mut libc::c_void,
        entry_id: libc::c_int,
        entry: *mut *mut libc::c_void,
    ) -> libc::c_int;
    fn acl_get_tag_type(entry: *mut libc::c_void, tag: *mut libc::c_int) -> libc::c_int;
    fn acl_get_qualifier(entry: *mut libc::c_void) -> *mut libc::c_void;
    fn acl_get_permset_mask_np(entry: *mut libc::c_void, mask: *mut u64) -> libc::c_int;
    fn acl_get_flagset_np(entry: *mut libc::c_void, flagset: *mut *mut libc::c_void)
    -> libc::c_int;
    fn acl_get_flag_np(flagset: *mut libc::c_void, flag: libc::c_int) -> libc::c_int;
    fn acl_init(count: libc::c_int) -> *mut libc::c_void;
    fn acl_set_fd_np(fd: libc::c_int, acl: *mut libc::c_void, acl_type: libc::c_int)
    -> libc::c_int;
    fn fchflags(fd: libc::c_int, flags: libc::c_uint) -> libc::c_int;
    fn CFStringCreateMutableCopy(
        allocator: *const libc::c_void,
        capacity: libc::c_long,
        value: *const libc::c_void,
    ) -> *mut libc::c_void;
    fn CFStringCreateWithBytes(
        allocator: *const libc::c_void,
        bytes: *const u8,
        length: libc::c_long,
        encoding: u32,
        external_representation: bool,
    ) -> *const libc::c_void;
    fn CFStringNormalize(value: *mut libc::c_void, form: libc::c_long);
    fn CFStringFold(value: *mut libc::c_void, flags: libc::c_ulong, locale: *const libc::c_void);
    fn CFStringGetLength(value: *const libc::c_void) -> libc::c_long;
    fn CFStringGetMaximumSizeForEncoding(length: libc::c_long, encoding: u32) -> libc::c_long;
    fn CFStringGetCString(
        value: *const libc::c_void,
        buffer: *mut libc::c_char,
        size: libc::c_long,
        encoding: u32,
    ) -> bool;
    fn CFRelease(value: *const libc::c_void);
}

#[cfg(target_os = "macos")]
struct OwnedAcl(*mut libc::c_void);

#[cfg(target_os = "macos")]
impl Drop for OwnedAcl {
    fn drop(&mut self) {
        // SAFETY: The pointer was returned by acl_get_fd_np and is released exactly once.
        unsafe {
            acl_free(self.0);
        }
    }
}

/// Read the raw owner-visible BSD flags from an already opened entry.
pub fn raw_bsd_flags(file: &File) -> io::Result<u32> {
    #[cfg(target_os = "macos")]
    {
        rustix::fs::fstat(file)
            .map(|metadata| metadata.st_flags)
            .map_err(io::Error::from)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = file;
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "BSD flags are supported only on macOS",
        ))
    }
}

/// Report every present BSD flag outside the supported equality allowlist.
pub fn unsupported_bsd_flags(
    file: &File,
    node_kind: crate::discovery::model::NodeKind,
) -> io::Result<Vec<String>> {
    #[cfg(target_os = "macos")]
    {
        let raw = raw_bsd_flags(file)?;
        Ok(unsupported_bsd_flag_names(raw, node_kind))
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (file, node_kind);
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "BSD flags are supported only on macOS",
        ))
    }
}

/// Convert raw Darwin flags outside the explicit allowlist into stable safe names.
#[doc(hidden)]
pub fn unsupported_bsd_flag_names(
    raw: u32,
    node_kind: crate::discovery::model::NodeKind,
) -> Vec<String> {
    #[cfg(target_os = "macos")]
    let mut supported = libc::UF_NODUMP | libc::UF_IMMUTABLE | libc::UF_APPEND | libc::UF_HIDDEN;
    #[cfg(not(target_os = "macos"))]
    let mut supported = 0x0000_0001 | 0x0000_0002 | 0x0000_0004 | 0x0000_8000;
    if node_kind == crate::discovery::model::NodeKind::Directory {
        supported |= 0x0000_0008;
    }
    let unsupported = raw & !supported;
    let mut names = Vec::new();
    let mut known = 0_u32;
    for (mask, name) in [
        (0x0000_0008, "UF_OPAQUE"),
        (0x0000_0020, "UF_COMPRESSED"),
        (0x0000_0040, "UF_TRACKED"),
        (0x0000_0080, "UF_DATAVAULT"),
        (0x0001_0000, "SF_ARCHIVED"),
        (0x0002_0000, "SF_IMMUTABLE"),
        (0x0004_0000, "SF_APPEND"),
        (0x0008_0000, "SF_RESTRICTED"),
        (0x0010_0000, "SF_NOUNLINK"),
        (0x0080_0000, "SF_FIRMLINK"),
        (0x4000_0000, "SF_DATALESS"),
    ] {
        if unsupported & mask != 0 {
            names.push(name.into());
            known |= mask;
        }
    }
    let unknown = unsupported & !known;
    if unknown != 0 {
        names.push(format!("0x{unknown:08x}"));
    }
    names
}

/// Capture an extended ACL in Darwin's stable external binary representation.
pub fn raw_acl(file: &File) -> io::Result<Option<Vec<u8>>> {
    #[cfg(target_os = "macos")]
    {
        // SAFETY: The descriptor remains open for the call and the returned pointer is owned.
        let pointer = unsafe { acl_get_fd_np(file.as_raw_fd(), ACL_TYPE_EXTENDED) };
        if pointer.is_null() {
            let error = io::Error::last_os_error();
            return if error.raw_os_error() == Some(libc::ENOENT) {
                Ok(None)
            } else {
                Err(error)
            };
        }
        let acl = OwnedAcl(pointer);
        // SAFETY: acl holds a valid ACL pointer for the duration of the call.
        let size = unsafe { acl_size(acl.0) };
        if size < 0 {
            return Err(io::Error::last_os_error());
        }
        if size == 0 {
            return Ok(None);
        }
        let mut bytes = vec![0; usize::try_from(size).expect("ACL size fits usize")];
        // SAFETY: bytes has exactly size writable bytes and acl remains valid.
        let copied = unsafe { acl_copy_ext(bytes.as_mut_ptr().cast(), acl.0, size) };
        if copied < 0 {
            return Err(io::Error::last_os_error());
        }
        bytes.truncate(usize::try_from(copied).expect("ACL byte count fits usize"));
        Ok(Some(bytes))
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = file;
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "extended ACLs are supported only on macOS",
        ))
    }
}

/// Decode the ordered semantic extended ACL from an open descriptor.
pub fn read_acl(file: &File, node_kind: crate::discovery::model::NodeKind) -> io::Result<AclState> {
    #[cfg(target_os = "macos")]
    {
        // SAFETY: The descriptor remains open and the returned ACL is owned by this function.
        let pointer = unsafe { acl_get_fd_np(file.as_raw_fd(), ACL_TYPE_EXTENDED) };
        if pointer.is_null() {
            let error = io::Error::last_os_error();
            return if error.raw_os_error() == Some(libc::ENOENT) {
                Ok(AclState::Absent)
            } else {
                Err(error)
            };
        }
        let acl = OwnedAcl(pointer);
        let mut entries = Vec::new();
        let mut entry_id = 0;
        loop {
            let mut entry = std::ptr::null_mut();
            // SAFETY: acl is valid and entry points to writable storage.
            let result = unsafe { acl_get_entry(acl.0, entry_id, &mut entry) };
            if result != 0 {
                let error = io::Error::last_os_error();
                if error.raw_os_error() == Some(libc::EINVAL) {
                    break;
                }
                return Err(error);
            }
            entry_id = -1;
            let mut tag = 0;
            // SAFETY: entry was returned by acl_get_entry.
            if unsafe { acl_get_tag_type(entry, &mut tag) } != 0 {
                return Err(io::Error::last_os_error());
            }
            let kind = match tag {
                1 => AclEntryKind::Allow,
                2 => AclEntryKind::Deny,
                _ => return Err(io::Error::other("ACL contains an unknown entry tag")),
            };
            // SAFETY: entry is valid and the returned qualifier is independently allocated.
            let qualifier = unsafe { acl_get_qualifier(entry) };
            if qualifier.is_null() {
                return Err(io::Error::last_os_error());
            }
            let mut principal_uuid = [0_u8; 16];
            // SAFETY: Darwin extended ACL qualifiers are uuid_t values of exactly 16 bytes.
            unsafe {
                std::ptr::copy_nonoverlapping(
                    qualifier.cast::<u8>(),
                    principal_uuid.as_mut_ptr(),
                    16,
                );
            }
            // SAFETY: qualifier is returned by acl_get_qualifier and released exactly once.
            unsafe {
                acl_free(qualifier);
            }
            let mut permission_mask = 0_u64;
            // SAFETY: entry is valid and mask is writable.
            if unsafe { acl_get_permset_mask_np(entry, &mut permission_mask) } != 0 {
                return Err(io::Error::last_os_error());
            }
            let directory = node_kind == crate::discovery::model::NodeKind::Directory;
            let permissions = permission_values(permission_mask, directory);
            let mut flagset = std::ptr::null_mut();
            // SAFETY: entry is valid and flagset points to writable storage.
            if unsafe { acl_get_flagset_np(entry, &mut flagset) } != 0 {
                return Err(io::Error::last_os_error());
            }
            let mut flags = BTreeSet::new();
            for (mask, flag) in [
                (1 << 5, AclEntryFlag::FileInherit),
                (1 << 6, AclEntryFlag::DirectoryInherit),
                (1 << 7, AclEntryFlag::LimitInherit),
                (1 << 8, AclEntryFlag::OnlyInherit),
                (1 << 4, AclEntryFlag::Inherited),
            ] {
                // SAFETY: flagset belongs to the valid entry.
                let present = unsafe { acl_get_flag_np(flagset, mask) };
                if present < 0 {
                    return Err(io::Error::last_os_error());
                }
                if present == 1 {
                    flags.insert(flag);
                }
            }
            entries.push(AccessControlEntry {
                principal_uuid,
                kind,
                permissions,
                flags,
            });
        }
        Ok(if entries.is_empty() {
            AclState::Absent
        } else {
            AclState::Present { entries }
        })
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (file, node_kind);
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "extended ACLs are qualified only on macOS",
        ))
    }
}

fn permission_values(mask: u64, directory: bool) -> BTreeSet<AclPermission> {
    let mut values = BTreeSet::new();
    for (bit, file_value, directory_value) in [
        (
            1 << 1,
            AclPermission::ReadData,
            AclPermission::ListDirectory,
        ),
        (1 << 2, AclPermission::WriteData, AclPermission::AddFile),
        (1 << 3, AclPermission::Execute, AclPermission::Search),
        (
            1 << 5,
            AclPermission::AppendData,
            AclPermission::AddSubdirectory,
        ),
    ] {
        if mask & bit != 0 {
            values.insert(if directory {
                directory_value
            } else {
                file_value
            });
        }
    }
    for (bit, value) in [
        (1 << 4, AclPermission::Delete),
        (1 << 6, AclPermission::DeleteChild),
        (1 << 7, AclPermission::ReadAttributes),
        (1 << 8, AclPermission::WriteAttributes),
        (1 << 9, AclPermission::ReadExtendedAttributes),
        (1 << 10, AclPermission::WriteExtendedAttributes),
        (1 << 11, AclPermission::ReadSecurity),
        (1 << 12, AclPermission::WriteSecurity),
        (1 << 13, AclPermission::ChangeOwner),
        (1 << 20, AclPermission::Synchronize),
    ] {
        if mask & bit != 0 {
            values.insert(value);
        }
    }
    values
}

/// Exact live xattr evidence separated by the documented policy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservedXattrs {
    pub synchronized: Vec<XattrFingerprint>,
    pub synchronized_values: Vec<(Vec<u8>, Vec<u8>)>,
    pub excluded: Vec<Vec<u8>>,
    pub unknown: Vec<Vec<u8>>,
}

/// Enumerate and read xattrs through an open descriptor with bounded race retries.
pub fn observe_xattrs(file: &File) -> io::Result<ObservedXattrs> {
    observe_xattrs_with_hook(file, |_, _| {})
}

/// Observe xattrs while allowing tests to inject a change after each value-size query.
#[doc(hidden)]
pub fn observe_xattrs_with_hook<F>(
    file: &File,
    mut after_size_query: F,
) -> io::Result<ObservedXattrs>
where
    F: FnMut(usize, &[u8]),
{
    #[cfg(target_os = "macos")]
    {
        let mut last_names = Vec::new();
        for attempt in 0..3 {
            let names = list_xattr_names(file)?;
            let mut synchronized_values = Vec::new();
            let mut excluded = Vec::new();
            let mut unknown = Vec::new();
            let mut unstable = false;
            for name in &names {
                match crate::metadata::xattr_policy(name) {
                    crate::metadata::XattrPolicy::Synchronized => {
                        match read_xattr_with_hook(file, name, || after_size_query(attempt, name)) {
                            Ok(value) => synchronized_values.push((name.clone(), value)),
                            Err(error)
                                if error.raw_os_error() == Some(libc::ERANGE)
                                    || error.raw_os_error() == Some(libc::ENOATTR) =>
                            {
                                unstable = true;
                                break;
                            }
                            Err(error) => return Err(error),
                        }
                    }
                    crate::metadata::XattrPolicy::Excluded => excluded.push(name.clone()),
                    crate::metadata::XattrPolicy::Unknown => unknown.push(name.clone()),
                }
            }
            if unstable || list_xattr_names(file)? != names {
                last_names = names;
                continue;
            }
            let synchronized = synchronized_values
                .iter()
                .map(|(name, value)| XattrFingerprint {
                    name: name.clone(),
                    length: value.len() as u64,
                    algorithm: "sha256".into(),
                    digest: format!("{:x}", Sha256::digest(value)),
                })
                .collect();
            return Ok(ObservedXattrs {
                synchronized,
                synchronized_values,
                excluded,
                unknown,
            });
        }
        let _ = last_names;
        Err(io::Error::other(
            "extended attributes changed during bounded observation",
        ))
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (file, &mut after_size_query);
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "xattrs are qualified only on macOS",
        ))
    }
}

#[cfg(target_os = "macos")]
fn list_xattr_names(file: &File) -> io::Result<Vec<Vec<u8>>> {
    let fd = file.as_raw_fd();
    // SAFETY: A null buffer with size zero is the documented size query.
    let size = unsafe { libc::flistxattr(fd, std::ptr::null_mut(), 0, 0) };
    if size < 0 {
        return Err(io::Error::last_os_error());
    }
    if size == 0 {
        return Ok(Vec::new());
    }
    let mut bytes =
        vec![0_u8; usize::try_from(size).map_err(|_| io::Error::other("xattr list is too large"))?];
    // SAFETY: bytes is writable for its full length and fd remains open.
    let read = unsafe { libc::flistxattr(fd, bytes.as_mut_ptr().cast(), bytes.len(), 0) };
    if read < 0 {
        return Err(io::Error::last_os_error());
    }
    bytes.truncate(usize::try_from(read).map_err(|_| io::Error::other("xattr list is too large"))?);
    let mut names = bytes
        .split(|byte| *byte == 0)
        .filter(|name| !name.is_empty())
        .map(<[u8]>::to_vec)
        .collect::<Vec<_>>();
    names.sort();
    names.dedup();
    Ok(names)
}

#[cfg(target_os = "macos")]
fn read_xattr_with_hook<F>(file: &File, name: &[u8], after_size_query: F) -> io::Result<Vec<u8>>
where
    F: FnOnce(),
{
    let name = std::ffi::CString::new(name)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "xattr name contains NUL"))?;
    // SAFETY: name is NUL-terminated and a null buffer is a documented size query.
    let size = unsafe {
        libc::fgetxattr(
            file.as_raw_fd(),
            name.as_ptr(),
            std::ptr::null_mut(),
            0,
            0,
            0,
        )
    };
    if size < 0 {
        return Err(io::Error::last_os_error());
    }
    after_size_query();
    let mut value = vec![
        0_u8;
        usize::try_from(size)
            .map_err(|_| io::Error::other("xattr value is too large"))?
    ];
    if value.is_empty() {
        return Ok(value);
    }
    // SAFETY: value has the queried writable length and both fd and name remain valid.
    let read = unsafe {
        libc::fgetxattr(
            file.as_raw_fd(),
            name.as_ptr(),
            value.as_mut_ptr().cast(),
            value.len(),
            0,
            0,
        )
    };
    if read < 0 {
        return Err(io::Error::last_os_error());
    }
    value
        .truncate(usize::try_from(read).map_err(|_| io::Error::other("xattr value is too large"))?);
    Ok(value)
}

/// One descriptor-bound metadata read shared by equality and diagnostic projections.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompleteMetadataObservation {
    pub metadata: MetadataState,
    pub xattrs: ObservedXattrs,
    pub unsupported_bsd_flags: Vec<String>,
}

/// Observe supported state and diagnostics without repeating xattr or BSD-flag reads.
pub fn observe_complete_metadata(
    file: &File,
    node_kind: crate::discovery::model::NodeKind,
) -> io::Result<CompleteMetadataObservation> {
    let stat = rustix::fs::fstat(file).map_err(io::Error::from)?;
    let xattrs = observe_xattrs(file)?;
    let flags = raw_bsd_flags(file)?;
    let mut bsd_flags = BTreeSet::new();
    #[cfg(target_os = "macos")]
    {
        for (mask, flag) in [
            (libc::UF_NODUMP, BsdFlag::Nodump),
            (libc::UF_IMMUTABLE, BsdFlag::Immutable),
            (libc::UF_APPEND, BsdFlag::Append),
            (libc::UF_HIDDEN, BsdFlag::Hidden),
            (libc::UF_OPAQUE, BsdFlag::Opaque),
        ] {
            if flags & mask != 0 {
                bsd_flags.insert(flag);
            }
        }
    }
    let acl = read_acl(file, node_kind)?;
    let state = MetadataState {
        permission_mode: format!("{:04o}", (stat.st_mode as u32) & 0o7777),
        uid: stat.st_uid,
        gid: stat.st_gid,
        modified_time: ModificationTime {
            seconds: stat.st_mtime,
            nanoseconds: u32::try_from(stat.st_mtime_nsec)
                .map_err(|_| io::Error::other("invalid modification-time nanoseconds"))?,
        },
        extended_attributes: xattrs.synchronized.clone(),
        acl,
        bsd_flags,
    };
    state.validate(node_kind).map_err(io::Error::other)?;
    Ok(CompleteMetadataObservation {
        metadata: state,
        xattrs,
        unsupported_bsd_flags: unsupported_bsd_flag_names(flags, node_kind),
    })
}

/// Observe the currently supported metadata dimensions from an open descriptor.
pub fn observe_metadata(
    file: &File,
    node_kind: crate::discovery::model::NodeKind,
) -> io::Result<MetadataState> {
    observe_complete_metadata(file, node_kind).map(|observed| observed.metadata)
}

/// Ordered boundaries exposed to deterministic mutation failure tests.
#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetadataApplyPhase {
    AfterProtectedFlagsCleared,
    AfterOwnership,
    AfterAcl,
    AfterExtendedAttributes,
    AfterPermissionMode,
    AfterModificationTime,
    AfterBsdFlags,
    AfterDurability,
    BeforeVerification,
    AfterVerification,
}

/// Apply the complete supported metadata from one open origin to one open target.
pub fn apply_metadata_from(
    origin: &File,
    target: &File,
    node_kind: crate::discovery::model::NodeKind,
    expected: &MetadataState,
) -> io::Result<()> {
    apply_metadata_from_with_hook(origin, target, node_kind, expected, |_| Ok(()))
}

/// Apply complete metadata with deterministic hooks after each ordered substep.
#[doc(hidden)]
pub fn apply_metadata_from_with_hook<F>(
    origin: &File,
    target: &File,
    node_kind: crate::discovery::model::NodeKind,
    expected: &MetadataState,
    mut hook: F,
) -> io::Result<()>
where
    F: FnMut(MetadataApplyPhase) -> io::Result<()>,
{
    #[cfg(target_os = "macos")]
    {
        expected.validate(node_kind).map_err(io::Error::other)?;
        let current_flags = raw_bsd_flags(target)?;
        let clear_mask = libc::UF_IMMUTABLE | libc::UF_APPEND;
        // SAFETY: target remains open and only supported owner flags are cleared.
        if unsafe { fchflags(target.as_raw_fd(), current_flags & !clear_mask) } != 0 {
            return Err(io::Error::last_os_error());
        }
        hook(MetadataApplyPhase::AfterProtectedFlagsCleared)?;
        let target_stat = rustix::fs::fstat(target).map_err(io::Error::from)?;
        if target_stat.st_uid != expected.uid || target_stat.st_gid != expected.gid {
            // SAFETY: target remains open; numeric IDs are the explicit contract.
            if unsafe { libc::fchown(target.as_raw_fd(), expected.uid, expected.gid) } != 0 {
                return Err(io::Error::last_os_error());
            }
        }
        hook(MetadataApplyPhase::AfterOwnership)?;
        copy_acl(origin, target)?;
        hook(MetadataApplyPhase::AfterAcl)?;
        let origin_xattrs = observe_xattrs(origin)?;
        if !origin_xattrs.unknown.is_empty() {
            return Err(io::Error::other("origin has unknown extended attributes"));
        }
        let target_xattrs = observe_xattrs(target)?;
        if !target_xattrs.unknown.is_empty() {
            return Err(io::Error::other("target has unknown extended attributes"));
        }
        for (name, _) in target_xattrs.synchronized_values {
            if !origin_xattrs
                .synchronized_values
                .iter()
                .any(|(candidate, _)| candidate == &name)
            {
                let name = std::ffi::CString::new(name)
                    .map_err(|_| io::Error::other("xattr name contains NUL"))?;
                // SAFETY: descriptor and C string remain valid for the call.
                if unsafe { libc::fremovexattr(target.as_raw_fd(), name.as_ptr(), 0) } != 0 {
                    return Err(io::Error::last_os_error());
                }
            }
        }
        for (name, value) in origin_xattrs.synchronized_values {
            let name = std::ffi::CString::new(name)
                .map_err(|_| io::Error::other("xattr name contains NUL"))?;
            // SAFETY: descriptor, name, and value remain valid for the call.
            if unsafe {
                libc::fsetxattr(
                    target.as_raw_fd(),
                    name.as_ptr(),
                    value.as_ptr().cast(),
                    value.len(),
                    0,
                    0,
                )
            } != 0
            {
                return Err(io::Error::last_os_error());
            }
        }
        hook(MetadataApplyPhase::AfterExtendedAttributes)?;
        let mode = u32::from_str_radix(&expected.permission_mode, 8)
            .map_err(|_| io::Error::other("invalid permission mode"))?;
        rustix::fs::fchmod(
            target,
            rustix::fs::Mode::from_raw_mode(
                u16::try_from(mode).map_err(|_| io::Error::other("invalid permission mode"))?,
            ),
        )
        .map_err(io::Error::from)?;
        hook(MetadataApplyPhase::AfterPermissionMode)?;
        rustix::fs::futimens(
            target,
            &rustix::fs::Timestamps {
                last_access: rustix::fs::Timespec {
                    tv_sec: 0,
                    tv_nsec: rustix::fs::UTIME_OMIT,
                },
                last_modification: rustix::fs::Timespec {
                    tv_sec: expected.modified_time.seconds,
                    tv_nsec: i64::from(expected.modified_time.nanoseconds),
                },
            },
        )
        .map_err(io::Error::from)?;
        hook(MetadataApplyPhase::AfterModificationTime)?;
        let final_flags = flags_to_raw(&expected.bsd_flags);
        // SAFETY: target remains open and final_flags contains only supported flags.
        if unsafe { fchflags(target.as_raw_fd(), final_flags) } != 0 {
            return Err(io::Error::last_os_error());
        }
        hook(MetadataApplyPhase::AfterBsdFlags)?;
        target.sync_all()?;
        hook(MetadataApplyPhase::AfterDurability)?;
        hook(MetadataApplyPhase::BeforeVerification)?;
        let observed = observe_metadata(target, node_kind)?;
        if &observed != expected {
            return Err(io::Error::other("complete metadata verification failed"));
        }
        hook(MetadataApplyPhase::AfterVerification)?;
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (origin, target, node_kind, expected, hook);
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "metadata application is qualified only on macOS",
        ))
    }
}

/// Open both endpoints without following their final components and apply complete metadata.
pub fn apply_metadata_paths(
    origin: &std::path::Path,
    target: &std::path::Path,
    node_kind: crate::discovery::model::NodeKind,
    expected: &MetadataState,
) -> io::Result<()> {
    apply_metadata_paths_with_hook(origin, target, node_kind, expected, |_| Ok(()))
}

/// Open both endpoints and apply complete metadata with deterministic substep hooks.
#[doc(hidden)]
pub fn apply_metadata_paths_with_hook<F>(
    origin: &std::path::Path,
    target: &std::path::Path,
    node_kind: crate::discovery::model::NodeKind,
    expected: &MetadataState,
    hook: F,
) -> io::Result<()>
where
    F: FnMut(MetadataApplyPhase) -> io::Result<()>,
{
    let mut flags = rustix::fs::OFlags::RDONLY
        | rustix::fs::OFlags::CLOEXEC
        | rustix::fs::OFlags::NOFOLLOW
        | rustix::fs::OFlags::NONBLOCK;
    if node_kind == crate::discovery::model::NodeKind::Directory {
        flags |= rustix::fs::OFlags::DIRECTORY;
    }
    let origin = File::from(
        rustix::fs::open(origin, flags, rustix::fs::Mode::empty()).map_err(io::Error::from)?,
    );
    let target = File::from(
        rustix::fs::open(target, flags, rustix::fs::Mode::empty()).map_err(io::Error::from)?,
    );
    apply_metadata_from_with_hook(&origin, &target, node_kind, expected, hook)
}

/// Copy only allowlisted xattrs between private, already-open objects and verify fingerprints.
pub fn copy_synchronized_xattrs(origin: &File, target: &File) -> io::Result<Vec<XattrFingerprint>> {
    #[cfg(target_os = "macos")]
    {
        let observed = observe_xattrs(origin)?;
        for (name, value) in &observed.synchronized_values {
            let name = std::ffi::CString::new(name.as_slice())
                .map_err(|_| io::Error::other("xattr name contains NUL"))?;
            // SAFETY: descriptors, name, and value are valid for the call.
            if unsafe {
                libc::fsetxattr(
                    target.as_raw_fd(),
                    name.as_ptr(),
                    value.as_ptr().cast(),
                    value.len(),
                    0,
                    0,
                )
            } != 0
            {
                return Err(io::Error::last_os_error());
            }
        }
        let copied = observe_xattrs(target)?;
        if copied.synchronized != observed.synchronized {
            return Err(io::Error::other("recovery xattr verification failed"));
        }
        Ok(copied.synchronized)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (origin, target);
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "xattrs are qualified only on macOS",
        ))
    }
}

#[cfg(target_os = "macos")]
fn copy_acl(origin: &File, target: &File) -> io::Result<()> {
    // SAFETY: origin remains open; returned ACL is owned when non-null.
    let pointer = unsafe { acl_get_fd_np(origin.as_raw_fd(), ACL_TYPE_EXTENDED) };
    let acl = if pointer.is_null() {
        let error = io::Error::last_os_error();
        if error.raw_os_error() != Some(libc::ENOENT) {
            return Err(error);
        }
        // SAFETY: acl_init creates an owned empty ACL.
        let empty = unsafe { acl_init(0) };
        if empty.is_null() {
            return Err(io::Error::last_os_error());
        }
        OwnedAcl(empty)
    } else {
        OwnedAcl(pointer)
    };
    // SAFETY: target and ACL remain valid for the call.
    if unsafe { acl_set_fd_np(target.as_raw_fd(), acl.0, ACL_TYPE_EXTENDED) } != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

fn flags_to_raw(flags: &BTreeSet<BsdFlag>) -> u32 {
    #[cfg(target_os = "macos")]
    {
        flags.iter().fold(0, |raw, flag| {
            raw | match flag {
                BsdFlag::Nodump => libc::UF_NODUMP,
                BsdFlag::Immutable => libc::UF_IMMUTABLE,
                BsdFlag::Append => libc::UF_APPEND,
                BsdFlag::Hidden => libc::UF_HIDDEN,
                BsdFlag::Opaque => libc::UF_OPAQUE,
            }
        })
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = flags;
        0
    }
}

/// Build an APFS-oriented canonical Unicode comparison key without changing path identity.
pub fn name_comparison_key(
    name: &[u8],
    case_sensitive: bool,
) -> crate::metadata::model::Evidence<Vec<u8>> {
    #[cfg(target_os = "macos")]
    {
        const UTF8: u32 = 0x0800_0100;
        if std::str::from_utf8(name).is_err() {
            return crate::metadata::model::Evidence::Unavailable {
                reason: "path component is not UTF-8; endpoint comparison is inconclusive".into(),
            };
        }
        // SAFETY: CoreFoundation copies the supplied bytes and all returned objects are released.
        unsafe {
            let immutable = CFStringCreateWithBytes(
                std::ptr::null(),
                name.as_ptr(),
                name.len() as libc::c_long,
                UTF8,
                false,
            );
            if immutable.is_null() {
                return crate::metadata::model::Evidence::Unavailable {
                    reason: "CoreFoundation could not represent the path component".into(),
                };
            }
            let value = CFStringCreateMutableCopy(std::ptr::null(), 0, immutable);
            CFRelease(immutable);
            if value.is_null() {
                return crate::metadata::model::Evidence::Unavailable {
                    reason: "CoreFoundation could not allocate a comparison key".into(),
                };
            }
            CFStringNormalize(value, 0);
            if !case_sensitive {
                CFStringFold(value, 1, std::ptr::null());
            }
            let length = CFStringGetLength(value);
            let maximum = CFStringGetMaximumSizeForEncoding(length, UTF8);
            if maximum < 0 {
                CFRelease(value);
                return crate::metadata::model::Evidence::Unavailable {
                    reason: "CoreFoundation could not size a comparison key".into(),
                };
            }
            let mut bytes = vec![0_u8; maximum as usize + 1];
            let converted = CFStringGetCString(
                value,
                bytes.as_mut_ptr().cast(),
                bytes.len() as libc::c_long,
                UTF8,
            );
            CFRelease(value);
            if !converted {
                return crate::metadata::model::Evidence::Unavailable {
                    reason: "CoreFoundation could not encode a comparison key".into(),
                };
            }
            let length = bytes
                .iter()
                .position(|byte| *byte == 0)
                .unwrap_or(bytes.len());
            bytes.truncate(length);
            crate::metadata::model::Evidence::Observed { value: bytes }
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (name, case_sensitive);
        crate::metadata::model::Evidence::Unsupported {
            reason: "APFS name comparison is qualified only on macOS".into(),
        }
    }
}

#[cfg(target_os = "macos")]
#[repr(C)]
struct VolumeCapabilitiesBuffer {
    length: u32,
    capabilities: [u32; 4],
    valid: [u32; 4],
}

/// Discover operation-scoped filesystem and metadata capabilities for one open endpoint.
pub fn endpoint_capability_profile(
    file: &File,
    role: crate::metadata::model::EndpointRole,
    root_display: String,
) -> io::Result<crate::metadata::model::EndpointCapabilityProfile> {
    #[cfg(target_os = "macos")]
    {
        let mut filesystem: libc::statfs = unsafe { std::mem::zeroed() };
        // SAFETY: filesystem is writable and the descriptor remains open.
        if unsafe { libc::fstatfs(file.as_raw_fd(), &mut filesystem) } != 0 {
            return Err(io::Error::last_os_error());
        }
        let filesystem_type = unsafe { std::ffi::CStr::from_ptr(filesystem.f_fstypename.as_ptr()) }
            .to_string_lossy()
            .into_owned();
        let stat = rustix::fs::fstat(file).map_err(io::Error::from)?;
        let mut attributes = libc::attrlist {
            bitmapcount: 5,
            reserved: 0,
            commonattr: 0,
            volattr: 0x0002_0000,
            dirattr: 0,
            fileattr: 0,
            forkattr: 0,
        };
        let mut volume = VolumeCapabilitiesBuffer {
            length: std::mem::size_of::<VolumeCapabilitiesBuffer>() as u32,
            capabilities: [0; 4],
            valid: [0; 4],
        };
        // SAFETY: attributes and volume have the documented Darwin layouts.
        let capability_result = unsafe {
            libc::fgetattrlist(
                file.as_raw_fd(),
                (&mut attributes as *mut libc::attrlist).cast(),
                (&mut volume as *mut VolumeCapabilitiesBuffer).cast(),
                std::mem::size_of::<VolumeCapabilitiesBuffer>(),
                0,
            )
        };
        let (case_sensitive, case_preserving) =
            if capability_result == 0 && volume.valid[0] & 0x0000_0300 == 0x0000_0300 {
                (
                    crate::metadata::model::Evidence::Observed {
                        value: volume.capabilities[0] & 0x0000_0100 != 0,
                    },
                    crate::metadata::model::Evidence::Observed {
                        value: volume.capabilities[0] & 0x0000_0200 != 0,
                    },
                )
            } else {
                (
                    crate::metadata::model::Evidence::Unavailable {
                        reason: "volume case-sensitivity capability was not returned".into(),
                    },
                    crate::metadata::model::Evidence::Unavailable {
                        reason: "volume case-preservation capability was not returned".into(),
                    },
                )
            };
        let eligible = filesystem_type == "apfs";
        let capabilities = [
            crate::metadata::model::MetadataDimension::Node,
            crate::metadata::model::MetadataDimension::Content,
            crate::metadata::model::MetadataDimension::PermissionMode,
            crate::metadata::model::MetadataDimension::Owner,
            crate::metadata::model::MetadataDimension::Group,
            crate::metadata::model::MetadataDimension::ModificationTime,
            crate::metadata::model::MetadataDimension::ExtendedAttribute,
            crate::metadata::model::MetadataDimension::AccessControlList,
            crate::metadata::model::MetadataDimension::BsdFlags,
        ]
        .into_iter()
        .map(|dimension| crate::metadata::model::CapabilityEvidence {
            dimension,
            inspect: if eligible {
                crate::metadata::model::Evidence::Observed { value: true }
            } else {
                crate::metadata::model::Evidence::Unsupported {
                    reason: "filesystem is not APFS".into(),
                }
            },
            apply: if eligible {
                crate::metadata::model::Evidence::Observed { value: true }
            } else {
                crate::metadata::model::Evidence::Unsupported {
                    reason: "filesystem is not APFS".into(),
                }
            },
            verify: if eligible {
                crate::metadata::model::Evidence::Observed { value: true }
            } else {
                crate::metadata::model::Evidence::Unsupported {
                    reason: "filesystem is not APFS".into(),
                }
            },
        })
        .collect();
        let profile = crate::metadata::model::EndpointCapabilityProfile {
            endpoint: role,
            root_display,
            root_raw_hex: None,
            filesystem_type: crate::metadata::model::Evidence::Observed {
                value: filesystem_type,
            },
            filesystem_identity: crate::metadata::model::Evidence::Observed {
                value: stat.st_dev as u64,
            },
            mount_flags: crate::metadata::model::Evidence::Observed {
                value: filesystem.f_flags as u64,
            },
            volume_capability_masks: if capability_result == 0 {
                crate::metadata::model::Evidence::Observed {
                    value: volume
                        .capabilities
                        .iter()
                        .zip(volume.valid.iter())
                        .map(|(capability, valid)| {
                            (u64::from(*valid) << 32) | u64::from(*capability)
                        })
                        .collect(),
                }
            } else {
                crate::metadata::model::Evidence::Unavailable {
                    reason: "volume capability masks were not returned".into(),
                }
            },
            case_sensitive,
            case_preserving,
            unicode_qualification: crate::metadata::model::Evidence::Observed {
                value: "corefoundation-nfd".into(),
            },
            mtime_precision_nanoseconds: crate::metadata::model::Evidence::Observed { value: 1 },
            capabilities,
            qualification_reference: None,
        };
        profile.validate().map_err(io::Error::other)?;
        Ok(profile)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (file, role, root_display);
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "endpoint qualification is available only on macOS",
        ))
    }
}

/// Conservatively prove whether the invoking process may apply numeric ownership.
pub fn ownership_authorization(
    file: &File,
    required_uid: u32,
    required_gid: u32,
) -> io::Result<crate::metadata::model::Evidence<bool>> {
    let stat = rustix::fs::fstat(file).map_err(io::Error::from)?;
    ownership_authorization_for(stat.st_uid, stat.st_gid, required_uid, required_gid)
}

/// Prove a numeric ownership transition from already observed target identities.
pub fn ownership_authorization_for(
    current_uid: u32,
    current_gid: u32,
    required_uid: u32,
    required_gid: u32,
) -> io::Result<crate::metadata::model::Evidence<bool>> {
    let effective_uid = rustix::process::geteuid().as_raw();
    if effective_uid == 0 {
        return Ok(crate::metadata::model::Evidence::Observed { value: true });
    }
    if required_uid != current_uid {
        return Ok(crate::metadata::model::Evidence::Unauthorized {
            reason: "ordinary users cannot prove a numeric owner transition".into(),
        });
    }
    if required_gid == current_gid {
        return Ok(crate::metadata::model::Evidence::Observed { value: true });
    }
    if effective_uid != current_uid {
        return Ok(crate::metadata::model::Evidence::Unauthorized {
            reason: "invoking user does not own the target".into(),
        });
    }
    #[cfg(target_os = "macos")]
    {
        // SAFETY: A zero-sized query returns the required supplementary-group count.
        let count = unsafe { libc::getgroups(0, std::ptr::null_mut()) };
        if count < 0 {
            return Err(io::Error::last_os_error());
        }
        let mut groups = vec![0; count as usize];
        // SAFETY: groups has capacity for the queried number of gid values.
        let read = unsafe { libc::getgroups(count, groups.as_mut_ptr()) };
        if read < 0 {
            return Err(io::Error::last_os_error());
        }
        groups.truncate(read as usize);
        if groups.contains(&required_gid) || rustix::process::getegid().as_raw() == required_gid {
            Ok(crate::metadata::model::Evidence::Observed { value: true })
        } else {
            Ok(crate::metadata::model::Evidence::Unauthorized {
                reason: "required numeric group is not in the invoking user's group set".into(),
            })
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        Ok(crate::metadata::model::Evidence::Unsupported {
            reason: "ownership authorization is qualified only on macOS".into(),
        })
    }
}
