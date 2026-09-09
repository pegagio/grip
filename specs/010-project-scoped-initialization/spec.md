# Feature Specification: Project-Scoped Initialization and Portable Mappings

**Feature Branch**: `specs/010-project-scoped-initialization`

**Created**: 2026-09-09

**Status**: Draft

**Input**: User description: "Replace the global Grip mapping pattern with initialized Grip projects. Add `grip init [PATH]`, discover a project by walking upward or select it explicitly, scope every project-dependent command to that project, store portable mapping intent in the project for version control, and keep machine-local operational state separate. Backward compatibility is not required before release."

## Clarifications

### Session 2026-09-09

- Q: How should Grip distinguish machine-local state for separate clones and for a newly initialized project that reuses an old filesystem location? → A: Scope state to the selected project's `.grip/state/` directory; do not use a generated project ID.
- Q: Must `--project PATH` name the exact Grip project root, or may it name any directory inside the project? → A: It must name the exact Grip project root.
- Q: Should Feature 010 use `.grip/config.toml` for committed mappings and `.grip/state/` for uncommitted local state, with no generated project ID? → A: Yes, and `grip init` must also create `.grip/.gitignore` to exclude `/state/` from Git while leaving `config.toml` trackable.
- Q: What should Grip do when a project is moved or copied through the filesystem together with its `.grip/state/` directory? → A: Retain the copied state, but completely rebind and revalidate it before any acceptance or mutation.
- Q: How should portable destination paths be written in `.grip/config.toml`? → A: Use literal `~` for the home root and `~/path` for descendants.
- Q: How should a project-root source and the Grip-owned `.grip/.gitignore` be treated? → A: Permit exact `.` only for a tree mapping, always exclude `.grip/` structurally, and require the Grip-owned `.gitignore` to contain only the canonical `/state/` rule.
- Q: How should invalid inner project metadata and `--project` on project-independent commands behave? → A: Treat any enclosing `.grip` as a candidate boundary that cannot be silently crossed when invalid, and reject `--project` with `init` or `version` as invalid usage.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Initialize a Portable Grip Project (Priority: P1)

A user initializes a directory as a Grip project so that the directory becomes the source root for portable mapping intent. The `.grip/config.toml` descriptor and `.grip/.gitignore` can be committed to version control without committing the machine-local `.grip/state/` contents.

**Why this priority**: Initialization establishes the product boundary that every mapping and later command depends on. Without it, Grip remains a global per-user registry rather than a portable source project.

**Independent Test**: Initialize an empty temporary directory, inspect the created `.grip/` metadata directory, and verify that `config.toml` contains valid empty portable mapping intent, `.gitignore` excludes `/state/`, no machine-local state or payload mutation is created, and the command reports the project root.

**Acceptance Scenarios**:

1. **Given** an existing ordinary directory that is not within another Grip project, **When** the user runs `grip init` from that directory, **Then** Grip creates `.grip/config.toml` and `.grip/.gitignore`, initializes that directory as a Grip project, and reports the canonical project root.
2. **Given** an existing ordinary directory outside any Grip project, **When** the user runs `grip init PATH`, **Then** Grip initializes the selected directory regardless of the invocation directory.
3. **Given** an already initialized directory with a valid unchanged project descriptor, **When** the user initializes that same directory again, **Then** Grip reports that the project is already initialized and performs an exact no-op.
4. **Given** a proposed project directory with incompatible or unsafe `.grip/` metadata or an enclosing Grip project, **When** initialization is requested, **Then** Grip fails without replacing existing nodes or creating machine-local state.
5. **Given** successful initialization, **When** the directory is inspected afterward, **Then** no Git repository, payload mapping, baseline, recovery record, or synchronization action has been created implicitly.
6. **Given** an existing `.grip/.gitignore` with missing, additional, reordered, or contradictory content, **When** initialization is requested, **Then** Grip treats the metadata as non-equivalent and fails without rewriting the file.

