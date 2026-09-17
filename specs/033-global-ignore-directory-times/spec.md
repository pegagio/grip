# Feature Specification: Global Ignore Policy and Directory Timestamp Boundary

**Feature Branch**: `fixups`  
**Created**: 2026-09-16  
**Status**: Implemented  
**Input**: Direct active-user decisions: project-root `.gripignore` is a global policy for all tree mappings, and directory modification times are unmanaged.

## User Scenarios & Testing

### User Story 1 - Apply one repository policy to tree mappings (Priority: P1)

An operator places `**/.DS_Store` in the Grip project root and runs status for a `home/` tree mapping. The ignored member is absent from the managed inventory and cannot be pushed.

**Acceptance Scenarios**:

1. **Given** a project-root `.gripignore` and a tree mapping below the project root, **When** Grip inspects the tree, **Then** root policy rules apply to that mapping.
2. **Given** a narrower source `.gripignore` negation, **When** it matches an entry, **Then** it overrides the matching global rule at that source scope.

### User Story 2 - Ignore directory timestamp drift (Priority: P1)

An operator changes only the modification times of paired directories. Status and diff classify the directory as current rather than as a conflict or directional change.

**Acceptance Scenarios**:

1. **Given** paired directory states with different modification times and equal managed metadata, **When** Grip compares them to an accepted baseline, **Then** the directory is synchronized and reports no modification-time difference.
2. **Given** a directory metadata mutation, **When** Grip applies other supported directory metadata, **Then** it does not set or require the directory modification time.

## Requirements

- **FR-001**: The project-root `.gripignore` MUST be the lowest-precedence policy for every tree mapping in that project.
- **FR-002**: Source-root and nested `.gripignore` policies MUST retain their existing narrower-scope precedence.
- **FR-003**: Project-root and source-tree policy files MUST remain excluded from mapping payload.
- **FR-004**: Directory modification time MUST be excluded from classification, changed-dimension reporting, mutation capability requirements, transfer, and verification.
- **FR-005**: Regular-file modification time MUST remain managed.

## Success Criteria

- **SC-001**: A project-root `**/.DS_Store` rule excludes matching descendants from every inspected tree mapping.
- **SC-002**: A directory whose only difference is modification time produces zero managed differences.
- **SC-003**: Existing file metadata comparison tests continue to distinguish a file modification-time change.

## Scope

**In scope**: global project policy precedence, ignored membership, directory timestamp comparison and transfer boundary, documentation, and regression tests.

**Out of scope**: changing file timestamp semantics, broadening ignore syntax, external diff-program behavior, or changing other directory metadata.
