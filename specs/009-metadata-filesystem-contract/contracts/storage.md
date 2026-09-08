# Storage Contract

Feature 009 evolves accepted and recovery state without weakening Grip's strict envelope, integrity, generation, or publication rules.

## State Envelope V3

State V3 preserves the existing integrity-protected envelope and accepted-state generation model. Each canonical entry baseline contains the complete `SupportedEntryStateV3` from [the data model](../data-model.md#supported-entry-state).

Conceptual payload:

```json
{
  "schema_version": 3,
  "payload": {
    "generation": 12,
    "registry": {},
    "baselines": {
      "<canonical-entry-identity>": {
        "node_kind": "file",
        "content": {"algorithm": "sha256", "digest": "<lowercase-hex>", "length": 42},
        "metadata": {
          "permission_mode": "0644",
          "uid": 501,
          "gid": 20,
          "modified_time": {"seconds": 1700000000, "nanoseconds": 123456789},
          "extended_attributes": [],
          "acl": {"state": "absent"},
          "bsd_flags": []
        }
      }
    }
  },
  "integrity": {"algorithm": "sha256", "digest": "<lowercase-hex>"}
}
```

Canonical serialization, integrity verification, strict unknown-field rejection, registry binding, generation checks, atomic publication, parent-directory durability, and post-publication verification remain mandatory.

## State V2 Compatibility

V2 is decoded strictly as legacy-incomplete accepted evidence. Missing Feature 009 fields are not defaults and are not treated as absent.

- Equal complete current copies: report `metadata_migration_ready`; only explicit `grip baseline accept` may publish V3.
- Unequal complete current copies: report `metadata_migration_conflict`; only explicit whole-entry `grip resolve PATH --source|--destination` may select the winner and publish V3 after synchronization and verification.
- Read-only commands never rewrite state.
- A failed or blocked migration leaves V2 authoritative.

Once a verified V3 generation publishes, the normal strict current-state rules apply. No dual-write or silent downgrade is permitted.

## Recovery Metadata V2

New destructive or metadata-only actions write Recovery Metadata V2 before the prior state becomes unavailable. V2 binds:

- Recovery identifier, operation identifier, plan identifier, action index, and managed entry identity
- Prior complete supported state
- Preserved payload reference when the entry is a file
- Exact xattr preservation evidence by name, length, and digest
- ACL, mode, owner, group, mtime, and BSD-flag preservation evidence
- Preservation verification and durability status
- Recovery-object private security state, separate from the original logical mode and ownership

Large xattr values, including resource forks, remain attached to or otherwise stored with the private recovery payload rather than embedded in accepted-state JSON. Restore must verify their exact bytes against the recorded fingerprints.

Recovery Metadata V1 remains strictly readable and restorable under its original contract. V1 cannot prove the expanded state and therefore cannot authorize a new Feature 009 replacement.

## Operation Record V1

The partitioned Operation Record V1 remains unchanged at the envelope level. Its existing plan, action checkpoint, and summary payload extension points carry:

- Complete expected before and after state
- Field-level changed dimensions
- Capability and authorization findings
- Required flag-clear and metadata-application steps
- Recovery Metadata V2 references
- Complete verification outcomes

The immutable plan is written before payload mutation, checkpoints remain bounded, and terminal summary publication follows existing durability rules.

## Result Envelope V1

The Result Envelope V1 remains the stable public envelope. New typed detail fields are additive and use stable snake-case enum values. Human output and JSON output must render the same classifications, differences, blockers, planned actions, recovery authority, verification, and baseline authority.

Raw synchronized xattr bytes, ACL display names used only for context, and unsafe path bytes must not leak into JSON or logs. Use safe paths, xattr name displays, lengths, and digests.