---

### User Story 2 - Resolve and Scope Every Command to One Project (Priority: P1)

A user runs Grip from a project root or descendant and has the command apply only to that enclosing Grip project. The user can instead name a project explicitly when invoking Grip from elsewhere.

**Why this priority**: Reliable project resolution prevents commands from reading or mutating an unrelated project and replaces the former global command scope.

**Independent Test**: Create two isolated initialized projects, invoke read-only and dry-run commands from their roots, descendants, and an unrelated directory, and verify that each command selects only the intended `.grip/` metadata directory or fails before accessing mappings or state.

**Acceptance Scenarios**:

1. **Given** exactly one valid Grip project in the invocation directory's ancestor chain, **When** a project-dependent command omits `--project`, **Then** Grip discovers and scopes the command to that project.
2. **Given** a valid initialized project, **When** a project-dependent command uses `--project PATH`, **Then** the explicit project is selected regardless of the invocation directory.
3. **Given** no valid project in the invocation directory's ancestor chain and no explicit project, **When** a project-dependent command runs, **Then** Grip fails before reading mappings, machine-local state, or payload paths.
4. **Given** more than one valid enclosing `.grip/config.toml`, **When** a project-dependent command relies on implicit discovery, **Then** Grip reports the ambiguity and fails without selecting either project.
5. **Given** an explicit project path that is missing, uninitialized, invalid, or not the descriptor's containing root, **When** a project-dependent command runs, **Then** Grip fails without falling back to implicit discovery or a global registry.
6. **Given** `grip version`, `grip --help`, or command help, **When** it runs outside a Grip project, **Then** it succeeds without project discovery because it does not operate on project data.
7. **Given** `--project PATH` with `grip init` or `grip version`, **When** Grip parses the command, **Then** it reports invalid usage without discovering or accessing a project.
8. **Given** an invalid or incomplete `.grip/` boundary inside a valid outer Grip project, **When** a project-dependent command relies on implicit discovery, **Then** Grip reports invalid project metadata rather than silently selecting the outer project.

---

### User Story 3 - Commit and Reuse Portable Mapping Intent (Priority: P1)

A user records file and tree mappings using sources relative to the Grip project root and destinations relative to the invoking user's home. Another clone of the same project can interpret the same intent for its own project location and home directory.

**Why this priority**: Portable, version-controllable mapping intent is the principal benefit of replacing the global absolute-path registry.

**Independent Test**: Add mappings in one initialized temporary project, copy the project to a different absolute location with a different selected home, and verify that the same descriptor resolves sources beneath the new project root and destinations beneath the new home without modifying the descriptor.

**Acceptance Scenarios**:

1. **Given** an initialized Grip project containing `home/editor`, **When** the user records a tree mapping from `home/editor` to `~/.config/editor`, **Then** the descriptor stores that portable relative source and home-relative destination without either resolved absolute path.
2. **Given** a committed project cloned to another absolute directory, **When** a command resolves its mappings, **Then** each source is resolved beneath that clone's project root and each destination beneath that invoking user's home.
3. **Given** a source containing parent traversal, resolving outside the project root, or using an absolute form, **When** it is submitted or loaded, **Then** Grip rejects the mapping before publication or payload access.
4. **Given** a destination other than `~` or `~/RELATIVE_PATH`, or a destination containing parent traversal, **When** it is submitted or loaded, **Then** Grip rejects the mapping without expanding environment variables or accepting a machine-specific absolute path.
5. **Given** a valid mapping, **When** it is listed or shown, **Then** human and machine output distinguish portable declared paths from locally resolved paths without changing the stored declaration.
6. **Given** a tree mapping whose source is exact `.`, **When** Grip discovers project content, **Then** it treats the project root as the source while excluding `.grip/` before any ignore policy is evaluated; a file mapping with source `.` is invalid.

---

### User Story 4 - Keep Operational Evidence Local and Isolated (Priority: P2)

