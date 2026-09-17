# Feature Specification: Destination Adoption

**Feature Branch**: `036-destination-adoption`

**Created**: 2026-09-17

**Status**: Complete

**Input**: User description: "Add `grip pull -a|--adopt DESTINATION` so an operator can explicitly copy one destination-only file under an existing tree mapping into its corresponding source path and establish its first accepted baseline without declaring a separate mapping."

## Clarifications

### Session 2026-09-17

- Q: When the paired source parent directories are absent, should adoption also enroll the required destination parent directories as matching managed directories? → A: Adoption requires an existing source-side ancestor directory, which may be a direct parent, grandparent, or more distant ancestor. Grip may create only intervening source directories; if no source-side ancestor exists, the command fails.
- Q: Should adoption refuse a destination file when its paired source path is excluded by applicable `.gripignore` rules? → A: Reject ordinary adoption, but let `--force` explicitly override the ignore rule for that exact adoption only.
- Q: Should a forced adoption of an ignored path complete even though ordinary discovery will later ignore it? → A: Yes. Complete the one-time forced adoption, leave `.gripignore` unchanged, and print the exact recommended negation rule set with a concise explanation that a later ordinary operation will ignore the entry unless the user adds it.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Adopt an existing destination file (Priority: P1)

An operator maintains a tree mapping and discovers an existing destination file that should become source-defined managed content. They explicitly adopt that exact destination file instead of creating a conflicting nested file mapping or using a separate copy command.

**Why this priority**: It makes a common dotfile enrollment workflow safe, explicit, and fully managed by Grip.

**Independent Test**: With an existing tree mapping and a destination-only regular file beneath its destination root, run `grip pull --adopt DESTINATION` and verify that the paired source file is created, the two complete managed states are equal, and status reports the entry as current.

**Acceptance Scenarios**:

1. **Given** an existing tree mapping, an absent paired source file, and an eligible destination regular file below the mapped destination root, **When** the operator runs `grip pull --adopt DESTINATION`, **Then** Grip creates the paired source file, preserves the destination file, and publishes accepted baselines for the file and any newly created required ancestor directories.
2. **Given** the same eligible destination file, **When** the operator runs `grip pull -a DESTINATION`, **Then** the result is identical to the long-option form.
3. **Given** an eligible destination file, **When** the operator runs `grip pull --adopt --dry-run DESTINATION`, **Then** Grip reports the planned adoption without changing either endpoint or accepted state.
4. **Given** an otherwise eligible destination file whose paired source path is ignored, **When** the operator runs `grip pull --adopt --force DESTINATION`, **Then** Grip adopts that exact file without changing the ignore policy.
5. **Given** a forced adoption of an ignored paired source path, **When** Grip completes the adoption, **Then** it prints the exact `.gripignore` negation rule set and placement needed to retain membership and explains that a later ordinary operation will ignore the entry unless the operator adds it.

---

### User Story 2 - Preserve source-defined ownership (Priority: P2)

An operator can distinguish explicit adoption from ordinary synchronization and cannot accidentally cause Grip to inventory or manage unrelated destination files.

**Why this priority**: Tree mappings remain predictable only when destination discovery is limited to deliberate, exact requests.

**Independent Test**: Run ordinary `pull`, `status`, and `diff` against a tree mapping containing destination-only files and verify none are copied or baselined; then adopt one exact requested file and verify no sibling becomes managed.

**Acceptance Scenarios**:

1. **Given** destination-only files under a tree mapping, **When** the operator runs ordinary `grip pull`, **Then** no destination-only file is adopted.
2. **Given** two eligible destination-only sibling files, **When** the operator adopts one exact destination path, **Then** only that file receives a source counterpart and accepted baseline.

---

### User Story 3 - Receive safe adoption diagnostics (Priority: P3)

An operator receives a specific error rather than a partial copy when the requested path cannot safely become a tree member.

**Why this priority**: Adoption crosses the normal source-discovery boundary and must retain Grip's ownership and metadata safeguards.

**Independent Test**: Attempt adoption for each unsupported or ambiguous state and verify neither payload nor accepted state changes.

**Acceptance Scenarios**:

1. **Given** an existing paired source entry, **When** the operator requests adoption, **Then** Grip refuses the request and identifies that the entry is already source-defined.
2. **Given** a destination path outside a tree mapping, an exact-file mapping, or a mapping-reserved destination leaf, **When** the operator requests adoption, **Then** Grip refuses without changing state.
3. **Given** an unsupported destination node or incompatible managed metadata, **When** the operator requests adoption, **Then** Grip reports the exact safety reason and leaves both endpoints unchanged.

### Edge Cases

