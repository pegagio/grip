# Data Model: Forced Missing-Peer Restoration

This feature introduces no persisted entity or schema change. It clarifies how existing runtime evidence is interpreted for one exact forced selection.

## Present forced winner

- **Identity**: An established exact managed file or tree-entry identity.
- **Winner side**: Source for forced push or destination for forced pull.
- **Current state**: Present complete supported state, which may equal or differ from the accepted baseline.
- **Peer state**: Absent at the opposite endpoint.
- **Outcome**: A directional restoration action copies the winner's complete current state to the peer and publishes a fresh accepted baseline only after verification.

## Absent forced winner

- **Identity**: An established exact managed identity.
- **Winner side**: Source for forced push or destination for forced pull.
- **Current state**: Absent at the selected winner endpoint.
- **Peer state**: Present, unchanged from accepted state, and eligible for existing exact deletion authority.
- **Outcome**: The existing deletion path propagates the winner's absence and retires its accepted baseline.

## Invariants

- A present winner and missing peer remains one exact identity; it never authorizes sibling, parent, subtree, mapping-wide, or aggregate mutation.
- Current winner evidence must survive complete inspection and pre-action revalidation before it is copied.
- A dry run derives the same identity, direction, and candidate action without mutating endpoints or accepted state.
- A successful restoration leaves both endpoints verified equivalent and stores fresh accepted evidence for that identity.
