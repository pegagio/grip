# Feature Specification: Destination Path Forms

**Feature Branch**: `012-destination-path-forms`

**Created**: 2026-09-10

**Status**: Complete

**Input**: User description: "Allow `grip add` destinations to be absolute paths or non-normalized `~/` paths. Reject relative destinations. Keep source paths relative to the Grip project root."

## Clarifications

### Session 2026-09-10

- Q: Should a destination consisting of standalone `~` remain a valid form? → A: Keep standalone `~` alongside absolute and `~/…` destinations.
- Q: How should Grip store an accepted destination whose input spelling is non-normalized? → A: Store the accepted destination spelling unchanged; resolve it when validating or using the mapping.
- Q: Should Feature 012 update the product definition to state that destinations may be absolute, `~`, or `~/…`, while sources remain project-relative? → A: Update the product definition as part of Feature 012.
- Q: Which non-normalized `~/` spellings should Grip accept? → A: Accept every `~/`-prefixed filesystem path spelling, including `.`, `..`, and repeated separators.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Add a user-relative destination (Priority: P1)

An operator adds a project file to a destination under their home directory using a `~/` path and can use that mapping without the command rejecting the destination for its spelling.

**Why this priority**: A common home-relative deployment target must work with the command form the operator naturally supplies.

**Independent Test**: In an initialized project, add an existing project file to a valid `~/Working/grip-dst/README.md` destination and confirm the mapping is recorded without a destination-normalization error.

**Acceptance Scenarios**:

1. **Given** an initialized Grip project and an eligible source file, **When** the operator runs `grip add README.md ~/Working/grip-dst/README.md`, **Then** Grip accepts and records the mapping.
2. **Given** an initialized Grip project and an eligible source file, **When** the operator supplies a `~/` destination containing `.`, `..`, or repeated separators, **Then** Grip accepts and preserves that spelling rather than requiring normalization before acceptance.

---

### User Story 2 - Add an absolute destination (Priority: P1)

An operator adds a project file to a specific absolute filesystem location.

**Why this priority**: Operators need to map project content to locations outside their home-relative namespace.

**Independent Test**: In an initialized project, add an eligible source file to an absolute destination and confirm the mapping is recorded.

**Acceptance Scenarios**:

1. **Given** an initialized Grip project and an eligible source file, **When** the operator runs `grip add README.md /opt/grip-dst/README.md`, **Then** Grip accepts and records the mapping without the prior destination-form error.

---

### User Story 3 - Reject an ambiguous destination (Priority: P2)

An operator receives a clear error when attempting to add a destination relative to the current directory or project root.

**Why this priority**: Rejecting relative destinations prevents a mapping from silently changing meaning when run from another directory or checkout.

**Independent Test**: Attempt to add an eligible source file with `grip add README.md destination/README.md` and confirm that no mapping is created.

**Acceptance Scenarios**:

1. **Given** an initialized Grip project and an eligible source file, **When** the operator supplies a relative destination, **Then** Grip rejects it, explains that a destination must be absolute or home-relative, and records no mapping.
2. **Given** an initialized Grip project, **When** an accepted source path is relative, **Then** Grip retains the source path relative to the project root rather than treating it as a destination-form error.

### Edge Cases

- A destination consisting of `~` continues to be treated as the operator's home directory if that form was already accepted.
- Every `~/`-prefixed filesystem spelling is accepted, including `.`, `..`, and repeated separators; it may resolve outside the home directory, and acceptance does not depend on caller normalization.
- A path beginning with `~` but not `~` or `~/` is rejected as neither an absolute nor a home-relative destination.
- An absolute destination remains absolute in declared mapping intent and may not be portable to another machine; Grip reports ordinary path, ownership, or topology failures separately from destination-form validation.
- The feature does not alter existing source-path validation, mapping overlap checks, endpoint-kind checks, or baseline behavior.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Grip MUST accept an absolute destination path for `grip add`.
- **FR-002**: Grip MUST accept a destination expressed as `~` or beginning with `~/`, including `~/` spellings containing `.`, `..`, or repeated separators.
- **FR-003**: Grip MUST preserve the accepted destination spelling unchanged in mapping intent and MUST NOT reject an otherwise valid accepted destination solely because it requires normalization during resolution.
- **FR-004**: Grip MUST reject every destination that is neither absolute nor `~` nor `~/`-prefixed, including a path relative to the current directory or project root.
- **FR-005**: When rejecting a relative or malformed destination form, Grip MUST emit a diagnostic that distinguishes the allowed destination forms from source-path rules and MUST NOT create or modify a mapping.
- **FR-006**: Grip MUST preserve project-relative source-path semantics: relative source inputs are resolved against and persisted relative to the Grip project root.
- **FR-007**: When Grip later validates or uses a mapping, it MUST resolve the preserved destination spelling for that operation while retaining the original declared spelling for display and persistence.
- **FR-008**: Grip MUST apply existing mapping-ownership, endpoint-kind, and topology validation after destination-form acceptance; this feature does not weaken those checks.
- **FR-009**: Grip MUST update the product definition so its mapping model states that destinations may be absolute, `~`, or `~/`-prefixed, while sources remain project-relative.
- **FR-010**: Grip MUST treat a `~/` destination that resolves outside the home directory as an accepted destination form and apply ordinary destination validation to the resolved path.

### Key Entities *(include if feature involves data)*

- **Mapping declaration**: The user-authored association between a project-relative source and a destination expressed in an accepted absolute or home-relative form.
- **Destination form**: The declared category of destination input: absolute, home directory, or home-relative; relative destinations are invalid.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In isolated filesystem tests, 100% of valid absolute and `~/` destination examples are accepted without a destination-normalization error.
- **SC-002**: In isolated filesystem tests, 100% of relative destination examples are rejected before any mapping declaration changes.
- **SC-003**: Operators can complete each supported `grip add SOURCE DESTINATION` path-form workflow in one command without manually converting a `~/` destination to another spelling.
- **SC-004**: Existing project-relative source-path scenarios continue to pass unchanged while destination-form coverage is added.

## Assumptions

- The already accepted `~` destination remains valid; this feature broadens accepted forms and does not remove a supported form.
- Grip resolves accepted destination forms only for validation and operation; the persisted declaration remains the accepted input spelling and does not require pre-normalization.
- Absolute destination declarations are intentionally machine-specific and may not remain usable after a project is moved to a machine without the same absolute path.
- The feature inherits Feature 011's flat `add SOURCE DESTINATION` command and Feature 010's project-scoped source semantics.
- The product definition is an affected durable contract and must be made consistent with the Feature 012 destination forms.
- A `~/` spelling may resolve outside the home directory; this is permitted because the destination form is explicit and existing ownership and topology checks continue to apply.
- Wiki pages document the prior command and mapping model but do not yet document Feature 012; the authoritative feature scope is the roadmap and direct active-user decision.
