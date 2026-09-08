//! Descriptor-relative, non-following filesystem inspection.

use crate::discovery::model::{EvidenceSide, NodeEvidence, NodeKind};
use rustix::fd::{AsFd, BorrowedFd, OwnedFd};
use rustix::fs::{AtFlags, CWD, Dir, FileType, Mode, OFlags, Stat, fstat, open, openat, statat};
use std::ffi::CString;
use std::io;
#[cfg(target_os = "macos")]
use std::os::fd::AsRawFd;
#[cfg(target_os = "macos")]
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

/// An open directory used as the authority for relative child inspection.
#[derive(Debug)]
pub struct Directory {
    descriptor: OwnedFd,
    metadata: Stat,
}

impl Directory {
    /// Open an absolute directory without following its final path component.
    pub fn open(path: &Path) -> io::Result<Self> {
        let descriptor = open(
            path,
            OFlags::RDONLY
                | OFlags::DIRECTORY
                | OFlags::CLOEXEC
                | OFlags::NOFOLLOW
                | OFlags::NONBLOCK,
            Mode::empty(),
        )
        .map_err(errno)?;
        let metadata = fstat(&descriptor).map_err(errno)?;
        Ok(Self {
            descriptor,
            metadata,
        })
    }

    /// Return the root metadata captured from the opened descriptor.
    pub const fn root_metadata(&self) -> &Stat {
        &self.metadata
    }

    /// Borrow the opened directory descriptor for descriptor-relative operations.
    pub(crate) fn as_fd(&self) -> BorrowedFd<'_> {
        self.descriptor.as_fd()
    }

    /// Enumerate exact child names in deterministic raw-byte order.
    pub fn child_names(&self) -> io::Result<Vec<Vec<u8>>> {
        let mut directory = Dir::read_from(&self.descriptor).map_err(errno)?;
        let mut names = Vec::new();
        for entry in &mut directory {
            let entry = entry.map_err(errno)?;
            let name = entry.file_name().to_bytes();
            if name != b"." && name != b".." {
                names.push(name.to_vec());
            }
        }
        names.sort();
        Ok(names)
    }

    /// Inspect a final child without following symbolic links.
    pub fn metadata(&self, name: &[u8]) -> io::Result<NodeMetadata> {
        let name = child_name(name)?;
        let stat =
            statat(&self.descriptor, name.as_c_str(), AtFlags::SYMLINK_NOFOLLOW).map_err(errno)?;
        Ok(NodeMetadata {
            stat,
            extended_flags: extended_flags_at(self.descriptor.as_raw_fd(), name.as_c_str()).ok(),
        })
    }

    /// Open a child directory relative to this directory without following links.
    pub fn open_child_directory(&self, name: &[u8]) -> io::Result<Self> {
        let name = child_name(name)?;
        let descriptor = openat(
            &self.descriptor,
            name.as_c_str(),
            OFlags::RDONLY
                | OFlags::DIRECTORY
                | OFlags::CLOEXEC
                | OFlags::NOFOLLOW
                | OFlags::NONBLOCK,
            Mode::empty(),
        )
        .map_err(errno)?;
        let metadata = fstat(&descriptor).map_err(errno)?;
        Ok(Self {
            descriptor,
            metadata,
        })
    }

    /// Open a regular child policy file relative to this directory without following links.
    pub fn open_child_file(&self, name: &[u8]) -> io::Result<OwnedFd> {
        let name = child_name(name)?;
        openat(
            &self.descriptor,
            name.as_c_str(),
            OFlags::RDONLY | OFlags::CLOEXEC | OFlags::NOFOLLOW | OFlags::NONBLOCK,
            Mode::empty(),
        )
        .map_err(errno)
    }
}

/// Raw non-following metadata for one observed node.
#[derive(Debug, Clone, Copy)]
pub struct NodeMetadata {
    pub stat: Stat,
    pub extended_flags: Option<u64>,
}

impl NodeMetadata {
    /// Return the filesystem node type encoded in the raw mode.
    pub fn file_type(&self) -> FileType {
        FileType::from_raw_mode(self.stat.st_mode)
    }

    /// Classify a node against the source root's filesystem allowlist.
    pub fn classify(&self, root_device: u64) -> NodeKind {
        classify_metadata(self, root_device)
    }
}

