# `grip add` Initial-Authority Contract

`grip add SOURCE DESTINATION` retains its existing syntax and concise success output. This contract defines only the post-add classification for source-defined members.

## Preconditions

- At least one endpoint exists and both existing endpoints have compatible supported node kinds.
- The source belongs to the selected project; the destination resolves under the existing destination-path policy.
- The mapping declaration, endpoint evidence, and required initial state can be safely inspected and published.

## Outcomes

| Source state | Destination state | Payload effect of `add` | Next normal status and plan |
|---|---|---|---|
| Complete and equal | Complete and equal | None | Current; existing equivalent baseline behavior |
| Complete and unequal | Complete and unequal | None | `Changes to push: SOURCE -> DESTINATION`; push and sync select it; pull does not |
| Complete | Absent | None | Existing source-addition pending push |
| Absent | Complete | None | Destination-only content remains unmanaged |

For a tree mapping, the table applies to every source-defined non-ignored member independently. A destination-only or ignored member is not made managed by this feature.

## Later change behavior

After an unequal add, the destination-derived baseline is the comparison reference. A destination change before push produces the existing conflict behavior. A successful ordinary push copies source to destination and publishes the usual accepted source baseline.

## Publication failure behavior

For an unequal pair, Grip creates and verifies an incomplete-add publication fence before publishing the descriptor or required initial state. The fence binds the exact mapping identity plus prior and candidate descriptor/state digests. If fence creation or verification fails, `add` returns an error with the descriptor and state unchanged. Once fenced, Grip publishes the candidate descriptor and State V4 state, verifies that both match the candidate digests, and only then clears the fence. If a later step fails, the fence remains and that mapping fails with an existing configuration or state-integrity category; it does not offer an ordinary or forced synchronization action. Retrying the same `grip add` revalidates the transition and either completes it or restores the prior descriptor before clearing the fence. If a state publication is already visible but cannot confirm directory durability, Grip retains the matching descriptor/state pair and keeps the fence until the retry verifies and clears it.

## Compatibility

The feature does not change default add output, selectors, force semantics, `grip diff`, or JSON shapes. The new unequal mapping appears through the existing `source_only_change` classification and its already-established status and mutation representations. The fenced incomplete-add condition uses an existing error category and has no normal mapping classification. A command explicitly selecting only unfenced mappings remains available; a command whose selected set includes the fenced mapping reports it as blocked.
