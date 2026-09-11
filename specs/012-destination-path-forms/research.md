# Research: Destination Path Forms

## Declaration type

**Decision**: Replace the home-only declaration value with a destination declaration type that preserves valid UTF-8 spelling and distinguishes absolute, `~`, and `~/` forms. Keep `ProjectRelativePath` unchanged for sources.

**Rationale**: `HomeRelativePath` in `src/path_policy.rs` accepts only normalized home-relative input, while `PortableMapping` stores it directly. A distinct destination declaration makes the permitted forms explicit without expanding source semantics.

**Alternatives considered**: Relaxing `HomeRelativePath` was rejected because it remains home-only. Persisting resolved absolute paths was rejected because it loses required declaration spelling.

## Operational resolution

**Decision**: Persist accepted destination text unchanged. Expand `~` and `~/` against the selected home and lexically normalize a separate operational path without filesystem canonicalization.

**Rationale**: The feature accepts `.`, `..`, and repeated separators, including paths resolving outside home. Existing `validate_input` rejects parent components, so it must receive the operational path after lexical normalization. Existing non-following inspection and topology validation remains in force.

**Alternatives considered**: Requiring normalized user input violates Feature 012. Filesystem canonicalization was rejected because it can follow symlinks and weaken the existing safety boundary.

## Descriptor and state publication

**Decision**: Make registry mutations declaration-aware and publish selected `PortableMapping` declarations directly. State rebinding must carry the original declaration instead of recreating a home-relative path from a runtime endpoint.

**Rationale**: `src/registry/publication.rs` and `src/state/mod.rs` currently reconstruct declarations by stripping the runtime destination from home. That rejects absolute values and discards lexical input. Runtime registries remain necessary for evidence and ownership validation, but cannot be the source of descriptor spelling.

**Alternatives considered**: Runtime-to-home-relative conversion cannot represent absolute mappings. A user-visible migration or recovery flow is out of scope because existing valid declarations remain valid under the broader parser.

## Destination-aware selectors

**Decision**: Use the same destination declaration resolver for existing destination-space selectors.

**Rationale**: `src/lib.rs` currently uses the home-only type for `--destination` selectors. Reusing the generalized resolver lets operators select absolute mappings without adding new CLI syntax.

## Validation Strategy

- Unit-test absolute, `~`, ordinary `~/`, `~/a/./b`, `~/a/../b`, and repeated-separator declarations; retain rejection of relative, other-user, environment-expansion, empty, and non-UTF-8 forms.
- Use `ProjectFixture` tests to prove `add` persists absolute and non-normalized home-relative spelling exactly and reloads them for `list`, `status`, and destination-space selection.
- Assert malformed input leaves descriptor bytes unchanged, and equivalent resolved destinations still conflict under ownership validation.
- Update `docs/product-definition.md` and `README.md` to remove home-only and absolute-rejected claims.
