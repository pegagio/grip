# Feature Specification: Configurable External Diff Program

**Feature Branch**: `028-configurable-external-diff`

**Created**: 2026-09-16

**Status**: Complete

**Input**: User description: "Make `grip diff` open the selected comparison in a configurable external diff program, with `diff` as the default executable."

## Clarifications

### Session 2026-09-16

- **Configuration boundary**: Grip reads machine-wide `~/.grip/config.toml` settings first, then selected-project `.grip/config.toml` settings, with each project setting overriding the same machine-wide key. A project may select a named tool defined only in the machine-wide configuration. Each named tool has one executable reference and an ordered list of literal argument tokens; a selected-project `args` array replaces rather than appends to the machine-wide array. The per-invocation `GRIP_EXTERNAL_DIFF` executable override bypasses named-tool configuration and receives Grip's fixed pair of resolved endpoints with no configured arguments. When no tool or environment override applies, Grip uses `diff` with no configured arguments.
- **Invocation boundary**: Grip selects one named tool from the `[diff]` configuration section and resolves it from a corresponding `[difftool.<name>]` section. It supplies configured arguments and the source and destination paths as separate final arguments; it never evaluates a shell command string.
- Q: How should Grip interpret a nonzero exit code from a configured comparison program? → A: Grip returns the exit code from the launched comparison program unchanged.
- Q: Where should an operator’s chosen diff executable and arguments be stored? → A: Use `~/.grip/config.toml` for the machine-wide default and the existing selected-project `.grip/config.toml` for the project-specific override, following Git-like precedence.
- Q: When no path is supplied, should human-readable `grip diff` launch a tool for every eligible managed difference or require one selected path? → A: Require one selected path for a human external launch; keep unselected human inspection read-only.
- Q: How should an operator set or change the selected diff tool configuration? → A: Use documented machine-wide and selected-project named-tool configuration, with `GRIP_EXTERNAL_DIFF` overriding the applicable configuration for one invocation.
- Q: How should environment variables represent the executable and its literal argument tokens? → A: Use `GRIP_EXTERNAL_DIFF` as an executable-only override that bypasses configured arguments and receives the two resolved endpoints; use a wrapper executable when additional arguments are needed.
- Q: Where should Grip store the machine-local per-user default and per-project override files? → A: Use `~/.grip/config.toml` for the machine-wide default and the existing selected-project `.grip/config.toml` for the project-specific override.
- Q: What shape should the diff profile use in each TOML configuration file? → A: Use Git-style named tool selection: `[diff] tool = "<name>"` and `[difftool.<name>]` with `program` and an ordered `args` array, while retaining direct, non-shell invocation.
- Q: When the selected project chooses a named tool that is defined only in `~/.grip/config.toml`, should Grip use that machine-wide tool definition? → A: Yes. Merge machine-wide and selected-project configuration by key, with selected-project values overriding matching machine-wide keys.
- Q: When a project defines `args` for an inherited named tool, should its array replace or append the machine-wide array? → A: Replace the entire array.
- Q: If the external comparison program is terminated by a signal instead of returning an exit code, what exit code should Grip return? → A: Return `128 + signal number` and identify the signal in the diagnostic.
- **Comparison result**: After a comparison program starts, Grip reports its completion and returns its exit code unchanged; Grip-owned preflight and launch failures remain precise failures because no program exit code exists.
- **Endpoint boundary**: Both selected endpoints must be present, supported, and safe before Grip launches the program. File and directory comparisons use the resolved selected endpoints directly; Grip creates no temporary comparison material.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Open a selected comparison (Priority: P1)

An operator can run `grip diff` for a managed file or directory and have their configured comparison program receive the selected source and destination paths without manually locating either endpoint.

**Why this priority**: The feature makes detailed inspection immediately useful at the point where Grip identifies a managed difference.

**Independent Test**: In an isolated project with a selected managed file and a configured recording comparison program, run `grip diff SOURCE` and confirm the program receives the configured literal arguments followed by the resolved source and destination paths.

