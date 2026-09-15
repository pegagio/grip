# Feature Specification: Local Artifact Release Automation

**Feature Branch**: `024-local-artifact-release-automation`

**Created**: 2026-09-15

**Status**: Complete

**Input**: User description: "Automate local Grip artifact releases with a mise task while retaining manual Git tag pushing and GitHub Release publication."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Prepare a release artifact (Priority: P1)

A maintainer with a clean `master` checkout prepares a release locally with one documented task. The task verifies the release candidate, creates the supported distribution archive and checksum, and creates a local annotated tag for the exact verified commit.

**Why this priority**: This turns a repeatable but error-prone release checklist into one safe, reviewable local operation.

**Independent Test**: From an eligible checkout with an unused project version, a maintainer can run the documented task and receive one archive, one checksum, and one local annotated tag that all use the same release version.

**Acceptance Scenarios**:

1. **Given** a clean `master` checkout with an unused valid release version, **When** the maintainer runs the release-preparation task, **Then** it completes validation, produces a macOS Apple Silicon archive and SHA-256 checksum in the documented output location, and creates an annotated local tag for that exact commit.
2. **Given** the prepared archive and checksum, **When** the maintainer extracts the archive and verifies the checksum, **Then** the executable, README, and license material are present and the checksum matches the archive.

---

### User Story 2 - Prevent an unsafe or ambiguous release (Priority: P2)

A maintainer receives a clear failure before a release identity is created when the candidate is not safe to release.

**Why this priority**: A release tag is a durable identity; preventing a misleading tag or overwritten asset is more valuable than completing a partial preparation run.

**Independent Test**: Each ineligible condition can be introduced independently and proves that the task reports the condition without replacing an existing artifact or tag.

**Acceptance Scenarios**:

1. **Given** uncommitted checkout changes or a branch other than `master`, **When** the maintainer runs the release-preparation task, **Then** it fails before producing release artifacts or creating a tag and identifies the failed precondition.
2. **Given** an existing local tag or completed artifact set for the selected version, **When** the maintainer runs the task, **Then** it refuses to overwrite that release identity and identifies the collision.
3. **Given** validation, packaging, or checksum generation fails, **When** the maintainer runs the task, **Then** it creates no local release tag and does not present incomplete output as releasable.

---

### User Story 3 - Publish deliberately (Priority: P3)

After preparation succeeds, a maintainer can inspect the local tag and artifacts, then explicitly choose whether and when to push the tag and publish the GitHub Release.

**Why this priority**: External publication is a distinct, irreversible decision that should remain visible and intentional.

**Independent Test**: A successful preparation run performs no Git push, GitHub Release or upload action, GitHub API call, or package-registry publication, and its documentation identifies the separate manual publication steps.

**Acceptance Scenarios**:

1. **Given** a successfully prepared release, **When** the task completes, **Then** no branch or tag has been pushed and no GitHub Release has been created or modified.

### Edge Cases

- The project version is empty, malformed, or cannot form a valid release tag.
- The archive, checksum, and tag version disagree because project metadata or inputs changed during preparation.
- A previous interrupted run left incomplete output in the release-artifact location.
- The supported release target cannot be built on the current machine.
- The local tag points to a commit other than the checked release candidate.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The project MUST provide one documented local mise release-preparation task for maintainers.
- **FR-002**: The task MUST run only from a clean checkout of `master` and MUST identify a violated branch or cleanliness precondition before creating release artifacts or a tag.
- **FR-003**: The task MUST complete the documented release validation gates before treating any output as releasable.
- **FR-004**: The task MUST derive one valid release version from the project’s release metadata and use it consistently for the archive name, checksum name, and local tag.
- **FR-005**: The task MUST produce a compressed macOS Apple Silicon distribution archive containing the Grip executable, README, and license material.
- **FR-006**: The task MUST produce a SHA-256 checksum for the exact archive it creates.
- **FR-007**: The task MUST place the archive and checksum in one documented, predictable project-local output location.
- **FR-008**: The task MUST create an annotated local tag for the exact clean `master` commit only after the validation and artifact steps succeed.
- **FR-009**: The task MUST refuse to overwrite an existing release archive, checksum, or local tag for the selected version.
- **FR-010**: On any failure, the task MUST report the failed stage, MUST NOT create a local release tag, and MUST NOT represent incomplete artifacts as releasable.
- **FR-011**: The task MUST NOT push branches or tags, create or modify a GitHub Release, upload artifacts, contact the GitHub API, or publish to a package registry.
- **FR-012**: The release documentation MUST state how to inspect the prepared artifact and how to perform the separate manual push and GitHub publication steps.

### Key Entities

- **Release version**: The valid project release identifier used consistently across the artifact names and local tag.
- **Release artifact set**: The archive and matching checksum prepared for one release version and one exact commit.
- **Local release tag**: The annotated local Git identity created only after the corresponding artifact set is ready.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: On an eligible checkout, a maintainer completes local release preparation with one documented task and obtains exactly one archive, one matching SHA-256 checksum, and one local annotated tag for the same version.
- **SC-002**: In 100% of tested ineligible cases—dirty checkout, wrong branch, invalid version, existing release identity, validation failure, and packaging failure—the task creates no new local release tag.
- **SC-003**: In 100% of successful test runs, the distribution archive extracts successfully, contains the documented three delivered files, and its published checksum verifies the extracted archive’s source file.
- **SC-004**: In 100% of successful test runs, no remote branch/tag changes, GitHub Release changes, artifact uploads, or package-registry publication occur.

## Governing Constraints

- The implementation must remain a small local workflow; no external service, daemon, CI workflow, GitHub API integration, or package-registry publishing is introduced.
- The workflow must preserve explicit operator control over externally visible publication and must not overwrite an existing local release identity.
- Feature artifacts, implementation, tests, and documentation remain one mutable, reviewable change set until merged into the integration branch.

## Assumptions

- The existing project release metadata is the authoritative source for the release version, and the local tag uses that version with a `v` prefix.
- The initial target remains macOS on Apple Silicon; cross-platform packaging, signing, notarization, and installer formats are separate future work.
- Maintainers run the task only after merging the release candidate to `master`; manual remote publication occurs after local artifact and tag inspection.