/// Inspect an exact path without following its final component.
pub fn metadata_at_path(path: &Path) -> io::Result<NodeMetadata> {
    statat(CWD, path, AtFlags::SYMLINK_NOFOLLOW)
        .map(|stat| NodeMetadata {
            stat,
            #[cfg(target_os = "macos")]
            extended_flags: CString::new(path.as_os_str().as_bytes())
                .ok()
                .and_then(|path| extended_flags_at(libc::AT_FDCWD, path.as_c_str()).ok()),
            #[cfg(not(target_os = "macos"))]
            extended_flags: None,
        })
        .map_err(errno)
}

/// Convert raw metadata into pass-comparison evidence.
pub fn evidence(
    metadata: &NodeMetadata,
    side: EvidenceSide,
    mapping_source: &Path,
    relative_bytes: &[u8],
    child_names: Option<Vec<Vec<u8>>>,
) -> NodeEvidence {
    let stat = &metadata.stat;
    NodeEvidence {
        side,
        mapping_source: mapping_source.to_path_buf(),
        relative_bytes: relative_bytes.to_vec(),
        device: to_u64(stat.st_dev),
        inode: to_u64(stat.st_ino),
        mode: to_u32(stat.st_mode),
        link_count: to_u64(stat.st_nlink),
        size: to_u64(stat.st_size),
        allocated_blocks: to_u64(stat.st_blocks),
        modified_seconds: to_i64(stat.st_mtime),
        modified_nanoseconds: to_i64(stat.st_mtime_nsec),
        changed_seconds: to_i64(stat.st_ctime),
        changed_nanoseconds: to_i64(stat.st_ctime_nsec),
        child_names,
    }
}

fn to_u64<T>(value: T) -> u64
where
    T: TryInto<u64>,
{
    value.try_into().unwrap_or(0)
}

fn to_u32<T>(value: T) -> u32
where
    T: TryInto<u32>,
{
    value.try_into().unwrap_or(0)
}

fn to_i64<T>(value: T) -> i64
where
    T: TryInto<i64>,
{
    value.try_into().unwrap_or(0)
}

/// Apply the ordered node allowlist to raw metadata.
pub fn classify(stat: &Stat, root_device: u64) -> NodeKind {
    match FileType::from_raw_mode(stat.st_mode) {
        FileType::Symlink => NodeKind::Symlink,
        FileType::Socket => NodeKind::Socket,
        FileType::Fifo => NodeKind::Fifo,
        FileType::CharacterDevice => NodeKind::CharacterDevice,
        FileType::BlockDevice => NodeKind::BlockDevice,
        FileType::Directory if stat.st_dev as u64 != root_device => NodeKind::NestedMount,
        FileType::Directory => NodeKind::Directory,
        FileType::RegularFile if stat.st_nlink > 1 => NodeKind::HardLink,
        FileType::RegularFile => NodeKind::File,
        FileType::Unknown => {
            #[cfg(any(target_os = "macos", target_os = "ios", target_os = "freebsd"))]
            {
                const S_IFWHT: u32 = 0o160000;
                if (stat.st_mode as u32) & 0o170000 == S_IFWHT {
                    return NodeKind::Whiteout;
                }
            }
            NodeKind::UnknownSpecial
        }
    }
}

/// Apply authoritative macOS extended-flag evidence when classifying a node.
pub fn classify_metadata(metadata: &NodeMetadata, root_device: u64) -> NodeKind {
    let basic = classify(&metadata.stat, root_device);
    if basic == NodeKind::File
        && metadata
            .extended_flags
            .is_some_and(|flags| flags & 0x0000_0010 != 0)
    {
        NodeKind::SparseFile
    } else {
        basic
    }
}

#[cfg(target_os = "macos")]
fn extended_flags_at(fd: libc::c_int, path: &std::ffi::CStr) -> io::Result<u64> {
    let mut attributes = libc::attrlist {
        bitmapcount: 5,
        reserved: 0,
        commonattr: 0,
        volattr: 0,
        dirattr: 0,
        fileattr: 0,
        forkattr: 0x0000_0200,
    };
    let mut buffer = [0_u8; 16];
    // SAFETY: all pointers refer to valid storage and flags prohibit link following.
    let result = unsafe {
        libc::getattrlistat(
            fd,
            path.as_ptr(),
            (&mut attributes as *mut libc::attrlist).cast(),
            buffer.as_mut_ptr().cast(),
            buffer.len(),
            0x0000_0020 | 0x0000_0001 | 0x0000_0800,
        )
    };
    if result != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(u64::from_ne_bytes(
        buffer[4..12].try_into().expect("fixed flag slice"),
    ))
}