A user can commit or clone a Grip project's source content, `.grip/config.toml`, and `.grip/.gitignore` without also committing baselines, locks, recovery payloads, resolved machine paths, or operation records. Each clone creates and uses state beneath its own `.grip/` directory.

**Why this priority**: Synchronization safety depends on machine-specific evidence, while portability depends on keeping that evidence outside the source project.

**Independent Test**: Initialize two projects from the same committed configuration, perform independent baseline operations, and verify that each project reads and writes only its own `.grip/state/` scope and that `.grip/.gitignore` excludes that scope from Git.

**Acceptance Scenarios**:

1. **Given** an initialized project with no prior local activity, **When** a project-dependent read-only command runs, **Then** absent local state is reported according to that command's existing uninitialized-state contract and is never reconstructed from another clone.
2. **Given** two clones of the same committed Grip project at different canonical locations, **When** each publishes operational state, **Then** their baselines, locks, recovery evidence, and operation records remain isolated beneath their respective `.grip/state/` directories.
3. **Given** a Git clone containing committed `.grip/config.toml` and `.grip/.gitignore`, **When** the clone is inspected before local state exists, **Then** Grip recognizes the project and treats `.grip/state/` as locally uninitialized rather than missing portable intent.
4. **Given** a project-dependent mutation, **When** local state is selected, **Then** Grip binds the state beneath the selected project's `.grip/state/` to the current descriptor and resolved mappings before any payload mutation occurs.
5. **Given** a legacy global Grip registry or `GRIP_HOME` setting, **When** a Feature 010 command runs, **Then** Grip neither reads nor mutates the legacy mapping registry and does not use it as command scope.
6. **Given** a moved or filesystem-copied project that includes `.grip/state/`, **When** Grip first evaluates that state at the new location, **Then** it completely rebinds and revalidates the descriptor, mappings, source observations, destination environment, and accepted evidence before using the state for acceptance or mutation.

---

### User Story 5 - Preserve Existing Synchronization Guarantees Within a Project (Priority: P2)

A user continues to use mapping inspection, status, baseline, push, pull, sync, resolution, deletion, retirement, and recovery behavior with the established safety guarantees, but every operation is bounded to the resolved Grip project.

**Why this priority**: The new project model is a product-boundary correction, not permission to weaken the ownership, recovery, or bidirectional synchronization behavior already verified.

**Independent Test**: Exercise representative read-only, dry-run, and mutating workflows in two projects and verify that existing deterministic planning, ownership, revalidation, recovery, and baseline rules hold while no operation accesses the other project's mappings, payloads, or state.

**Acceptance Scenarios**:

1. **Given** two initialized Grip projects with disjoint mappings, **When** any command operates on one project, **Then** registry validation, selection, inspection, mutation, locking, recovery, and baseline publication exclude the other project completely.
2. **Given** a project-scoped mutating command, **When** its selected evidence drifts or an unsupported or conflicting entry exists, **Then** the existing blocking, revalidation, recovery, and truthful-result contracts remain enforced.
3. **Given** a dry-run project-scoped command, **When** it completes, **Then** it changes neither portable project intent nor machine-local state nor payload content.
4. **Given** human or machine-readable output, **When** a project-dependent result is rendered, **Then** it identifies the resolved project consistently while preserving the existing separation of results and diagnostics.

### Edge Cases

