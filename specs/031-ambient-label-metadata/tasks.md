# Tasks: Ambient Label Metadata and Add Diagnostics

**Input**: [spec.md](./spec.md) and [plan.md](./plan.md)

## Phase 1: Metadata Policy Foundation

- [X] T001 Classify only nonempty `com.apple.metadata:kMDLabel_` suffix names as excluded in `src/metadata/mod.rs`
- [X] T002 Add xattr-policy coverage in `tests/metadata_filesystem_integration.rs`

## Phase 2: User Story 1 - Admit Ambient Labels (Priority: P1)

- [X] T003 [US1] Cover successful add with an excluded ambient label and retained blocking unknown attributes in `tests/mapping_cli_contract.rs`

## Phase 3: User Story 2 - Explain a Rejected Add (Priority: P2)

- [X] T004 [US2] Render concise baseline-add blocker details in `src/result.rs`
- [X] T005 [US2] Cover human unknown-xattr add diagnostics without attribute values in `tests/mapping_cli_contract.rs`

## Phase 4: Documentation and Validation

- [X] T006 Document ambient label exclusion and failed-add diagnostics in `README.md` and `docs/product-definition.md`
- [X] T007 Run focused and complete validation plus cross-artifact analysis
