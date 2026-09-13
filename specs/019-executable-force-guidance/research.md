# Research: Executable Force-Resolution Guidance

This research records the narrow decisions that keep the user-facing correction aligned with Grip's existing safety and selector contracts.

## Decision: Preserve exact-entry force scope

**Decision**: Do not make `grip push --force TREE` or `grip pull --force --destination TREE_DEST` resolve an aggregate tree mapping.

**Rationale**: `exact_resolution_selection` deliberately accepts one exact managed entry and rejects an aggregate mapping without an accepted baseline for that root. Expanding force to cover a tree would turn an output defect into a broader mutation-policy change with a larger safety surface.

**Alternatives considered**:

- Expand force resolution to traverse a selected tree: rejected because it changes established force semantics and could apply a winner to multiple entries.
- Continue showing force commands whenever classification says collision or divergence: rejected because the displayed command can fail validation, which violates the user-facing contract.

## Decision: Derive guidance from selector eligibility, not conflict classification

**Decision**: Extract or reuse the criteria embodied by `exact_resolution_selection` when deciding whether to render force commands.

**Rationale**: `initial_collision` and `divergent_conflict` describe why attention is needed, not whether a rendered selector resolves to one entry. A file root may be an exact entry, a tree child can be exact when its accepted baseline establishes that identity, and an unbaselined tree mapping root is aggregate. The existing selection authority already captures those distinctions.

**Alternatives considered**:

- Check only the classification enum: rejected because it caused the reported invalid tree-root command.
- Check only whether a baseline exists: rejected because an exact file entry can still be a valid force target without an accepted baseline.

## Decision: Use `grip diff SOURCE` for aggregate conflicts

**Decision**: For an aggregate conflict that cannot be force-resolved as one exact entry, show `Run: grip diff SOURCE` using the same source-path display selector shown in status.

**Rationale**: `diff` is read-only, already understands the source selector, and lets the user find the exact conflicted entries before choosing an existing force action. The feature explicitly leaves diff behavior unchanged.

**Alternatives considered**:

- Show no command: rejected because it leaves the user without a useful next step.
- Show `grip status SOURCE`: rejected because status repeats the summary instead of inspecting the difference.

## Decision: Keep guidance non-serialized

**Decision**: Represent force-pair or inspect-diff guidance as internal human-output data, not as a new JSON field.

**Rationale**: The scope is a human-output correction. Existing JSON contracts and machine consumers must remain stable, while the renderer needs enough context to make status and blocked mutation guidance consistent.

## Decision: Validate rendered commands through CLI contracts

**Decision**: Add fixtures that distinguish an exact conflict from an aggregate tree-root conflict and assert both the output and acceptance of every shown force command.

**Rationale**: A string-only snapshot would not detect the reported defect. A dry-run invocation can exercise selector acceptance without mutating endpoints; aggregate output is verified to contain the diff instruction and no force command.
