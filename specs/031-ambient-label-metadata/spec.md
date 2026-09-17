# Feature Specification: Ambient Label Metadata and Add Diagnostics

**Feature Branch**: `031-ambient-label-metadata`

**Created**: 2026-09-16

**Status**: Implemented

**Input**: User description: "Ignore innocuous macOS metadata labels and make failed `add` output name the problematic file and unknown managed extended attribute."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Admit mappings with ambient macOS label metadata (Priority: P1)

An operator can add a mapping when either endpoint has an opaque macOS metadata-label attribute that Grip does not manage.

**Why this priority**: Ambient Finder or application labels must not make an otherwise valid dotfile mapping unusable.

**Independent Test**: Add a mapping whose destination has `com.apple.metadata:kMDLabel_<opaque-id>` and verify it succeeds without recording or transferring the attribute.

**Acceptance Scenarios**:

1. **Given** a source or destination has `com.apple.metadata:kMDLabel_` followed by a nonempty opaque suffix, **When** Grip inspects or adds the mapping, **Then** the attribute is excluded from comparison and transfer.
2. **Given** a different unknown extended attribute is present, **When** Grip inspects or adds the mapping, **Then** it remains a blocker.

### User Story 2 - Explain a rejected add (Priority: P2)

An operator sees why a mapping declaration was rejected without needing machine-readable output.

**Why this priority**: A failed declaration must identify the blocked managed entry and the metadata that requires action.

**Independent Test**: Add a mapping with a destination unknown xattr and verify human output names the file, endpoint, and attribute name, but not its value.

**Acceptance Scenarios**:

1. **Given** a candidate add finds an unknown managed extended attribute, **When** the operator uses human output, **Then** Grip names the affected file and says it has an unknown managed extended attribute, including the attribute name and endpoint.
2. **Given** the unknown attribute has a value, **When** Grip reports the failure, **Then** neither human nor JSON output reveals the value.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Grip MUST classify only names beginning with `com.apple.metadata:kMDLabel_` and having a nonempty suffix as excluded metadata.
- **FR-002**: Excluded label metadata MUST not participate in equality, baseline, or transfer decisions and MUST remain absent from persisted supported metadata evidence.
- **FR-003**: Grip MUST continue to block and report unknown extended attributes outside the narrow label family.
- **FR-004**: A human failed `add` caused by an unknown managed extended attribute MUST identify the managed file, endpoint, and attribute name in a concise blocker section.
- **FR-005**: Diagnostics MUST NOT include extended-attribute values.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of fixture mappings with the permitted label-family attribute add successfully without payload mutation.
- **SC-002**: 100% of fixture mappings with a non-label unknown attribute remain blocked.
- **SC-003**: 100% of human rejected-add fixtures include the affected path, endpoint, and attribute name while omitting the injected attribute value.

## Assumptions

- The label-family suffix is opaque and Grip does not need to decode or preserve its value.
- Existing explicitly excluded xattrs remain quiet in normal human output.
