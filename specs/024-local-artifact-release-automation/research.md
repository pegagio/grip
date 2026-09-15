# Research: Local Artifact Release Automation

## Decision: Expose one `mise run release` task backed by a repository-owned script

**Rationale**: Existing project workflows are flat mise tasks (`build`, `test`, `validate`, and `performance`). The release flow has ordered guard, staging, archive, checksum, and tag behavior that would be difficult to review or test as one inline task declaration. A small script keeps the public command concise while preserving a local, dependency-free design.

**Alternatives considered**:

- An inline mise command list: rejected because safe cleanup, collision rechecks, and stage-specific failure messages would become opaque.
- GitHub Actions or a GitHub API workflow: rejected because Feature 024 intentionally retains manual external publication.
- A new Rust release binary: rejected because this is maintainer tooling, not Grip product behavior.

## Decision: Derive the release identity from Cargo package metadata

**Rationale**: `Cargo.toml` owns the package version, the Grip CLI exposes that package version, and the existing release tag uses the `v<version>` convention. Deriving the version from Cargo metadata avoids a second manually supplied version that could disagree with the binary.

**Alternatives considered**:

- A required task argument: rejected because it can disagree with the package version.
- Parsing a version with an ad hoc text match: rejected because Cargo already provides authoritative package metadata.

## Decision: Reuse both existing release gates

**Rationale**: `mise run validate` performs formatting, linting, default tests, and a release build. `mise run performance` runs the ignored release performance acceptance suite. The README identifies both as explicit release gates, so Feature 024 must compose rather than duplicate or weaken them.

**Alternatives considered**:

- Run only a release build: rejected because it omits established quality and performance evidence.
- Add another test framework: rejected because the existing task system and isolated shell contract test cover the new boundary.

## Decision: Use `dist/` with private attempt staging and collision refusal

**Rationale**: Root `dist/` is already ignored and is a predictable artifact location. Final archive and checksum files must never be overwritten. Attempt-owned staging and final collision rechecks prevent an interrupted or concurrent preparation attempt from presenting incomplete or replaced output as releasable.

**Alternatives considered**:

- Write directly to final archive paths: rejected because failures can leave misleading partial artifacts.
- Use `target/release`: rejected because Cargo owns that build output and it does not represent a complete distributable artifact set.
- Use a root `release/` directory: rejected because `target/release` already names the Cargo profile output and `dist/` better distinguishes packaged deliverables.

## Decision: Require local `master`, clean state, and supported host before work

**Rationale**: The local tag must identify the exact reviewed release commit. A clean attached `master` checkout and captured `HEAD` provide that boundary. Feature 024 supports only a Darwin/arm64 artifact, so the task must reject a different host before creating outputs.

**Alternatives considered**:

- Compare to `origin/master`: rejected because it introduces remote availability and divergence policy outside the agreed local scope.
- Permit any clean branch: rejected because it weakens the release identity guarantee.
- Cross-compile automatically: rejected because cross-platform artifacts are explicitly out of scope.

## Decision: Create the local annotated tag last and never push

**Rationale**: A tag is the durable local identity for a completed release candidate. Creating it only after all validation and packaging succeeds prevents a failed preparation attempt from being mistaken for a release. No push or GitHub command appears in the workflow, so publication remains a deliberate maintainer action.

**Alternatives considered**:

- Tag before validation: rejected because a failed candidate would already have a release identity.
- Push from the task: rejected because it makes release publication externally visible without an explicit final review.

## Decision: Validate with an isolated Git-repository shell contract test

**Rationale**: The feature boundary spans Git state, archives, checksums, and local task execution. An isolated temporary repository can prove success and failure behavior without changing the developer checkout or any remote. Existing Rust tests already use isolated temporary roots; the same principle applies here.

**Alternatives considered**:

- Manually exercise the task only: rejected because collision and failure behavior must be repeatable.
- Embed the release workflow in Rust tests: rejected because it would add production-language coupling to maintainer-only shell behavior.