- A destination selector naming a mapped tree root, a directory, or a path outside the mapped destination root is rejected; initial delivery adopts one exact regular file only.
- Adoption requires an existing source-side ancestor directory; it may be a direct parent, grandparent, or more distant ancestor. Grip may create only intervening directories inside the mapped source tree, and fails if no such ancestor exists.
- `--adopt` is incompatible with `--source`. With adoption, `--force` overrides only applicable ignore rules for the selected exact path; it does not select a conflict winner or bypass any other safety check.
- Forced adoption never changes `.gripignore`. Its accepted baseline is intentionally temporary while the path remains ignored; a later ordinary discovery pass may remove that ignored entry from active membership.
- If the source, destination, mapping declaration, or accepted state changes after planning, Grip stops before publication and does not publish a baseline.
- A failed copy, verification, or baseline publication leaves the destination unchanged and does not represent the entry as adopted.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: `grip pull` MUST accept `-a` and `--adopt` as equivalent explicit adoption options.
- **FR-002**: Adoption MUST accept exactly one destination-space path that resolves to an existing regular file beneath an existing tree mapping's destination root.
- **FR-003**: For an eligible adoption, Grip MUST require an existing source-side ancestor directory, which may be any ancestor of the paired source path. Grip MAY create only the intervening source directories and paired source file within the mapped source tree, while leaving the destination file unchanged.
- **FR-004**: Before reporting success, Grip MUST verify equality of the adopted file's supported managed state at both endpoints and publish accepted baselines for that file and any newly created required ancestor directories.
- **FR-005**: Adoption MUST be available in dry-run form and dry-run MUST not create source paths, change destination state, or publish accepted evidence.
- **FR-006**: Ordinary `pull`, `status`, and `diff` MUST continue to ignore destination-only tree members unless the operator explicitly invokes adoption for one exact path.
- **FR-007**: Ordinary adoption MUST reject a path excluded by applicable `.gripignore` rules. `grip pull --adopt --force DESTINATION` MUST override only those ignore rules for the selected exact adoption and MUST NOT modify the ignore policy.
- **FR-008**: After successful forced adoption of an ignored path, human output MUST print the exact `.gripignore` negation rule set and policy-file placement recommended to retain membership, including any required ancestor exemptions, and concisely explain that a later ordinary operation will ignore the entry unless the operator adds it. JSON output MUST expose the same warning, placement, and recommended rules without changing the ignore policy.
- **FR-009**: Adoption MUST reject an existing paired source entry, directory or non-regular destination node, unsupported metadata, ownership conflict, destination outside a tree mapping, stale evidence, and incompatible option combinations without mutating payload or accepted state. `--force` MUST NOT bypass these checks.
- **FR-010**: Human and JSON results for adoption MUST identify the exact selected destination and paired source and distinguish successful adoption, dry-run, no-op, and blocked outcomes.

### Key Entities *(include if feature involves data)*

- **Adoption request**: One explicit destination-space selector and its dry-run state.
- **Adopted tree member**: The exact newly source-defined regular file and its paired destination file, with accepted baseline evidence. Required newly created ancestor directories are structural supporting members and are baselined with the file.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: An operator can enroll one eligible existing destination file into a tree mapping with one `grip pull --adopt DESTINATION` command and no separate mapping declaration or external copy command.
- **SC-002**: In automated isolated-filesystem coverage, successful adoption leaves the destination byte-for-byte unchanged and produces an equivalent paired source file plus accepted baseline entries for the file and any newly created required ancestor directories.
- **SC-003**: In automated isolated-filesystem coverage, every rejected or dry-run adoption leaves both endpoint payloads and accepted state unchanged.
- **SC-004**: In automated isolated-filesystem coverage, adopting one destination-only file does not cause any unselected destination sibling to become managed.

## Assumptions

- Initial delivery is intentionally limited to one exact regular file under an already-declared tree mapping; directory and bulk adoption are separate future decisions.
- The destination is authoritative only for the explicitly requested previously destination-only member; adoption does not alter ordinary pull direction or conflict-resolution semantics. In adoption mode, `--force` is an exact-path ignore-policy override rather than a conflict-winner selection.
- A forced adoption of an ignored path is a one-time import. Grip does not edit `.gripignore`; the operator must add the printed precise negation rule set at the reported policy-file placement to retain the adopted entry through later ordinary discovery.
- Existing managed metadata compatibility, no-follow, revalidation, staging, verification, and accepted-state publication safeguards remain applicable.
- This feature flows forward from the source-defined tree-membership model and Feature 035's destination-space pull selector contract.

## Governance

This feature is governed by Constitution Principles II (explicit ownership), III (validated, verified mutation), IV (bounded revalidation), V (isolated filesystem testing), and the merge-bounded flow-back model. The explicit exact selector is the narrow exception that permits discovery of one otherwise destination-only tree member; it does not authorize ambient destination traversal or ownership.