/// Return the stable reason associated with an unsupported node kind.
pub const fn unsupported_reason(kind: NodeKind) -> Option<&'static str> {
    match kind {
        NodeKind::File | NodeKind::Directory => None,
        NodeKind::Symlink => Some("symlink"),
        NodeKind::HardLink => Some("hard_link"),
        NodeKind::SparseFile => Some("sparse_file"),
        NodeKind::Socket => Some("socket"),
        NodeKind::Fifo => Some("fifo"),
        NodeKind::CharacterDevice => Some("character_device"),
        NodeKind::BlockDevice => Some("block_device"),
        NodeKind::Whiteout => Some("whiteout"),
        NodeKind::UnknownSpecial => Some("unknown_special"),
        NodeKind::NestedMount => Some("nested_mount"),
    }
}

fn child_name(bytes: &[u8]) -> io::Result<CString> {
    if bytes.is_empty() || bytes == b"." || bytes == b".." || bytes.contains(&b'/') {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "child name must be one safe path component",
        ));
    }
    CString::new(bytes).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "child name must not contain NUL",
        )
    })
}

fn errno(error: rustix::io::Errno) -> io::Error {
    io::Error::from_raw_os_error(error.raw_os_error())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::symlink;

    #[test]
    fn directory_lists_exact_names_in_raw_byte_order() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("z"), "z").unwrap();
        std::fs::write(root.path().join("a"), "a").unwrap();
        let directory = Directory::open(root.path()).unwrap();
        assert_eq!(
            directory.child_names().unwrap(),
            vec![b"a".to_vec(), b"z".to_vec()]
        );
    }

    #[test]
    fn child_metadata_does_not_follow_a_final_symlink() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("target"), "payload").unwrap();
        symlink("target", root.path().join("link")).unwrap();
        let directory = Directory::open(root.path()).unwrap();
        let metadata = directory.metadata(b"link").unwrap();
        assert_eq!(metadata.file_type(), rustix::fs::FileType::Symlink);
    }

    #[test]
    fn child_directories_open_relative_to_the_parent_descriptor() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir(root.path().join("child")).unwrap();
        let directory = Directory::open(root.path()).unwrap();
        let child = directory.open_child_directory(b"child").unwrap();
        assert!(child.child_names().unwrap().is_empty());
    }

    #[test]
    fn raw_mode_classification_covers_supported_and_unsupported_kinds() {
        let root = tempfile::tempdir().unwrap();
        let file = root.path().join("file");
        std::fs::write(&file, "payload").unwrap();
        let mut stat = metadata_at_path(&file).unwrap().stat;
        let device = stat.st_dev as u64;
        for (file_type, expected) in [
            (rustix::fs::FileType::Symlink, NodeKind::Symlink),
            (rustix::fs::FileType::Socket, NodeKind::Socket),
            (rustix::fs::FileType::Fifo, NodeKind::Fifo),
            (
                rustix::fs::FileType::CharacterDevice,
                NodeKind::CharacterDevice,
            ),
            (rustix::fs::FileType::BlockDevice, NodeKind::BlockDevice),
            (rustix::fs::FileType::Unknown, NodeKind::UnknownSpecial),
        ] {
            stat.st_mode = file_type.as_raw_mode();
            assert_eq!(classify(&stat, device), expected);
        }
        stat.st_mode = rustix::fs::FileType::Directory.as_raw_mode();
        assert_eq!(classify(&stat, device), NodeKind::Directory);
        assert_eq!(classify(&stat, device + 1), NodeKind::NestedMount);
        stat.st_mode = rustix::fs::FileType::RegularFile.as_raw_mode();
        stat.st_nlink = 2;
        assert_eq!(classify(&stat, device), NodeKind::HardLink);
        stat.st_nlink = 1;
        stat.st_size = 4096;
        stat.st_blocks = 0;
        assert_eq!(classify(&stat, device), NodeKind::File);
        assert_eq!(
            classify_metadata(
                &NodeMetadata {
                    stat,
                    extended_flags: Some(0x10)
                },
                device,
            ),
            NodeKind::SparseFile
        );
        stat.st_size = 1;
        stat.st_blocks = 1;
        assert_eq!(classify(&stat, device), NodeKind::File);

        #[cfg(any(target_os = "macos", target_os = "ios", target_os = "freebsd"))]
        {
            stat.st_mode = 0o160000;
            assert_eq!(classify(&stat, device), NodeKind::Whiteout);
        }
    }
}
