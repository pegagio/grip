# Contract: Project Metadata and Local State Storage

## Layout

```text
<project>/
├── .grip/
│   ├── config.toml
│   ├── .gitignore
│   └── state/                         # Lazy and ignored
│       ├── locks/
│       │   ├── mutation.lock
│       │   ├── registry.lock
│       │   └── state.lock
│       ├── staging/
│       ├── state.json
│       ├── operations/
│       └── recovery/
└── managed source content
```

Every mutable or machine-specific Grip artifact is below `.grip/state/`. No global path participates in selection, locking, publication, recovery, or state lookup.

## Descriptor V2

The initialized descriptor is:

```toml
schema_version = 2
mappings = []
```

A populated descriptor uses portable strings only:

```toml
schema_version = 2

[[mappings]]
kind = "tree"
source = "Users/pegagio"
destination = "~"
```

The decoder denies unknown fields, validates path grammar, resolves all declarations in one context, and rejects the complete document on topology or safety conflict. Encoding is deterministic and never writes a resolved absolute path.

## Git Exclusion

`.grip/.gitignore` has exact bytes:

```gitignore
/state/
```

The anchored rule ignores local state relative to `.grip/` while leaving `config.toml` trackable. The file is Grip-owned: missing, additional, reordered, or contradictory content is non-equivalent metadata and is not repaired automatically.

Grip does not require Git, run Git, create a repository, stage files, or inspect repository status.

## Permissions and Nodes

- The project root and `.grip/` are current-user-owned non-symlink directories with no group/world write bits; ordinary `0755` clone directories are valid.
- `config.toml` and `.gitignore` are current-user-owned non-symlink regular files, readable, and not group/world writable; ordinary `0644` files are valid. Descriptor publication also requires owner write permission.
- `.grip/state/` and its directories are current-user-owned, non-symlink, and `0700`.
- Private state files are current-user-owned, non-symlink, and `0600`.
- Every lookup and publication uses non-following checks and identity revalidation.

## Publication

Initialization stages and syncs both files in a unique private directory within the target, revalidates the target, renames without replacement to `.grip`, and syncs the project root. Failure removes only its own staging nodes.

Mapping writers acquire project-local mutation and registry locks, revalidate the descriptor and mappings, retain prior bytes under `.grip/state/recovery/registry/`, stage beneath `.grip/state/staging/`, verify, atomically rename to `.grip/config.toml`, and sync `.grip/`.

Feature 010 publishes State V4 and new operation and recovery schemas wherever earlier payloads held absolute authority. Older schemas are rejected without conversion. Read-only and dry-run commands create no state. Actual mapping publication may lazily create state for locking and recovery; initialization never does.

## Reserved Namespace

`.grip/` is structurally excluded from payload discovery, selection, accepted state, mutation plans, and recovery targets before `.gripignore` evaluation. This remains true for a project-root tree mapping. Descriptors and resolved topology cannot overlap metadata or state.
