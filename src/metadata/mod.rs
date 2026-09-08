//! Complete metadata evidence and platform-specific filesystem operations.

pub mod macos;
pub mod model;

/// Exact extended attributes synchronized by the initial macOS contract.
pub const SYNCHRONIZED_XATTRS: [&[u8]; 3] = [
    b"com.apple.FinderInfo",
    b"com.apple.ResourceFork",
    b"com.apple.TextEncoding",
];

/// Exact known attributes reported but excluded from equality and transfer.
pub const EXCLUDED_XATTRS: [&[u8]; 7] = [
    b"com.apple.quarantine",
    b"com.apple.provenance",
    b"com.apple.macl",
    b"com.apple.metadata:kMDItemWhereFroms",
    b"com.apple.metadata:kMDItemDownloadedDate",
    b"com.apple.lastuseddate#PS",
    b"com.apple.root.installed",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XattrPolicy {
    Synchronized,
    Excluded,
    Unknown,
}

pub fn xattr_policy(name: &[u8]) -> XattrPolicy {
    if SYNCHRONIZED_XATTRS.contains(&name) {
        XattrPolicy::Synchronized
    } else if EXCLUDED_XATTRS.contains(&name) {
        XattrPolicy::Excluded
    } else {
        XattrPolicy::Unknown
    }
}
