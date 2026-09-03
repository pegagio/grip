# Feature Specification: CLI, Configuration, and State Foundation

**Git Branch**: `develop`

**Feature Identifier**: `001-cli-state-foundation`

**Created**: 2026-09-03

**Status**: Draft

**Input**: User description: "Use roadmap entry 001 and its wiki-backed governing decisions and constraints to specify Grip's CLI, configuration, and state foundation."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Invoke Grip Predictably (Priority: P1)

As a local user, I can invoke `grip` and view help or version information without first creating a Grip home, registry, or state.

**Why this priority**: A predictable, non-mutating command boundary is the smallest useful foundation for every later Grip capability.

**Independent Test**: Run the executable's help flag, version flag, and human-readable `version` command in an isolated environment with no Grip home and verify deterministic text, exit status `0`, and zero filesystem changes.

**Acceptance Scenarios**:

1. **Given** no Grip home or registry exists, **When** the user invokes `--help` or `--version`, **Then** Grip returns conventional text and exits successfully without resolving or creating a Grip home.
2. **Given** no Grip home or registry exists, **When** the user invokes the human-readable `version` command, **Then** Grip reports its application version and exits successfully without creating or changing any filesystem entry.

---

### User Story 2 - Validate Configuration and State Safely (Priority: P2)

As a local user, I can validate a minimal versioned registry and Grip-owned state so that malformed, unknown, incompatible, or corrupt data is rejected before later features depend on it.

**Why this priority**: Later mapping and synchronization behavior needs a trustworthy boundary between user intent and operational evidence.

**Independent Test**: Place valid and invalid registry and state documents in isolated Grip homes, run the foundation validation command, and verify the reported result without allowing any mapped-payload mutation.

**Acceptance Scenarios**:

1. **Given** no `GRIP_HOME` override, **When** the user invokes `validate`, **Then** Grip selects `~/.grip/` as the single root for its registry and state.
2. **Given** `GRIP_HOME` contains a non-empty absolute path, **When** the user invokes `validate`, **Then** Grip uses that exact path without appending another directory component.
3. **Given** `GRIP_HOME` is empty or relative, **When** the user invokes `validate`, **Then** Grip reports `invalid_configuration`, exits with code `10`, creates nothing, and does not fall back to another root.
4. **Given** the selected Grip home or its required `config.toml` does not exist, **When** validation runs, **Then** Grip reports `invalid_configuration`, exits with code `10`, and creates neither the missing root nor document.
5. **Given** a supported minimal registry and no existing machine state, **When** the user validates the Grip home, **Then** Grip reports the registry as valid and the absent state as an uninitialized condition rather than corruption.
6. **Given** a registry with an unsupported schema version, unknown field, malformed value, or path escaping the Grip home, **When** validation runs, **Then** Grip identifies the offending document and category without changing it.
7. **Given** state with an unsupported schema version, failed integrity check, or malformed content, **When** validation runs, **Then** Grip reports incompatible or corrupt state distinctly from invalid user configuration.
8. **Given** a valid empty registry, **When** validation runs, **Then** no mapping, traversal, baseline, copying, or synchronization behavior is implied or performed.
9. **Given** accepted state is being replaced, **When** publication reaches the replacement boundary, **Then** the previous accepted generation already exists as a verified recovery copy; an initial publication requires no recovery copy.
10. **Given** Grip creates machine-owned state or recovery nodes on a supported target, **When** creation completes under any permitted process umask, **Then** directories have mode `0700`, files have mode `0600`, and a pre-existing user-authored Grip home retains its original permissions.

---

### User Story 3 - Consume Deterministic Results (Priority: P3)

As a user or automation author, I can choose human-readable or machine-readable results and receive stable error categories, exit codes, and diagnostic separation.

**Why this priority**: Stable result boundaries let later commands add domain behavior without forcing automation to parse prose or diagnostics.

**Independent Test**: Exercise every foundation success and failure category in both output modes and verify equivalent meaning, stable field names, documented exit codes, and diagnostics that never contaminate machine-readable output.

**Acceptance Scenarios**:

