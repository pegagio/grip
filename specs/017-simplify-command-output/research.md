# Research: Simplify Default Command Output

## Decision 1: Preserve typed outcomes and change only human projections

**Decision**: Keep existing command outcome details and the JSON envelope unchanged; add concise default-human projections at the renderer boundary.

**Rationale**: Existing outcomes already retain declared mappings, action source/destination paths, per-action directions, completion state, and safety evidence. Rendering from that data changes operator presentation without altering automation contracts or mutation behavior.

**Alternatives considered**:

- Replace detailed values in command outcomes before rendering. Rejected because it would change JSON and discard operational evidence.
- Create a second command-result model. Rejected because the existing typed outcome is sufficient and a second model would duplicate lifecycle state.

## Decision 2: Derive arrows from each mutation action

**Decision**: Render each concise mutation row from its individual source, destination, and direction fields.

**Rationale**: `sync` can contain both directions, so a command-level label cannot determine every row’s source/destination ordering. Per-action direction preserves the approved `SOURCE -> DESTINATION` and `SOURCE <- DESTINATION` forms.

**Alternatives considered**:

- Print stored action path order unchanged. Rejected because destination-to-source actions would not match the approved source-left pull form.
- Split synchronization into separate push and pull headings. Rejected because the approved transcript uses one synchronization heading with per-row arrows.

## Decision 3: Render every terminal public mutation result concisely

**Decision**: Apply a concise human projection to action-bearing success, no-action, blocked, and failed public mutation results. Map the internal `resolve` operation to its public push or pull direction. Preserve structured outcome data and detailed diagnostics.

**Rationale**: The audit found that the generic fallback leaked planner reason codes, conflict-winner fields, baseline authority, action milestones, recovery state, and operation-record identifiers in precisely the common terminal paths an operator needs to understand. `Nothing to …`, conflict rows and force choices, a `grip status` next step for technical blockers, and a status-before-retry next step for failures are enough to operate safely without weakening JSON or diagnostic evidence.

**Alternatives considered**:

- Retain the generic fallback for no-action, blocked, and failed mutations. Rejected because it is the source of the audited implementation-detail leakage.
- Suppress technical blockers without a next step. Rejected because it would leave the operator unable to locate the detailed, current safety evidence in `grip status`.

## Decision 4: Preserve technical blockers while guiding force-resolvable conflicts

**Decision**: Replace the generic `Blocked:` line with two explicit force commands for initial collisions and ordinary divergent conflicts. Retain technical compatibility and unsupported-state blocker details without force guidance.

**Rationale**: A `<->` row identifies the endpoints but does not identify the safe next command. Initial collisions and ordinary divergent conflicts have existing explicit resolution operations; technical blockers do not necessarily do so.

**Alternatives considered**:

- Suppress every blocker detail. Rejected because it would weaken the existing safety contract.
- Retain the generic conflict explanation without a command. Rejected because it leaves the operator unable to choose a side safely.

## Decision 6: Render concrete selectors only in human guidance

**Decision**: Derive a CWD-relative source display for blocked mutation guidance and pair it with the existing destination endpoint under `--destination`.

**Rationale**: `push` rejects absolute source selectors, while status already displays a CWD-relative source path suitable for push. A destination selector is accepted by `pull --force --destination`, so the displayed endpoint supplies a concrete destination-winning command without altering domain selection.

**Alternatives considered**:

- Show abstract placeholders such as `<source path>`. Rejected because they do not answer the operator’s immediate next step.
- Show an absolute source path. Rejected because `push` correctly rejects it.

## Decision 5: Normalize error prefixes at both output entry points

**Decision**: Normalize human parser errors at the process entrypoint and human domain errors at ordinary outcome rendering; leave JSON errors unchanged.

**Rationale**: Parser failures bypass normal command outcome rendering, while domain failures use it. Both must begin with the same operator-visible prefix without changing exit behavior.

**Alternatives considered**:

- Change only `CommandOutcome` rendering. Rejected because argument-parser errors would retain the old lowercase prefix.
- Change error categories or parser behavior. Rejected because the feature changes presentation only.
