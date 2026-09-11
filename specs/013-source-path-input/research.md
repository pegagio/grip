# Research: Source Path Input Normalization

## Command input and descriptor declarations

**Decision**: Keep the existing strict project-relative parser as the descriptor decoder and introduce a distinct source-input parser for CLI arguments.

**Rationale**: Persisted declarations must remain canonical and safe to deserialize without implicit rewriting. CLI arguments are user conveniences and may contain harmless lexical components such as `./` and a trailing separator. Separating the boundaries allows the command line to normalize input while preserving strict storage validation.

**Alternatives considered**: Relaxing the descriptor parser was rejected because malformed or noncanonical descriptor content would become silently accepted. Requiring users to normalize shell input was rejected because it causes the reported failure and conflicts with ordinary command-line expectations.

## Lexical normalization and containment

**Decision**: Normalize relative source input lexically, removing repeated separators and current-directory components and resolving parent components only while the result remains below the project root. Apply `.grip` exclusion to the normalized result.

**Rationale**: Lexical normalization produces a canonical declaration without opening paths or following links. Existing endpoint inspection remains responsible for filesystem safety, including symlink and topology checks. Rejecting any parent traversal that would escape the root makes containment deterministic before runtime resolution.

**Alternatives considered**: Filesystem canonicalization was rejected because it follows filesystem state and could alter the intended no-follow safety boundary. Passing raw source text to existing endpoint validation was rejected because declaration identity and lookup require one canonical form.

## Shared source-space command behavior

**Decision**: Route `add`, `list`, `remove`, and source-space selectors for `status`, `diff`, `push`, `pull`, and `sync` through the command-input parser. Leave destination-space selection on its existing destination parser.

**Rationale**: The current command layer already centralizes source-space selector resolution, so one parser provides consistent behavior with minimal change. An accepted source spelling should work for creation and later selection.

**Alternatives considered**: Limiting normalization to `add` was rejected because users could create a mapping with `./app/` but not select it with the same spelling. Applying source rules to destination selectors was rejected because destination syntax has its own explicit contract.

## Validation strategy

**Decision**: Add model tests for normalization and strict stored-declaration rejection, plus integration tests for add/persist/list/remove and source-space selectors. Preserve existing rejection coverage for escapes, `.grip`, non-UTF-8 values, and destination forms.

**Rationale**: The feature changes a command boundary but must prove storage, selection, and non-mutation guarantees. Isolated fixtures exercise those public contracts without touching real paths.

**Alternatives considered**: Parser-only tests were rejected because they would not prove that normalized values persist and select mappings consistently. A migration test is unnecessary because stored descriptors remain unchanged.