**Acceptance Scenarios**:

1. **Given** a selected managed file with present, supported endpoints and no diff profile, **When** the operator runs `grip diff SOURCE`, **Then** Grip launches the default `diff` executable with the source and destination paths as separate arguments.
2. **Given** a selected managed file or directory and a configured named tool, **When** the operator runs `grip diff` with the existing source-space or destination-space selector, **Then** Grip resolves the same managed comparison as before and launches the configured executable with each configured token and both selected endpoints as distinct arguments.
3. **Given** an operator runs human-readable `grip diff` without a selector, **When** one or more managed comparisons are in scope, **Then** Grip preserves the existing broad read-only inspection result and does not launch an external program.

---

### User Story 2 - Configure a safe comparison program (Priority: P2)

An operator can select a named machine-wide comparison tool and, when needed, select or define a selected-project tool or use a per-invocation external-diff executable override without turning a command string into shell input or changing mapping and synchronization behavior.

**Why this priority**: Operators need support for their chosen diff tool, while the command boundary must remain predictable and safe.

**Independent Test**: Configure an executable reference and literal tokens containing spaces or shell-significant characters, run a selected comparison, and confirm they reach the program unchanged without shell interpretation.

**Acceptance Scenarios**:

1. **Given** a selected project, **When** its `.grip/config.toml` selects a named diff tool, **Then** Grip uses that selected tool in preference to the machine-wide selection, preserves every configured argument as a separate invocation element, and appends the selected endpoint paths separately.
2. **Given** no selected-project tool selection and a machine-wide `~/.grip/config.toml` tool selection, **When** the operator runs `grip diff`, **Then** Grip uses the machine-wide selected tool for the project.
3. **Given** a selected-project configuration selects a named tool defined only in `~/.grip/config.toml`, **When** the operator runs `grip diff`, **Then** Grip uses the inherited tool definition.
4. **Given** a selected-project configuration supplies `args` for a machine-wide-defined named tool, **When** the operator runs `grip diff`, **Then** Grip uses only the selected-project array rather than appending it to the machine-wide array.
5. **Given** `GRIP_EXTERNAL_DIFF` names an executable, **When** the operator runs `grip diff`, **Then** Grip launches that executable in preference to both configuration layers, supplies only the two resolved endpoints as its arguments, and does not persist the override.
6. **Given** applicable tool settings are absent, malformed, empty, or name a program that cannot be launched, **When** the operator runs `grip diff`, **Then** Grip uses the built-in default when settings are absent, or gives a precise configuration or launch diagnostic when a selected tool is invalid, without altering payloads, mapping declarations, or accepted state.
7. **Given** a token resembles shell syntax, **When** Grip launches the configured program, **Then** the token is supplied literally and cannot execute another command, redirect input or output, expand variables, or interpolate endpoint paths.

---

### User Story 3 - Preserve automation and safety expectations (Priority: P3)

An operator or automation user can continue to obtain the existing structured inspection result and understands why an external comparison was not launched when the selected endpoints are not safe to compare.

**Why this priority**: External interactive tools must not break scripts or weaken Grip's established filesystem boundary.

**Independent Test**: Run JSON diff output and selected comparisons with a missing endpoint, unsupported node, or unsafe link condition; confirm no external program is launched and the existing safe diagnostic or structured result is retained.

**Acceptance Scenarios**:

1. **Given** an operator requests JSON output, **When** they run `grip diff`, **Then** Grip returns the existing structured inspection result and does not launch an external program.
2. **Given** either selected endpoint is absent, unsupported, or blocked by existing no-follow safety checks, **When** the operator runs human `grip diff`, **Then** Grip does not launch the configured program and reports the established detailed diagnostic.
3. **Given** a comparison program exits after comparing selected endpoints, **When** it returns any exit code, **Then** Grip reports the completed comparison and returns that exact exit code unchanged.
4. **Given** a comparison program is terminated by a signal, **When** Grip reports the interruption, **Then** it identifies the signal and returns `128 + signal number`.