- The initialization target is absent, a regular file, a symbolic link, unreadable, not owned by the invoking user, or unsafe to modify.
- The `.grip/` directory, `config.toml`, or `.gitignore` is missing, malformed, unsupported, unsafe, replaced by a symbolic link, or changes during command startup.
- An ancestor-chain descriptor disappears, changes, or is substituted after discovery but before a mutating action.
- An explicit `--project` path names a descendant rather than the exact project root.
- The invocation directory is deleted or renamed during ancestor discovery.
- The user home cannot be determined, is relative, is unsafe, or changes between mapping resolution and mutation.
- A portable source is empty, uses `.` for a file mapping, contains `..`, uses an absolute path, targets project metadata, or resolves through a symbolic-link ancestor.
- A portable destination is `~`, has a trailing separator, contains `.`, `..`, repeated separators, unsupported encoding, or attempts another user's home notation.
- A mapping would cause the project descriptor or project-local source content to overlap a destination, another mapping, or machine-local state.
- A project is copied through the filesystem while `.grip/state/` contains accepted baselines or recovery evidence.
- `.grip/.gitignore` exists but differs from the canonical `/state/` rule through missing, additional, reordered, or contradictory content.
- A legacy global registry exists and conflicts with the selected project's mappings.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Grip MUST define an initialized source directory as a **Grip project**.
- **FR-002**: `grip init [PATH]` MUST initialize the explicit path, or the invocation directory when the path is omitted.
- **FR-003**: Initialization MUST require an existing safe ordinary directory and MUST NOT create a missing initialization target.
- **FR-004**: Initialization MUST create a `.grip/` metadata directory at the project root containing a portable `config.toml` descriptor with a supported schema version and a valid empty mapping collection; the descriptor MUST NOT contain a generated project ID.
- **FR-005**: Reinitializing a root with equivalent valid `.grip/` metadata MUST be an exact no-op; existing non-equivalent, invalid, unsupported, or unsafe metadata MUST block without replacement.
- **FR-006**: Initialization MUST NOT create or modify Git metadata, mappings, payload destinations, baselines, recovery evidence, operation records, or synchronization state.
- **FR-007**: Initialization MUST reject a target beneath an already initialized Grip project so ordinary initialization cannot create nested projects.
- **FR-008**: Every command that reads or changes mappings, payloads, baselines, recovery evidence, or operational state MUST resolve exactly one Grip project before accessing that project data.
- **FR-009**: Project-independent version and help operations MUST NOT require or discover a Grip project.
- **FR-010**: Project-dependent commands MUST accept a global `--project PATH` option whose path names the exact initialized project root containing `.grip/config.toml`; a descendant path is invalid rather than an alternate discovery starting point.
- **FR-011**: When `--project` is omitted, Grip MUST inspect the invocation directory and each ancestor through the filesystem root and MUST select the project only when exactly one valid project metadata boundary is found.
- **FR-012**: Explicit project selection MUST take precedence over implicit discovery and MUST NOT fall back to an enclosing project, legacy registry, or default global scope when the explicit selection is invalid.
- **FR-013**: Implicit discovery MUST fail on zero or multiple enclosing Grip projects and MUST NOT silently choose the nearest descriptor when more than one exists.
- **FR-014**: Grip MUST bind the discovered descriptor and canonical project root and revalidate relevant identity and safety evidence before any project-owned or payload mutation.
- **FR-015**: `.grip/config.toml` MUST contain portable user-authored mapping intent and MUST be suitable for committing to version control without machine-resolved absolute paths or machine-owned state.
- **FR-016**: Stored mapping sources MUST use normalized project-relative paths and MUST resolve beneath the canonical project root.
- **FR-017**: Stored mapping destinations MUST use literal `~` for the invoking user's home root or normalized `~/RELATIVE_PATH` for a descendant and MUST resolve beneath the invoking user's canonical home.
- **FR-018**: Grip MUST reject absolute sources, absolute destinations, parent traversal, other-user home notation, `${HOME}` and all other environment-variable expressions, and any declared path that escapes its permitted root.
- **FR-019**: Source arguments and selectors supplied to project-dependent commands MUST be interpreted in project-root-relative source space unless an existing command explicitly selects destination space.
- **FR-020**: Destination-space selection MUST preserve the existing directional meaning of source and destination while resolving the portable destination against the current user's home.
- **FR-021**: Mapping identity MUST remain stable and unambiguous within a project and MUST be based on the normalized project-relative source plus the mapping kind and portable destination wherever the complete tuple is required for safety.
- **FR-022**: Complete mapping validation MUST remain project-scoped and MUST reject equal, nested, overlapping, escaping, or cross-recursive ownership after paths are safely resolved.
- **FR-023**: Grip MUST maintain machine-owned baselines, locks, recovery evidence, operation records, and other local state beneath the selected project's `.grip/state/` directory and outside `config.toml`.
- **FR-024**: Each project's `.grip/state/` MUST be its complete local state boundary so separate clones do not share accepted baselines, locks, recovery evidence, or operation records through a global lookup.
- **FR-025**: A Git clone containing the committed `.grip/config.toml` and `.grip/.gitignore` but no `.grip/state/` MUST be recognized as the same portable project intent with a new uninitialized local state scope.
- **FR-026**: Grip MUST treat moved or copied `.grip/state/` contents as untrusted operational evidence until it completely rebinds and revalidates the selected project descriptor, resolved mappings, source observations, destination environment, and accepted evidence; any stale, incompatible, incomplete, or contradictory evidence MUST block acceptance and mutation.
- **FR-027**: Feature 010 MUST replace the global mapping model completely: project-dependent commands MUST NOT use `~/.grip/config.toml`, `GRIP_HOME`, or another global registry as mapping authority or command scope.
- **FR-028**: Grip MUST NOT implement migration, compatibility reads, compatibility writes, fallback, or dual-mode behavior for the superseded global mapping model.
- **FR-029**: Existing behavior for source-defined membership, destination-only unmanaged content, bidirectional accepted entries, conflicts, explicit deletion, retirement, recovery, dry runs, deterministic planning, pre-action revalidation, verification, and truthful baseline publication MUST remain effective within each resolved project.
- **FR-030**: Locks, recovery records, and mutation coordination MUST be scoped beneath the selected project's `.grip/state/` so concurrent operations in unrelated Grip projects do not contend solely because they belong to the same user.
- **FR-031**: Human output, machine-readable output, and diagnostics MUST remain distinct and MUST identify the resolved project consistently for project-dependent commands.
- **FR-032**: Project descriptor and state validation failures MUST use stable error categories that distinguish invalid usage, project not found, ambiguous project discovery, invalid project configuration, unsupported schema, corrupt local state, contention, and operational failure.
- **FR-033**: Project discovery and mapping resolution MUST not follow symbolic links to escape the project or destination roots and MUST preserve the allowlisted filesystem-node boundary.
- **FR-034**: The user-facing product documentation MUST describe Grip exclusively through the project-scoped model and MUST remove the superseded global mapping workflow rather than present both modes.
- **FR-035**: Automated acceptance coverage MUST use isolated temporary project roots and home roots and MUST prove that tests do not inspect or mutate the developer's real project, home, legacy Grip registry, or unrelated project state.
- **FR-036**: Initialization MUST create `.grip/.gitignore` with canonical contents consisting only of the root-relative `/state/` exclusion and a terminating newline, keeping `.grip/state/` out of Git while leaving `.grip/config.toml` eligible to be tracked; missing, additional, reordered, or contradictory content MUST be non-equivalent project metadata and MUST NOT be repaired automatically.
- **FR-037**: Grip MUST treat `.grip/` as reserved project metadata that is never eligible mapping payload, regardless of mapping breadth or ignore policy.
- **FR-038**: During implicit discovery, any enclosing `.grip/` node MUST establish a candidate project boundary; invalid, incomplete, unsafe, or unsupported candidate metadata MUST block discovery and MUST NOT be skipped in favor of an outer project.
- **FR-039**: `--project PATH` MUST be invalid usage with `grip init` and the application `grip version` command, and those commands MUST NOT perform project discovery in response to that invalid option.
- **FR-040**: The exact source declaration `.` MUST be valid only for a tree mapping rooted at the Grip project, and discovery for that mapping MUST structurally exclude `.grip/` before applying ignore policy; file mappings and non-exact dot-component forms MUST be rejected.

