# Data Model: Mapping Registry and Ownership Validation

This model separates user-authored wire data, canonical domain identity, path evidence, and the publication candidate.

## Table of Contents

- [Registry V1](#registry-v1)
- [Mapping](#mapping)
- [Canonical path](#canonical-path)
- [Namespace](#namespace)
- [Ownership conflict](#ownership-conflict)
- [Registry snapshot and candidate](#registry-snapshot-and-candidate)
- [Registry recovery generation](#registry-recovery-generation)
- [Lifecycle transitions](#lifecycle-transitions)
- [Validation order](#validation-order)

## Registry V1

The complete user-authored intent document.

| Field | Type | Rules |
|---|---|---|
| `schema_version` | positive integer | Required and exactly `1` |
| `mappings` | ordered collection of Mapping Wire values | Required; empty is valid; unknown fields are rejected |

The accepted serialized order is canonical source-path order. Semantic fields survive every update; comments and presentation-only ordering may be normalized.

## Mapping

The version-neutral domain relationship used for ownership validation.

| Field | Type | Rules |
|---|---|---|
| `kind` | `File` or `Tree` | Required |
| `source` | Canonical Path | Required; sole user-facing mapping identity |
| `destination` | Canonical Path | Required |

Wire values use lowercase `file` and `tree`. A file source exists as an ordinary regular file. A tree source exists as an ordinary directory. A destination may be absent but must resolve from a safe existing ancestor; an existing destination must match the mapping kind.

Mappings have no user-assigned identifier. Equality requires the same kind, source, and destination, while identity is source alone.

## Canonical path

A validated absolute path used for identity and component-aware comparisons.

| Field | Type | Rules |
|---|---|---|
| `submitted` | path value | Retained only for safe diagnostics during one command |
| `canonical` | absolute UTF-8 path | Stored; normalized through the longest existing prefix |
| `existence` | `Existing` or `AbsentDestination` | Sources cannot be absent |
| `node_kind` | `RegularFile`, `Directory`, or absent | Must agree with mapping kind when present |
| `evidence` | observed ancestry and final-node identity | Recomputed before publication; not persisted in registry |

Relative paths, parent traversal, non-UTF-8 values, inaccessible ancestry, final-node symbolic links, unsupported nodes, and changed evidence are invalid. Accepted Registry V1 documents must be current-user-owned regular files with group/other write bits unset; mutation additionally requires the owner-write bit.

## Namespace

The extent a mapping can own without discovering members.

| Field | Type | Meaning |
|---|---|---|
| `root` | Canonical Path | Source or destination root |
| `extent` | `Exact` or `Tree` | Exact owns one path; Tree reserves the root and prospective descendants |
| `side` | `Source` or `Destination` | Directional role for diagnostics |
| `mapping_source` | Canonical Path | Identity of the owning mapping |

Relations are component-aware: `Equal`, `Ancestor`, `Descendant`, or `Disjoint`. String-prefix similarity alone never establishes containment.

## Ownership conflict

A deterministic explanation of an invalid relation.

| Field | Type | Rules |
|---|---|---|
| `reason` | stable enum | One of `duplicate_source`, `duplicate_tuple`, `source_overlap`, `destination_overlap`, `equal_endpoints`, `recursive_topology`, or `cross_mapping_recursion` |
| `first_mapping` | Canonical Path | Lower canonical source identity |
| `second_mapping` | Canonical Path or absent | Present for pair conflicts |
| `first_path` | Canonical Path | First conflicting namespace |
| `second_path` | Canonical Path | Second conflicting namespace |
| `relation` | path relation | Component-aware relation that caused rejection |

All conflicts are collected, deduplicated, and ordered by mapping identities, paths, and reason before rendering.

## Registry snapshot and candidate

`RegistrySnapshot` binds the accepted document bytes to the decoded and validated Registry V1 plus observed registry-node evidence. `RegistryCandidate` contains the complete post-operation registry and the path evidence used to validate it.

A candidate is publishable only when:

1. the accepted registry is fully valid;
2. the requested lifecycle operation is valid;
3. every mapping path resolves and has the required node status;
4. the candidate ownership graph has no conflicts;
5. after acquiring the registry lock, the accepted bytes and relevant path evidence still match;
6. the exact accepted bytes are retained and verified as a Registry Recovery Generation;
7. serialized candidate bytes decode to the same domain value and retain the accepted registry mode.

## Registry recovery generation

An immutable recovery record created before accepted registry replacement.

| Field | Type | Rules |
|---|---|---|
| `digest` | lowercase SHA-256 | Computed over the exact accepted `config.toml` bytes and used in the generation directory name |
| `config` | exact byte sequence | Must be byte-identical to the accepted snapshot and decode as that same valid registry |
| `path` | Grip-owned path | `state/recovery/registry/sha256-<digest>/config.toml` |

An existing byte-identical generation is reused. An existing mismatched, unsafe, or unverifiable generation blocks replacement. The generation is retained after successful publication; restoration and cleanup transitions are outside Feature 002.

## Lifecycle transitions

```text
Absent mapping --add file/tree--> Recorded mapping
Recorded mapping --show/list--> Recorded mapping
Recorded mapping --remove--> Absent mapping
Any state --validation/recovery/publication failure--> Prior accepted registry
```

Add and remove change registry intent only after retaining the prior registry as recovery evidence. Show and list are read-only. None of these transitions creates managed tree members, synchronization state, baselines, payload backups, or payload actions.

Repeated add of an existing source is a validation failure, even if the tuple is identical. Removal or show of a missing source is a not-found validation failure. Repeating list over equivalent registry evidence produces identical ordering and fields.

## Validation order

1. Validate Grip home and accepted `config.toml` node.
2. Decode and version-check the complete Registry V1.
3. Canonicalize and validate every stored mapping path.
4. Validate the complete accepted ownership graph.
5. Parse and canonicalize the requested mapping identity or endpoints.
6. Construct the complete candidate for add or remove.
7. Validate candidate paths and ownership graph.
8. For writes, acquire the stable lock, repeat steps 1–7 from current evidence, and require the accepted bytes and planned identities to match.
9. Retain and verify the exact prior registry recovery generation.
10. Stage, decode, compare, preserve the accepted mode, and atomically publish the candidate.
