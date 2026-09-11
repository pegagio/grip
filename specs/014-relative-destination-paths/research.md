# Research: Relative Destination Paths

This research resolves the design choices needed to make a destination declaration portable relative to the selected Grip project.

## Declaration acceptance and persistence

**Decision**: Broaden the existing destination declaration parser to accept every non-empty UTF-8 relative spelling while retaining its exact input text in the descriptor. Continue rejecting unsupported other-user-home and environment-expansion-like forms.

**Rationale**: The feature expressly requires `./`, `../`, lexical dot components, and repeated separators to remain relative declarations. The destination declaration already has transparent serialization and participates in mapping identity, so preserving the raw text requires no schema change. Source declarations stay strict and normalized, which is an intentional different contract.

**Alternatives considered**: Reusing source command-input normalization was rejected because it would erase the required relative spelling. Persisting a resolved absolute path was rejected because it breaks copied-project portability. A new destination flag or syntax was rejected because the requested interface is ordinary relative input.

## Operational resolution

**Decision**: Resolve absolute destinations unchanged, resolve `~` and `~/` from the selected home, and resolve all other accepted destinations by joining them to the selected project root. Lexically normalize only the resulting absolute operational path without filesystem canonicalization.

**Rationale**: The selected project is the persistent base common to descriptor loading, project discovery, and explicit project selection; the process current working directory is not. Lexical normalization gives existing endpoint inspection an absolute path without following symbolic links and ensures equivalent resolved destinations are still detected by ownership and topology validation.

**Alternatives considered**: Resolving relative destinations from the current working directory was rejected because the same persisted mapping would change meaning from a descendant directory. Requiring that destinations remain inside the project was rejected because the accepted `../grip-dst/app/` use case intentionally targets an adjacent directory.

## Declaration-aware publication

**Decision**: Publish filtered or updated portable mapping declarations directly when removing a mapping rather than reconstructing them from resolved runtime endpoints.

**Rationale**: Add already publishes its portable candidate, but a resolver-to-home reconstruction during remove cannot faithfully recreate a retained project-relative declaration. Publishing the retained declaration snapshot preserves relative, absolute, and home-relative spelling consistently while resolved candidates continue to supply safety validation.

**Alternatives considered**: Reconstructing each retained destination from its resolved endpoint was rejected because a resolved path loses its declared base and lexical spelling. Converting all declarations to an absolute representation was rejected because it violates the portability and exact-persistence contract.

## Destination-space selectors and state

**Decision**: Use the same declaration parser and project-root-aware resolver for destination-space selectors, descriptor resolution, state binding, runtime state use, and rebinding.

**Rationale**: Every path that consumes a persisted declaration must derive the same endpoint. The shared selector boundary makes `--destination ../grip-dst/app/` consistent with `add`, and state/rebinding coverage proves copied projects retain the intended target.

**Alternatives considered**: Leaving selectors or state on home-only resolution was rejected because it would make an accepted declaration unusable after reload. Supporting relative declarations only during `add` was rejected because it creates a descriptor that later commands cannot interpret.

## Validation strategy

**Decision**: Add isolated tests for parser acceptance and exact descriptor round trips, project-root resolution independent of CWD, copied-project behavior, destination-space selection, resolved-equivalence ownership conflicts, remove publication, and existing absolute/home behavior.

**Rationale**: These scenarios directly cover the feature’s declared-versus-resolved distinction and preserve the existing safety boundary. They also detect the publication path that might otherwise rewrite retained declarations.

**Alternatives considered**: Parser-only testing was rejected because it would not prove descriptor reload, state rebinding, topology validation, or removal preserves the user’s declaration.
