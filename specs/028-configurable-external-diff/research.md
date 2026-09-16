# Research: Configurable External Diff Program

## Decision 1: Retain Descriptor V2 with a narrow optional profile surface

**Decision**: Continue requiring `schema_version = 2` for `.grip/config.toml`, retaining strict mapping validation while accepting only optional `diff` and `difftool` keys alongside mappings.

**Rationale**: Descriptor V2 is the selected project's strict mapping document, used by project selection and registry publication. A version bump would force users to migrate unchanged mappings for a tool preference. The decoder retains an accepted profile fragment and defers typed profile validation until an eligible external-diff request, so malformed diff settings do not change existing JSON inspection or mapping commands. Mapping publication must preserve the full descriptor rather than re-encoding mappings alone.

**Alternatives rejected**:

- Validate profile values during every descriptor read: turns a diff-only error into failure of unrelated commands.
- Introduce Descriptor V3: imposes an unrelated migration boundary on existing projects.
- Store project settings in a second file: fragments the existing selected-project configuration location.

## Decision 2: Keep the machine-wide file read-only and preference-only

**Decision**: Load `~/.grip/config.toml` only as an optional diff-profile layer. A missing directory or file is empty configuration; Grip never creates, writes, or repairs it.

**Rationale**: The user chose Git-like global-then-project precedence, but mappings and accepted state remain project-owned. A present profile and containing directory must meet the same current-user-owned, non-symlink, safe-mode, no-follow safety posture as existing project metadata.

**Alternatives rejected**:

- Make it a global mapping registry or state location: violates Grip ownership and persistence boundaries.
- Accept unsafe files: allows an untrusted executable selection into a local command boundary.
- Auto-create configuration: adds persistent user state to a read-only inspection feature.

## Decision 3: Merge named tools field-by-field

**Decision**: Merge machine-wide values first and selected-project values second. Matching scalar fields replace; a project `args` array replaces the whole global array. Resolve the selected tool only after merging.

**Rationale**: A project can select a globally defined tool or override program/arguments predictably. An explicit `[]` has meaning and must not concatenate with the global array.

**Alternatives rejected**:

- Require fully project-defined tools: prevents global tool reuse.
- Append arrays: violates the clarified replacement requirement.
- Permit command strings: introduces shell quoting and execution ambiguity.

## Decision 4: Give the environment override a complete bypass

**Decision**: A non-empty `GRIP_EXTERNAL_DIFF` provides only the executable. It bypasses named-tool selection and receives just source and destination; extra arguments require a wrapper executable.

**Rationale**: This is per-invocation, unambiguous, and direct without a second argument-parsing language.

**Alternatives rejected**:

- Parse arguments from the variable: reintroduces ambiguous shell-like parsing.
- Combine it with configured arguments: makes an override depend on persistent settings.

## Decision 5: Prepare one direct handoff after exact inspection

**Decision**: Preserve the inspection pipeline and create a typed handoff only for human `diff` with a selector. Derive direct endpoints from that exact selection and validate both immediately before spawn.

**Rationale**: The pipeline owns selector spaces, registry/state loading, classification, and read-only revalidation. Tree descendants do not prove a selected mapping-root directory is safe, so the root needs direct checks.

**Alternatives rejected**:

- Reconstruct paths from rendered output: loses typed safety evidence.
- Launch for unselected diff: converts broad diagnostics into multi-process action.
- Stage copies: adds lifecycle and copy semantics excluded by the specification.

## Decision 6: Keep child exits outside Grip result categories

**Decision**: The process entrypoint distinguishes ordinary rendered outcomes from a prepared handoff. It uses `std::process::Command`, inherited terminal I/O, exact normal exit propagation, and Unix `128 + signal` handling.

**Rationale**: `CommandOutcome` has fixed category exit codes; a normal nonzero diff result is not a Grip error and must not be remapped.

**Alternatives rejected**:

- Map child exits to result categories: loses the required status.
- Use a shell: violates literal argument safety.
- Hide signals as generic success/failure: loses child termination semantics.