1. **Given** a successful application command in machine-readable mode, **When** Grip responds, **Then** it emits exactly one valid result document with the required envelope fields and exits with code `0`.
2. **Given** invalid command usage, invalid configuration, incompatible or corrupt state, or an operational failure, **When** Grip responds, **Then** it uses the documented category and exit code in both human and machine-readable modes.
3. **Given** diagnostics are enabled, **When** Grip emits diagnostic detail, **Then** human or machine result content remains independently consumable and no sensitive document content is exposed by default.

### Edge Cases

- The user's home directory cannot be determined while `GRIP_HOME` is unset.
- The selected Grip home exists as a regular file, symbolic link, or inaccessible directory rather than an accessible directory owned by the invoking user.
- The registry exists but the machine-owned state area does not, or the reverse.
- The selected Grip home or required registry document does not exist when validation runs.
- A document is syntactically valid but has a missing, non-integer, older unsupported, or newer unsupported schema version.
- A state publication is interrupted before the replacement becomes visible.
- A recovery copy already exists for the accepted generation when publication is retried.
- Two Grip processes attempt to publish Grip-owned state concurrently.
- Human or machine-readable output is written to a closed or failing output destination.
- Diagnostic detail contains a path or value that must be represented without corrupting the selected result format.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The project and executable identity MUST be `grip`; user-facing product prose MAY use the title `Grip`.
- **FR-002**: Grip MUST provide non-mutating `--help` and `--version` text displays that succeed independently of whether a Grip home or registry exists, plus a `version` command that supports both result modes.
- **FR-003**: Grip MUST use `~/.grip/` as the default per-user root for registry and state.
- **FR-004**: A non-empty absolute `GRIP_HOME` MUST select the exact alternate root for both registry and state; an empty, relative, or otherwise invalid value MUST produce an error without fallback.
- **FR-005**: Grip MUST NOT implicitly read, merge, or write `/etc/grip/` or any other machine-wide registry or state location.
- **FR-006**: Grip MUST keep user-authored registry data logically and physically distinct from machine-owned operational state under the selected Grip home.
- **FR-007**: The user-authored registry MUST use a documented, human-editable TOML document with an explicit positive integer schema version and MUST support a valid empty mapping collection.
- **FR-008**: Machine-owned state MUST use versioned JSON documents with an explicit positive integer schema version and integrity evidence sufficient to detect incomplete or corrupt publication.
- **FR-009**: Configuration and state readers MUST reject unknown fields, malformed values, unsupported schema versions, and paths that escape their allowed Grip-owned locations; readers MUST distinguish invalid user configuration from incompatible or corrupt machine state.
- **FR-010**: An absent machine-state document MUST be treated as uninitialized state when the registry is otherwise valid; Grip MUST NOT guess, reconstruct, or publish synchronization evidence in this feature.
- **FR-011**: Any creation or replacement of Grip-owned state MUST be staged and atomically published where the selected filesystem supports atomic replacement. Before replacing accepted machine state, Grip MUST retain and verify the previous accepted state in a Grip-owned recovery namespace; initial publication requires no recovery copy. Recovery inspection, restoration, and cleanup commands remain outside Feature 001. An unsuccessful publication MUST leave the prior accepted document intact or report precisely that no accepted document exists.
- **FR-012**: Grip MAY use only short-lived coordination around Grip-owned state publication; contention MUST fail with an actionable error, and this feature MUST NOT introduce locks over user payload trees.
- **FR-013**: Grip MUST offer separate human-readable and machine-readable modes for application commands. Machine-readable results MUST be JSON documents with stable top-level fields: `schema_version`, `status`, `code`, `message`, and `details`; parser metadata displays produced by `--help` and `--version` remain conventional text outside this envelope.
- **FR-014**: Public Feature 001 commands MUST use these exit codes: `0` success, `2` invalid command usage, `10` invalid user configuration, `11` unsupported configuration or state schema, `12` corrupt or internally inconsistent machine state, and `20` other operational failure.
- **FR-015**: Machine-readable `status` MUST be either `ok` or `error`; `code` MUST be a stable symbolic category corresponding to the process exit code; `message` MUST be a concise summary; and `details` MUST be an object that may add category-specific fields without changing the meaning of existing fields.
- **FR-016**: Diagnostic output MUST remain separate from human and machine result output, MUST be disabled by default, and MUST avoid exposing registry or state document contents unless the user explicitly requests diagnostic detail.
- **FR-017**: All automated tests and acceptance-validation procedures for this feature MUST operate within isolated temporary roots and MUST NOT inspect or mutate the developer's real files, default Grip home, mapped payloads, version-control state, or privileged locations. The production `validate` command MUST inspect only the Grip home selected by the user.
- **FR-018**: This feature MUST NOT add mapping lifecycle, source traversal, ignore processing, baseline capture, payload copying, synchronization, deletion, remote access, background services, caching, persistent indexing, or parallel payload execution.
- **FR-019**: CLI parsing and presentation MUST remain behaviorally separate from Grip-owned configuration and state validation so the same domain result is representable in human and machine-readable forms.
- **FR-020**: Foundation behavior MUST be deterministic: equivalent inputs and filesystem evidence MUST produce equivalent result categories, exit codes, and machine-readable fields.
- **FR-021**: Grip MUST provide a read-only `validate` command that validates the selected Grip home, registry, and state without creating missing documents or directories. An absent selected Grip home or required `config.toml` MUST produce `invalid_configuration` and exit `10`.
- **FR-022**: On supported macOS and Unix targets, Grip MUST create machine-owned state and recovery directories with mode `0700` and state, lock, staging, and recovery files with mode `0600`. Grip MUST validate the ownership, node type, symbolic-link status, and accessibility of existing Grip-owned nodes. Unsafe existing nodes MUST be rejected without following them or silently changing their permissions. Grip MUST NOT silently change permissions on a pre-existing user-authored Grip home.

