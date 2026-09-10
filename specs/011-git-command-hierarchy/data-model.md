# Data Model: Git-Inspired Command Hierarchy

## Public concepts

| Concept | Fields and invariant |
|---|---|
| Project | Root selected from the working directory or global `-p`/`--project PATH`. It owns `.grip`, mappings, and ignore policy. |
| Mapping | Source path, destination path, inferred compatible entry kind, and stable identity. A mapping declaration never itself copies payloads. |
| Active membership | The set of mapping entries not excluded by the current `.gripignore` rules. Only active membership may use a baseline. |
| Ignore-policy revision | A private token containing the relevant `.gripignore` content digest and filesystem change identity/time when a baseline is established. A changed token makes previous baseline evidence ineligible, even when rules are later restored. |
| Endpoint observation | Existence, supported entry kind, content state, and canonical path for either side of a mapping. |
| Baseline | Private comparison evidence keyed by an active mapping identity and its ignore-policy revision. It exists only after matching registration or successful verified synchronization. |
| Selection | Optional path plus the selector space. `-d` changes the path space to destination; it never changes the direction of a command. |
| Publication request | Direction (`push` or `pull`), selected entries, dry-run state, and force state. Force requires one selected managed entry. |
| Output mode | Human by default or explicit JSON via `-o json`/`--output json`. |

## Mapping lifecycle

```text
unmapped
  └─ add (at least one endpoint exists) ──> declared

declared + matching endpoints ──> baselined
declared + different or missing endpoint ──> unbaselined

baselined ── remove or observed ignore ──> unmapped/unbaselined
unmapped/unbaselined ── later add or unignore ──> newly discovered
```

`newly discovered` has no inherited baseline, even if a prior mapping used the same paths. A later matching observation or successful directional publication establishes fresh evidence.

## Baseline persistence and membership reconciliation

Current state binds baseline identities to the project descriptor. The replacement state model removes pending-retirement records and stores only baselines that carry the current mapping's relevant ignore-policy revision.

For an operation that needs baseline evidence:

1. Load and validate the mapping descriptor, ignore policy, and private state binding.
2. Derive active membership and the current ignore-policy revision for each tree mapping.
3. Treat any baseline for an inactive mapping or changed policy revision as ineligible before classification.
4. Construct a state candidate containing only eligible baselines.
5. On `status` and non-dry-run mutating paths, publish the private pruning transition under bounded state/registry locking before classification proceeds. On `diff` and every dry run, apply the same in-memory eligibility rule without persisting state.

The policy revision means an ignored-then-unignored entry cannot recover a stale baseline even if the transition happened between Grip commands. The reconciliation does not change endpoint payloads or mapping declarations.

`remove` performs the same pruning as part of its descriptor mutation. Its implementation prevalidates descriptor and state candidates, publishes them as one logical transition, and leaves at most an ephemeral completion marker for an interrupted internal publication. Startup and the next state-writing path finish such a marker before baseline use. The marker is neither a user-facing record nor a payload-recovery artifact.

## Synchronization state rules

| Endpoint condition | Ordinary push/pull | Sync | Forced direction |
|---|---|---|---|
| Both endpoints match | No content publish; baseline may be established or refreshed. | No content publish. | Not needed. |
| One endpoint changed from baseline | Publish only in its clear direction. | Select the clear direction. | Allowed for one selected entry. |
| Both endpoints changed differently | Block. | Block. | Direction chooses winner for one entry. |
| One endpoint absent | Block. | Block. | Direction may propagate winner or absence for one entry. |
| Endpoints differ with no baseline | Explicit `push` or `pull` required. | Block. | Direction may choose winner for one entry. |

## Internal transition invariants

- A baseline is never read when its mapping is inactive, removed, newly reintroduced, or bound to an old ignore-policy revision.
- `add` never creates, modifies, or deletes endpoint payloads.
- A dry run leaves registry, endpoint payloads, and baseline state unchanged while reporting the same plan that a real operation would evaluate.
- A failed publication leaves endpoint payloads and new baseline publication unchanged; an independently completed pre-publication membership-pruning transition may remain durable.
- A successful content publication updates baseline evidence only after revalidation and post-publication verification.
- Temporary staging is discarded after success or failure and is never exposed as a recovery command.
- No model type represents pending retirement, user recovery records, or a public undo target.