### Key Entities

- **Grip Project**: An initialized source directory that owns one portable project descriptor and provides the root for all declared source paths.
- **Project Metadata Directory**: The reserved `.grip/` directory containing portable configuration, Git exclusion policy, and machine-local state while remaining outside managed payload membership.
- **Project Descriptor**: The version-controllable `.grip/config.toml` document containing schema version and portable mapping intent, but no machine-local operational evidence or generated project ID.
- **Local Project Instance**: One filesystem copy of a Grip project whose operational state lives only beneath its own `.grip/state/` directory.
- **Portable Mapping**: A file or tree relationship expressed with a project-relative source and a home-relative destination.
- **Project Selection**: The explicit or ancestor-discovered binding that determines the only project a command may access.
- **Machine-Local State**: Baselines, fingerprints, locks, recovery evidence, operation records, and resolved machine paths stored beneath `.grip/state/` and excluded from Git by `.grip/.gitignore`.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In acceptance testing, 100% of initialization cases create exactly one valid portable descriptor or fail without changing the target, payloads, version-control metadata, or machine-local state.
- **SC-002**: In acceptance testing covering root, descendant, explicit, missing, invalid, and multiply enclosing cases, 100% of project-dependent commands select exactly the intended project or fail before project data access.
- **SC-003**: A project copied to at least two different absolute locations and evaluated with at least two different home roots resolves 100% of valid mappings correctly without changing the committed descriptor.
- **SC-004**: Inspection of every accepted `.grip/config.toml` fixture finds zero machine-specific absolute source paths, absolute destination paths, baselines, locks, recovery references, operation records, resolved local paths, or generated project IDs.
- **SC-005**: Two clones with the same committed `.grip/config.toml` maintain distinct `.grip/state/` contents in 100% of baseline, contention, recovery, and operation-record acceptance scenarios.
- **SC-006**: The complete existing synchronization acceptance suite passes under project-scoped execution with no cross-project payload, configuration, recovery, or baseline access.
- **SC-007**: All project-scoped dry-run scenarios produce zero changes to project descriptors, local state, and payloads while reporting the same selected project and action plan as corresponding execution scenarios.
- **SC-008**: Representative implicit project discovery from a directory 100 levels beneath a project root completes within one second in at least 95 of 100 local runs without persistent indexing or background services.
- **SC-009**: A repository-wide interface scan finds zero supported user workflows that depend on the legacy global mapping registry, absolute stored mapping paths, `GRIP_HOME` command scoping, or a generated project ID.
- **SC-011**: In 100% of initialization fixtures, `.grip/.gitignore` contains only canonical `/state/` content while leaving `config.toml` eligible for Git tracking, non-equivalent ignore content blocks without repair, and `.grip/` never appears as managed payload even for a project-root tree mapping.
- **SC-010**: Human and machine-readable acceptance fixtures identify the same project and outcome in 100% of initialization, discovery, mapping, inspection, mutation, and failure scenarios.

## Assumptions

- Grip remains a local, per-user, offline CLI and does not require Git; version control is enabled by the portable descriptor rather than performed by Grip.
- A Grip project is expected to contain source payloads and its descriptor, while destinations remain beneath the invoking user's home for this feature.
- `.grip/` is a reserved metadata namespace and not managed payload, even when a broad mapping otherwise includes the project root.
- The invoking user's home can be selected through the existing trusted platform boundary used by Grip's tests and runtime; failure to determine a safe home blocks destination resolution.
- A direct filesystem move or copy may include `.grip/state/`; Grip retains that evidence but must completely rebind and revalidate it before acceptance or mutation. An explicit migration or reset workflow remains outside this feature.
- Existing verified feature artifacts remain immutable historical evidence; Feature 010 carries all semantic replacements forward.
- No migration or backward compatibility is required because Grip has not been released.
