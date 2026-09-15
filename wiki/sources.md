# Source Registry

Append-only IDs (S001, S002…). Dedup key: normalized path or URL.
Sources are immutable inputs — the wiki never edits them.

| ID | Source | Type | First ingested | Last ingested | Pages touched |
|----|--------|------|----------------|---------------|---------------|
| S001 | `docs/product-definition.md` | file | 2026-09-03 | 2026-09-15 | `baseline-classification-and-status.md`, `mappings-and-managed-membership.md`, `synchronization-and-conflicts.md` |
| S002 | `.specify/memory/constitution.md` | file | 2026-09-03 | 2026-09-03 | `proportional-engineering-rigor.md`, `spec-evolution-and-merge-bounded-persistence.md`, `safety-and-recovery-model.md`, `implementation-and-delivery-direction.md` |
| S003 | `.specify/memory/roadmap.md` | file | 2026-09-03 | 2026-09-15 | `initial-delivery-roadmap-later-features.md`, `initial-delivery-roadmap-recent-features.md`, `initial-delivery-roadmap.md` |
| S004 | `README.md` | file | 2026-09-03 | 2026-09-15 | `local-development-workflows.md`, `mappings-and-managed-membership.md` |
| S005 | `specs/002-mapping-registry-ownership` | directory | 2026-09-04 | 2026-09-04 | `mappings-and-managed-membership.md`, `safety-and-recovery-model.md`, `filesystem-support-boundaries.md`, `proportional-engineering-rigor.md`, `spec-evolution-and-merge-bounded-persistence.md`, `mapping-registry-publication.md`, `configuration-and-state.md` |
| S006 | `specs/003-source-discovery-gripignore` | directory | 2026-09-05 | 2026-09-05 | `command-line-and-path-selection.md`, `mappings-and-managed-membership.md`, `filesystem-support-boundaries.md`, `implementation-and-delivery-direction.md`, `proportional-engineering-rigor.md`, `spec-evolution-and-merge-bounded-persistence.md` |
| S007 | `specs/004-baselines-classification-status` | directory | 2026-09-06 | 2026-09-06 | `baseline-classification-and-status.md`, `synchronization-and-conflicts.md`, `configuration-and-state.md`, `command-line-and-path-selection.md`, `safety-and-recovery-model.md` |
| S008 | `specs/005-safe-push-recovery` | directory | 2026-09-06 | 2026-09-06 | `command-line-and-path-selection.md`, `configuration-and-state.md`, `filesystem-support-boundaries.md`, `implementation-and-delivery-direction.md`, `local-development-workflows.md`, `safety-and-recovery-model.md`, `spec-evolution-and-merge-bounded-persistence.md`, `synchronization-and-conflicts.md` |
| S009 | `specs/006-reverse-synchronization` | directory | 2026-09-07 | 2026-09-07 | `command-line-and-path-selection.md`, `configuration-and-state.md`, `filesystem-support-boundaries.md`, `implementation-and-delivery-direction.md`, `local-development-workflows.md`, `safety-and-recovery-model.md`, `spec-evolution-and-merge-bounded-persistence.md`, `synchronization-and-conflicts.md` |
| S010 | `specs/007-bidirectional-sync-conflicts` | directory | 2026-09-07 | 2026-09-07 | `command-line-and-path-selection.md`, `configuration-and-state.md`, `local-development-workflows.md`, `safety-and-recovery-model.md`, `synchronization-and-conflicts.md` |
| S011 | `specs/008-authorized-deletion-retirement` | directory | 2026-09-07 | 2026-09-07 | `configuration-and-state.md`, `deletion-retirement-and-recovery.md`, `local-development-workflows.md`, `mappings-and-managed-membership.md`, `safety-and-recovery-model.md`, `synchronization-and-conflicts.md` |
| S012 | `specs/009-metadata-filesystem-contract` | directory | 2026-09-08 | 2026-09-08 | `metadata-and-filesystem-contract.md`, `filesystem-support-boundaries.md`, `baseline-classification-and-status.md`, `synchronization-and-conflicts.md`, `local-development-workflows.md`, `deletion-retirement-and-recovery.md` |
| S013 | `specs/010-project-scoped-initialization` | directory | 2026-09-09 | 2026-09-11 | `configuration-and-state.md`, `implementation-and-delivery-direction.md`, `local-development-workflows.md`, `mapping-registry-publication.md` |
| S014 | `specs/011-git-command-hierarchy` | directory | 2026-09-10 | 2026-09-10 | `git-inspired-command-hierarchy.md`, `command-line-and-path-selection.md`, `implementation-and-delivery-direction.md` |
| S015 | `specs/012-destination-path-forms` | directory | 2026-09-11 | 2026-09-11 | `destination-path-forms.md`, `command-line-and-path-selection.md`, `mappings-and-managed-membership.md`, `configuration-and-state.md` |
| S016 | `specs/013-source-path-input` | directory | 2026-09-11 | 2026-09-13 | `mapping-registry-publication.md` |
| S017 | `specs/014-relative-destination-paths` | directory | 2026-09-11 | 2026-09-11 | `command-line-and-path-selection.md`, `destination-path-forms.md`, `mappings-and-managed-membership.md`, `configuration-and-state.md`, `mapping-registry-publication.md` |
| S018 | `specs/015-simplify-status-output` | directory | 2026-09-11 | 2026-09-11 | `baseline-classification-and-status.md` |
| S019 | `specs/017-simplify-command-output` | directory | 2026-09-13 | 2026-09-13 | `baseline-classification-and-status.md` |
| S020 | `specs/018-source-authoritative-add` | directory | 2026-09-13 | 2026-09-13 | `baseline-classification-and-status.md`, `mappings-and-managed-membership.md`, `mapping-addition-and-initial-baselines.md` |
| S021 | `specs/021-large-file-performance` | directory | 2026-09-14 | 2026-09-14 | `large-file-observation-performance.md`, `local-development-workflows.md` |
| S022 | `specs/022-force-mapping-replacement` | directory | 2026-09-14 | 2026-09-14 | `force-mapping-replacement.md`, `mapping-addition-and-initial-baselines.md`, `mapping-registry-publication.md` |
| S023 | `specs/023-force-missing-peer-restoration` | directory | 2026-09-14 | 2026-09-14 | `synchronization-and-conflicts.md` |
| S024 | `specs/024-local-artifact-release-automation` | directory | 2026-09-15 | 2026-09-15 | `local-development-workflows.md`, `local-artifact-release-preparation.md` |
| S025 | `specs/025-contained-source-tree-mappings` | directory | 2026-09-15 | 2026-09-15 | `contained-source-tree-mappings.md`, `mappings-and-managed-membership.md` |
