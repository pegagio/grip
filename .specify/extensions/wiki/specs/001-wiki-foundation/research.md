# Research: Wiki Foundation

## Decision 1: Preserve the prompt-defined extension architecture

**Decision**: Implement the feature contract in the existing Markdown command, YAML configuration template, and extension manifest.

**Rationale**: The extension intentionally has no executable runtime or external dependency. Keeping initialization declarative preserves compatibility across Spec Kit AI integrations and matches every other wiki command in the repository.

**Alternatives considered**:

- Add a shell or Python initializer: rejected because it would introduce platform and packaging dependencies without adding user value.
- Add a shared command library: rejected because there is no runtime module system and only one initialization operation owns these writes.

## Decision 2: Treat the schema as the initialization sentinel

**Decision**: Continue using the resolved wiki schema’s existence as the sole initialized-state sentinel.

**Rationale**: The schema is the user-editable authority for wiki structure and workflows. Its presence signals that maintainers may already own the directory’s contents; silently filling missing artifacts would conflict with the preservation guarantee.

**Alternatives considered**:

- Require all three foundation artifacts before considering the wiki initialized: rejected because it could overwrite or reconstruct deliberately edited partial state.
- Add a separate machine-owned marker: rejected because it duplicates state and creates disagreement risk.

## Decision 3: Enforce canonical project containment before writes

**Decision**: Resolve the configured wiki directory against the project root, normalize both paths, and stop if the result is not the root itself or a descendant of it.

**Rationale**: The wiki is defined as repository-owned state. A relative value containing traversal segments must not turn initialization into an arbitrary filesystem write.

**Alternatives considered**:

- Trust every configured relative value: rejected because `../` can escape the project boundary.
- Restrict the directory to the literal default `wiki/`: rejected because supported project customization is a documented capability.

## Decision 4: Keep deterministic configuration precedence

**Decision**: Resolve settings in ascending authority from extension defaults to saved project configuration, environment overrides, and invocation overrides.

**Rationale**: This ordering supports stable project defaults, automation-specific adjustments, and explicit one-run intent without ambiguity.

**Alternatives considered**:

- Let saved configuration override invocation input: rejected because explicit user intent would become ineffective.
- Merge all sources without a fixed order: rejected because the same inputs could yield inconsistent state.