### Key Entities

- **Grip Home**: The exact per-user root selected by the default rule or `GRIP_HOME`; it contains distinct registry and state areas and is not a mapped payload root merely by existing.
- **Registry Document**: Human-authored, versioned mapping intent. In this feature it may contain an empty mapping collection but does not yet create, validate, or mutate mappings.
- **State Document**: Grip-owned, versioned operational evidence with integrity information. In this feature it proves safe publication and validation boundaries but contains no synchronization baseline.
- **Recovery Copy**: A verified, immutable copy of the previously accepted state retained by generation before replacement; Feature 001 creates and validates it but does not expose restoration or cleanup commands.
- **Result Envelope**: A stable machine-readable description of one command outcome, including version, status, symbolic category, message, and structured details.
- **Diagnostic Event**: Optional troubleshooting information emitted separately from the command result and subject to safe disclosure defaults.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In acceptance testing, 100% of default, valid override, empty override, relative override, unavailable-home, and wrong-node-type cases select the documented root or fail with the documented category without fallback.
- **SC-002**: A test suite covering every supported and unsupported registry and state version, malformed document class, unknown field, and integrity failure produces the expected distinct result in 100% of cases.
- **SC-003**: Every application-command success and failure scenario requested in machine-readable mode produces valid output with all five required top-level fields and the documented exit code in 100% of automated contract tests; `--help` and `--version` are separately verified as conventional text displays.
- **SC-004**: Interrupted or contended state-publication tests never expose a partially written document as accepted state, never replace accepted state without first retaining its verified prior generation, and never alter mapped payloads.
- **SC-005**: Help, version, validation, and failed-operation tests cause zero changes outside their isolated Grip home and zero payload, backup, baseline, or version-control changes.
- **SC-006**: On a representative local workstation, help and version results complete within 100 milliseconds and validation of a minimal Grip home completes within 250 milliseconds in at least 95 of 100 consecutive warm runs.
- **SC-007**: A user following the documented setup example can select an alternate Grip home and validate a minimal registry successfully on the first attempt without needing knowledge of machine-owned state internals.

## Assumptions

- The roadmap entry is eligible because it is `planned` and has no dependencies.
- `develop` is the working integration branch for this draft; naming the durable merge-freeze branch remains a project governance question outside Feature 001.
- TOML is selected for the human-authored registry because it provides a concise, inspectable configuration surface; JSON is selected for machine-owned state and automation output because it provides an unambiguous, broadly consumable data contract.
- Schema version `1` is the first supported registry, state, and result-envelope version; exact filenames and internal state layout remain planning decisions so long as the logical separation and external contracts above hold.
- Feature 001 establishes only the foundation portion of the automation contract. Exit `13` and symbolic code `state_contention` are reserved for the first public command that publishes Grip-owned state; Feature 001 uses the condition only as an internal publication error. Later features may add categories and exits for drift, conflict, unsupported payload entries, and synchronization outcomes without changing this feature's public meanings.
