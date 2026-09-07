# Domain Contract: Recovery Inventory, Restore, and Cleanup

## Recovery references

```text
payload:<operation-id>:<action-index>
registry:sha256:<64-lower-hex>
state:generation:<generation>:sha256:<64-lower-hex>
operation:<operation-id>
```

References are opaque public identities. Parsing validates tagged components and containment; it never treats user text as a private relative or absolute path.

## Inventory

`recovery list` enumerates the existing operation payload, registry, accepted-state, and operation evidence stores without a central index. `recovery show` strictly validates one selected reference and, where safe, verifies its recoverable bytes. Neither command prints payload contents by default or mutates state.

Each entry reports kind, origin, bound side/path where applicable, managed identity, creation time when known, integrity, provenance completeness, availability (`available`, `cleaned`, `cleanup_incomplete`, or `missing`), byte count, and restore eligibility or blocker.

Historical entries lacking Recovery Manifest V1 are projected as legacy evidence and never rewritten. Missing facts are explicit. A legacy entry is restorable only if existing immutable operation and authority evidence independently prove every required binding; otherwise it reports `legacy_provenance_incomplete`.

## Restore eligibility

Payload restore requires:

- one integrity-valid payload reference and verified bytes;
- complete manifest or equivalent immutable legacy provenance;
- safe current mapping ownership and accepted membership;
- exact bound original target and no-follow ancestry;
- an absent target or an occupied target exactly equal to the recorded verified post-action state.

An occupied eligible target is first preserved as recovery for the restore operation. Restoration stages, publishes, and verifies the prior supported state, retains the source recovery entry, and does not accept a new baseline.

Registry restore requires exact digest verification, strict Registry V1 decoding, complete ownership validation, safe endpoints, transition provenance, and compatibility with live accepted-state and payload evidence. Accepted-state restore requires strict State V2 decoding/integrity/generation validation, transition provenance, compatible live registry, safe paths, and complete live payload observation; payload drift from historical baselines may remain.

A valid current authority must equal the recorded post-transition document. A newer or different valid authority blocks. A missing or corrupt target being repaired need not validate; the recovered candidate and every remaining authority must.

## Cleanup

`recovery remove` accepts only unique exact payload, registry, or accepted-state references. Operation references are inspectable provenance and are not cleanup targets. Execute requires `--confirm`; preview is non-mutating.

After full preflight and lock-held plan reconstruction, each reference is processed in canonical order:

1. checkpoint cleanup intent;
2. unlink only descriptor-bound recoverable bytes within the private recovery namespace;
3. synchronize the containing directory and verify absence;
4. atomically publish an integrity-protected cleanup tombstone;
5. checkpoint completion.

The immutable manifest and operation provenance remain. The first failure stops later cleanup actions. Byte absence without a valid tombstone is `cleanup_incomplete`, not successful cleanup; a fresh explicit retry may complete the tombstone after revalidation.
