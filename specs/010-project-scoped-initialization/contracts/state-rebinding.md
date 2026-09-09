# Contract: Local State Rebinding

## Purpose

A move or filesystem copy may retain `.grip/state/` while changing the project root, user home, or endpoints. Retained state is useful evidence, but is never trusted through prefix rewriting or a generated identifier.

## Detection and Authority

State V4 carries a local binding with publication-time project root, home root, descriptor digest, and resolved mapping-set digest. Any mismatch marks state untrusted and starts complete validation. Integrity or schema failure is corrupt or unsupported state, not a rebind candidate.

Every baseline, operation action, pending retirement, recovery manifest, and recovery metadata record uses a portable mapping tuple, entry-relative bytes, and endpoint role. Absolute paths may be diagnostic only and never determine a current target.

## Complete Validation

Before retained state may authorize acceptance or mutation, Grip must:

1. Verify schema, integrity, ordering, and portable grammar.
2. Revalidate project metadata, exact descriptor, and canonical home.
3. Resolve the complete descriptor and validate ownership topology.
4. Match each identity to exactly one mapping or explicit pending-retirement record.
5. Reconstruct endpoints through current evidence without symlink escape.
6. Reobserve every accepted entry required by the command and compare complete state.
7. Validate every recovery and unfinished-operation target the command could affect.
8. Revalidate after acquiring project-local locks and immediately before each action.

No step may infer authority by replacing an old project or home prefix.

## Outcomes

| Outcome | Read-only or dry-run | Acceptance or mutation |
|---|---|---|
| Binding and evidence valid | Report normally. | Proceed under existing rules. |
| Binding differs; complete evidence valid | Report `rebind_eligible`; write nothing. | Proceed after locked revalidation; final state publication records current binding. |
| Identity missing, ambiguous, or unsafe | Report `rebind_blocked`. | Block. |
| Evidence stale, incomplete, or contradictory | Report `rebind_blocked`. | Block. |
| Schema or integrity invalid | Report existing category. | Block. |

A successful in-memory rebind does not itself publish state. Read-only and dry-run commands preserve exact bytes.

## Clone and Recovery Rules

A Git clone with Descriptor V2 and `.gitignore` but no state is initialized with uninitialized local state. It never searches another clone or global path. Separate projects use disjoint state and locks.

Recovery inventory may describe retained evidence read-only, but restore and cleanup resolve references only within the selected state root. Restore derives live targets from portable identity and endpoint role. Descriptor recovery always targets the selected `.grip/config.toml`; stored absolute targets cannot redirect it.

State versions 1–3 and prior absolute-authority operation and recovery versions are unsupported. Grip does not import, rewrite, or migrate global registry or state data.
