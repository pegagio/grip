# CLI Contract

Feature 009 adds no command or flag. It extends the established commands with complete metadata inspection, classification, planning, recovery, verification, and accepted-state publication.

## Existing Commands

| Command | Feature 009 behavior |
|---|---|
| `grip status [PATH]` | Reports complete field differences, migration state, excluded/unknown xattrs, unsupported nodes, collisions, and capability findings. |
| `grip check [PATH]` | Applies the same complete-state and compatibility checks using the existing check semantics. |
| `grip diff [PATH]` | Shows field-level metadata differences without exposing raw xattr bytes. |
| `grip baseline accept [PATH]` | Explicitly publishes State V3 only when complete current copies are equal and fully verified. |
| `grip push [PATH] [--dry-run]` | Plans or applies the complete source entry state to the destination. |
| `grip pull [PATH] [--dry-run]` | Plans or applies the complete destination entry state to the source. |
| `grip sync [PATH] [--dry-run]` | Uses existing three-way direction rules over the complete entry state. |
| `grip resolve PATH --source|--destination [--dry-run]` | Selects one complete entry winner, including metadata migration conflicts. |
| Existing recovery commands | Display and restore Recovery Metadata V2 while retaining V1 compatibility. |

## Stable Classifications

The existing classifications remain. Two migration-specific classifications are added:

- `metadata_migration_ready`: the authoritative baseline lacks Feature 009 fields and the complete current copies are equal.
- `metadata_migration_conflict`: the authoritative baseline lacks Feature 009 fields and the complete current copies differ.

Metadata-only changes use the existing source-only, destination-only, converged, and divergent classifications. They are not a separate weaker synchronization mode.

## Field Differences

Each record may report these stable dimensions: `node_kind`, `content`, `permission_mode`, `owner`, `group`, `modification_time`, `extended_attribute`, `access_control_list`, and `bsd_flags`. An xattr difference identifies the safe attribute name and transition without printing its raw value.

## Compatibility Findings

Machine output for a finding includes:

```json
{
  "endpoint": "destination",
  "path": {"display": "example/file"},
  "field": "extended_attribute",
  "required": {"name": "com.example.unknown"},
  "evidence_state": "unsupported",
  "reason": "unknown_extended_attribute",
  "blocking": true
}
```

Human output conveys the same endpoint, path, field, required state, reason, and blocker status. Excluded xattrs use a nonblocking `excluded_extended_attribute` reason. Diagnostic logs remain separate.

## Planning and Results

Dry-run output includes the exact complete-state transition, action dependencies, directory-finalization order, any supported flag clearing, recovery requirement, and expected verification. It performs no probe writes, metadata mutation, recovery creation, operation-record publication, or baseline publication.

A blocker anywhere in the selected scope prevents every payload mutation in that invocation. Execution drift after mutation begins uses the existing precise partial-failure category and exposes recovery authority. Successful results identify the authoritative accepted-state generation.

## Exit Categories

Feature 009 preserves existing result and exit categories. Metadata differences that are eligible for synchronization are not command failures. Unknown, unavailable, unsupported, unreadable, unauthorized, collision, and migration-conflict conditions map through the existing attention/blocking categories rather than introducing platform-specific exit codes.

## Safety and Privacy

CLI parsing remains separate from domain and filesystem behavior. Paths use the existing safe representation. Output never prints raw xattr values, recovery payload bytes, or locale-derived identity as authoritative data. No workflow invokes privilege escalation or follows symbolic links.
