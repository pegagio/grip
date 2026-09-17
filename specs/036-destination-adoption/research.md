# Research: Destination Adoption

## Decision 1: Make adoption a distinct `pull` mode

**Decision**: Add `grip pull -a|--adopt DESTINATION` as an explicit operation branch, rather than extending ordinary pull selection.

**Rationale**: Ordinary tree discovery is source-defined by contract. An explicit mode makes the authority reversal visible at the invocation site and lets normal `pull`, `status`, and `diff` retain their current bounded membership behavior.

**Alternatives considered**:

- Make ordinary pull inspect destination-only members. Rejected because it silently changes tree ownership and could import unrelated destination content.
- Add a nested file mapping. Rejected because it conflicts with tree-mapping ownership and creates duplicate ownership.
- Add a standalone copy command. Rejected because it would duplicate accepted-state, verification, and revalidation behavior already owned by pull/mutation workflows.

## Decision 2: Resolve one destination path directly against declared tree mappings

**Decision**: Resolve the supplied destination selector to exactly one declared tree mapping and derive its raw relative path. Do not call ambient destination discovery or enumerate sibling paths.

**Rationale**: The selected path is the only authority exception. Declared mapping containment, exact-leaf reservations, canonical no-follow path checks, and raw relative identity prevent the operation from crossing ownership boundaries.

**Alternatives considered**:

- Reuse normal `resolve_selection` and discovery unchanged. Rejected because destination-only entries do not appear in source-defined inventory.
- Scan the destination subtree then filter the selector. Rejected because it spends work and creates accidental authority over unselected files.

## Decision 3: Baseline the required ancestor chain

**Decision**: When the paired source parent is absent but a more distant source ancestor exists, create only intervening source directories and publish accepted baselines for those structural directories as well as the selected file.

**Rationale**: Tree discovery treats directories as managed members. Creating an unbaselined source directory would make the immediately following status non-current. The source-ancestor rule therefore implies a bounded structural adoption set, not an unrelated sibling import.

**Alternatives considered**:

- Require the direct source parent to exist. Rejected because the clarified product requirement permits a grandparent or more distant ancestor.
- Create directories but do not baseline them. Rejected because ordinary status would immediately report their state as unaccepted.

## Decision 4: Keep `--force` narrowly scoped to ignored adoption

**Decision**: In adoption mode, `--force` permits only the selected path to proceed despite applicable source-side ignore rules. It neither selects a conflict winner nor bypasses topology, metadata, ownership, stale-evidence, or source-absence checks.

**Rationale**: This preserves the familiar explicit override spelling without granting ordinary force semantics to an unowned path.

**Alternatives considered**:

- Refuse all ignored paths. Rejected because the user needs a deliberate one-time adoption workflow.
- Modify `.gripignore` automatically. Rejected because policy is user-authored configuration and an automatic edit could broaden future membership unexpectedly.

## Decision 5: Treat forced ignored adoption as durable evidence with an explicit retention warning

**Decision**: Publish the verified baseline for the exact forced adoption, leave `.gripignore` unchanged, and emit an exact, placement-qualified negation rule set in human and JSON results. The recommendation must include every ancestor exemption Gitignore-compatible pruning needs and must be checked against the effective policy without changing it.

**Rationale**: The operation succeeded and needs a truthful record, while the warning accurately explains that a later ordinary discovery pass may prune the ignored member unless the operator changes policy. A lone leaf negation cannot reliably revive a leaf beneath an ignored parent directory.

**Alternatives considered**:

- Refuse to publish a baseline for forced ignored adoption. Rejected because the operation could not report its verified result or converge immediately.
- Preserve ignored entries indefinitely outside normal discovery. Rejected because it would make ignore policy non-authoritative and add hidden state behavior.

## Decision 6: Reuse the established mutation safety lifecycle

**Decision**: Model adoption as a first-class mutation operation that uses existing complete-state inspection, pre-action revalidation, staged publication, post-copy verification, and accepted-state publication mechanisms.

**Rationale**: The new authority boundary is narrow, but the filesystem mutation itself has the same failure modes as pull. Reusing the lifecycle avoids a weaker copy-only path.

**Alternatives considered**:

- Invoke an external copy utility. Rejected because it cannot preserve Grip's supported metadata, revalidation, result, or baseline guarantees.
- Add broad locks or filesystem indexing. Rejected as disproportionate to one exact local path and inconsistent with the constitution.
