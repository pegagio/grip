# Data Model: Local Artifact Release Automation

## Release Candidate

Represents the exact local source state eligible for preparation.

| Field | Meaning | Validation |
|---|---|---|
| repository root | Project root containing release metadata and documentation | Must be the Git top-level directory |
| branch | Attached branch for the candidate | Must be exactly `master` |
| commit | Captured candidate commit | Must remain `HEAD` through local tag creation |
| working state | Tracked and untracked source changes | Must be clean before work and before tag creation |
| platform | Host platform and architecture | Must be Darwin/arm64 |
| package version | Version read from Cargo package metadata | Must be non-empty, valid, and convertible to `v<version>` |

## Release Artifact Set

Represents the files prepared for one release candidate.

| Field | Meaning | Validation |
|---|---|---|
| archive path | `dist/grip-v<version>-darwin-arm64.tar.gz` | Must not exist before finalization; archive must contain executable, README, and license material |
| checksum path | Archive path plus `.sha256` | Must describe the exact final archive and verify successfully |
| staging directory | Attempt-owned private packaging location | Must not be treated as a final artifact and must be cleaned after failure |
| version | Shared identity | Must equal the candidate package version |
| target | Supported release target | Must equal `darwin-arm64` |

## Local Release Tag

Represents the local Git release identity.

| Field | Meaning | Validation |
|---|---|---|
| name | `v<version>` | Must not already exist |
| type | Annotated Git tag | Must be annotated, not lightweight |
| target | Commit identified by the tag | Must equal the captured candidate commit |
| creation point | Final workflow transition | May be created only after final artifact and checksum verification |

## State Transitions

```text
candidate checked
  -> release gates passed
  -> artifact staging verified
  -> final archive and checksum published
  -> local annotated tag created
  -> ready for explicit manual publication
```

Any failure before the final transition ends the run without creating a new local tag. Existing final identities remain untouched.
