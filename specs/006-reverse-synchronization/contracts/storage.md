# Storage Contract: Direction-Aware Operation Evidence

Feature 006 preserves State Envelope V2 and the Feature 005 operation/recovery layout. It adds no pull-specific state root.

## Layout

```text
<grip-home>/
├── config.toml
├── .mutation.lock
├── .registry.lock
└── state/
    ├── state.json
    ├── state.lock
    ├── recovery/generation-<N>/state.json
    └── operations/<operation-id>/
        ├── plan.json
        ├── operation.json
        ├── actions/<zero-padded-index>.json
        └── recovery/<zero-padded-index>/
            ├── metadata.json
            └── payload
```

## Operation identity

Partitioned Operation Record V1 admits `operation: push` and `operation: pull`. Plan and summary operation values must agree. Pull records use exclusively allocated opaque identifiers and may use a `pull-` diagnostic prefix. Identifier text never grants authority or bypasses path containment.

The immutable `plan.json` contains the complete shared mutation plan with `direction`. Action checkpoints retain the existing generic milestones. Operation summaries retain baseline and result-delivery states. Unknown operation values, mismatched direction, invalid integrity, escaping recovery references, or inconsistent action bindings are corrupt state.

## Recovery binding

Recovery Metadata V1 is unchanged. For pull, `prior_state` describes the replaced source. The immutable parent plan binds direction, Entry Identity, mapping-role paths, and action index, so recovery metadata does not duplicate source/destination side flags.

## Writer coordination

Every writer preserves the global order:

```text
.mutation.lock -> .registry.lock -> state/state.lock
```

Actionful pull holds the mutation lock from lock-held revalidation through operation checkpoints, source mutation, final observation, and baseline publication. Mapping changes and changed baseline acceptance remain participants in the same outer lock. Dry runs, blocked plans, no-ops, and read-only commands remain lock-free.

## Accepted-state publication

State Envelope V2 remains the sole accepted baseline authority. After every pull action verifies, publish one generation that updates actioned identities to the complete supported destination state observed on both sides. Preserve selected no-actions, out-of-scope records, and pending-retirement records. Do not publish for blocked, no-op, stale, contended, partial, or verification-failed pull results.

## Interrupted records and delivery

Ordinary planning does not enumerate retained operations. A valid nonterminal pull record remains immutable and does not itself block a later invocation. A later pull performs fresh inspection and allocates a separate operation ID. Terminal baseline authority and result-delivery checkpoints use the existing V1 transition rules; failed output delivery changes neither payload nor baseline authority.