### Edge Cases

- The configured executable is unavailable, not executable, or cannot start: Grip identifies the configured program and launch outcome without changing files, mapping intent, or accepted state.
- The configured program terminates abnormally: Grip identifies the terminating signal and returns `128 + signal number` rather than claiming a program-supplied exit result.
- The selected entry is a directory: Grip passes the two selected directory endpoints directly and does not enumerate unrelated destination content or create a staged copy.
- A selected entry has a missing endpoint, unsupported node, exact destination symlink obstacle, or unsafe symlink ancestor: Grip retains its existing read-only safety result and does not invoke the external program.
- An invocation token or endpoint path contains spaces, quotes, glob characters, variable-like text, or command separators: each remains one literal argument and cannot be interpreted as shell syntax.
- The operator supplies `--destination`: selector interpretation remains destination-space only; it does not change the direction, ownership, classification, or invocation safety contract.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: For human-readable `grip diff` output with a selected comparison whose endpoints are present, supported, and safe, Grip MUST invoke an external comparison program for the resolved source and destination endpoints.
- **FR-002**: Grip MUST resolve the comparison executable in this precedence order: `GRIP_EXTERNAL_DIFF`, the named tool selected by the selected project's `.grip/config.toml`, the named tool selected by the machine-wide `~/.grip/config.toml`, then `diff` with no configured arguments.
- **FR-003**: Grip MUST support an optional `[diff]` section that selects one named tool and an optional `[difftool.<name>]` section for each named tool. Each named tool MUST define one non-empty `program` reference and an ordered `args` array of literal argument tokens. Selected-project tool selection and definitions MUST coexist with mapping intent without changing its ownership, endpoint, or baseline semantics.
- **FR-004**: Grip MUST merge the machine-wide and selected-project configuration by key, with a selected-project key overriding a machine-wide key of the same name. A selected tool MAY obtain its final definition from either layer or both; after merging, it MUST have one non-empty `program` reference and an ordered `args` array.
- **FR-005**: When both configuration layers define `args` for the same named tool, Grip MUST use the selected-project array as the complete effective array and MUST NOT append machine-wide arguments.
- **FR-006**: Grip MUST invoke a selected named tool's `program` directly, preserve every `args` element as one literal argument, and supply the resolved source and destination endpoints as distinct arguments after those tokens; it MUST NOT evaluate a shell command string or interpolate values into one.
- **FR-007**: When `GRIP_EXTERNAL_DIFF` is set to a non-empty executable reference, Grip MUST bypass both configuration layers and invoke that executable with only the resolved source and destination endpoints as distinct arguments. Operators needing extra arguments MUST provide a wrapper executable rather than a command string.
- **FR-008**: `grip diff` MUST retain its current mapping selection, source-space and destination-space interpretation, project selection, classification, ownership, and no-follow filesystem safety behavior before any program launch.
- **FR-009**: For a selected file or directory comparison, Grip MUST pass the resolved selected endpoints directly and MUST NOT create, retain, or clean up temporary comparison copies.
- **FR-010**: If either endpoint is absent, unsupported, unsafe, or otherwise ineligible for a direct comparison, Grip MUST not launch an external program and MUST return the existing detailed read-only diagnostic.
- **FR-011**: If a selected-project or machine-wide tool selection or named-tool definition is invalid, `GRIP_EXTERNAL_DIFF` is empty, or the selected executable cannot be launched, Grip MUST give a precise diagnostic that identifies the configuration or program problem and MUST leave payloads, mappings, and accepted state unchanged.
- **FR-012**: After a configured, default, or environment-overridden comparison program starts and returns an exit code, Grip MUST return that exact exit code unchanged and present the completed comparison deterministically in human output.
- **FR-013**: If a comparison program is terminated by a signal, Grip MUST identify that signal in its diagnostic and return `128 + signal number`.
- **FR-014**: `grip --output json diff` MUST retain the existing structured read-only inspection contract and MUST NOT launch an external program.
- **FR-015**: This feature MUST NOT change mapping ownership, endpoint classification, baseline semantics, mutation commands, JSON schemas or values, automatic tool installation, network access, or shell execution.
- **FR-016**: Human-readable `grip diff` MUST launch an external program only when the operator supplies a selector that resolves to one exact managed comparison. Without a selector, it MUST retain the existing broad read-only inspection result and MUST NOT launch an external program.

