---
title: Mapping registry publication
type: component
sources: [S005]
updated: 2026-09-04
---

# Mapping registry publication

Feature 002 stores accepted mapping intent in `<GRIP_HOME>/config.toml` as schema version 1 with deterministically ordered `kind`, `source`, and `destination` tuples. Mapping commands never treat staging files or recovery generations as accepted configuration and do not create synchronization state. (S005)

Readers require the accepted registry to be a current-user-owned, non-symlink regular file without group or other write permission. Writers additionally require owner write permission and preserve the accepted file's exact safe mode. Grip-owned lock and staging files use mode `0600`; recovery directories use `0700`. (S005)

A writer follows one bounded publication sequence: (1) load and validate the complete accepted registry, (2) retain its exact bytes and path evidence, (3) build and validate the complete candidate, (4) acquire the stable `.registry.lock`, (5) reread and revalidate accepted bytes and exact submitted-path evidence, (6) publish and verify the prior bytes under `state/recovery/registry/sha256-<digest>/config.toml`, (7) exclusively create, write, sync, reread, semantically verify, and revalidate an attempt-owned same-directory staging file, and (8) rename it over `config.toml` and sync the Grip-home directory where supported. (S005)

The rename is the acceptance point. An earlier failure leaves the old registry authoritative and may leave an immutable verified recovery generation. A post-rename directory-sync failure is an operational failure with the new document visible and the previous document recoverable. Cleanup removes a failed staging path only while its descriptor-backed identity still proves it belongs to that attempt; unexpected staging nodes are never accepted or deleted. (S005)

Recovery generations preserve prior registry documents, not payload backups or synchronization baselines. Restoration, listing, retention, and cleanup policy remain outside Feature 002. (S005)

## Related pages

- [Configuration and state](./configuration-and-state.md)
- [Mappings and managed membership](./mappings-and-managed-membership.md)
- [Safety and recovery model](./safety-and-recovery-model.md)