## Key Entities *(include if feature involves data)*

- **Named diff tool**: A configuration-defined identity with one direct executable reference and ordered literal argument tokens.
- **Diff tool selection**: An optional `[diff]` setting in the machine-wide or selected-project configuration that names the tool used for `grip diff`.
- **External-diff override**: The non-persistent `GRIP_EXTERNAL_DIFF` executable reference that bypasses named-tool configuration and receives only Grip's resolved source and destination endpoints.
- **Comparison handoff**: One direct launch request composed from a resolved managed source endpoint, its resolved destination endpoint, and an applicable named tool or external-diff override.
- **Comparison result**: The user-visible outcome that distinguishes a completed comparison and its returned exit code, a signal-derived `128 + signal number` result, and a Grip-owned launch diagnostic.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In 100% of isolated file and directory comparison acceptance scenarios with safe present endpoints, `grip diff` selects `GRIP_EXTERNAL_DIFF` over named-tool configuration and supplies only its two resolved endpoints, otherwise resolves the selected tool from key-by-key merged selected-project and machine-wide configuration, or uses the `diff` fallback when no tool is selected, and invokes a selected named tool with exactly its literal `args` followed by the two resolved endpoints.
- **SC-002**: In 100% of acceptance scenarios containing shell-significant configured tokens or endpoint paths, no extra command, expansion, redirection, or substitution occurs.
- **SC-003**: In 100% of JSON diff acceptance scenarios, no external program is launched and the structured inspection result remains unchanged.
- **SC-004**: In 100% of missing-endpoint, unsupported-node, and unsafe-link acceptance scenarios, no external program is launched and no payload, mapping, or accepted-state change occurs.
- **SC-005**: In 100% of default and configured comparison acceptance scenarios where the program returns an exit code, Grip returns that exact code; in 100% of signal-termination scenarios, Grip identifies the signal and returns `128 + signal number`; unavailable-program scenarios receive deterministic Grip-owned diagnostics.
- **SC-006**: In 100% of unselected human `grip diff` acceptance scenarios, Grip launches no external program and retains the broad read-only inspection result.

## Assumptions

- Configuration follows Git-like precedence: Grip merges machine-wide configuration first and selected-project configuration second, with each selected-project key overriding the matching machine-wide key; therefore a project may select a machine-wide-defined tool or override individual selected-tool settings. A selected-project `args` key replaces the corresponding machine-wide array, rather than appending to it. The executable-only `GRIP_EXTERNAL_DIFF` override is more specific than either configuration layer. The environment override never persists, and all sources are optional so existing projects retain the `diff` fallback without migration work by operators. A project selection may name a program unavailable on another machine; Grip reports that condition precisely rather than silently choosing a lower-precedence tool.
- Comparison programs own the meaning of their exit codes. Grip transparently returns a numeric exit code from a program that starts and exits normally. A launch failure remains a Grip-owned diagnostic; signal termination returns the conventional `128 + signal number` and identifies the signal.
- The feature intentionally keeps JSON output non-interactive and unchanged, preserving existing automation rather than adding an external-process result to a stable structured interface.
- Direct endpoint handoff is sufficient for the initially supported file and directory comparisons. Missing, unsupported, or unsafe endpoints remain inspection diagnostics rather than candidates for temporary copies or implicit materialization.
- An external comparison handoff is intentionally exact-entry only. An unselected human inspection remains broad and read-only so it cannot unexpectedly launch multiple external programs.
- Existing verified command hierarchy, project selection, baseline, mapping, filesystem-support, and no-follow safety contracts remain authoritative except where this specification explicitly adds the external comparison handoff.
