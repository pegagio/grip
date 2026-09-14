---
title: Mapping addition and initial baselines
type: component
sources: [S004, S020, S022]
updated: 2026-09-14
---

# Mapping addition and initial baselines

`grip add` never copies endpoint payloads. For an equal compatible pair, it retains the ordinary accepted baseline. For an unequal source-defined member, it records the destination's complete supported state as the initial comparison reference, so existing classification presents the source as a normal pending push; a source-only member remains a pending addition and destination-only content remains unmanaged. (S020)

The policy applies independently to every source-defined, non-ignored member of a tree mapping. A later destination change before the first push remains a divergent conflict, so initial source authority does not become a permanent source-wins rule. (S020)

An unequal add creates and verifies a mapping-scoped incomplete-add fence before publishing its descriptor or matching State V4 comparison state. The fence binds the mapping identity and prior and candidate descriptor and state digests. A failure retains the fence for that mapping until a retry verifies completion or restores the prior descriptor; selected unfenced mappings remain available. (S020)

When exactly one active file mapping owns the requested resolved destination, `grip add --force SOURCE DESTINATION` deliberately replaces that mapping without copying either endpoint. It remains unavailable for source, tree, nested, and other ownership conflicts. After exact removal, an ordinary add may reuse the destination; removing a different mapping does not release the active owner. (S004)

The replacement transition retires only the displaced mapping's declared ownership and accepted comparison identities, then establishes the requested mapping's normal initial comparison state in the same recoverable descriptor/state publication. A visible interrupted transition may complete only its recorded candidate or restore its recorded prior pair. (S022)

## Related pages

- [Mappings and managed membership](./mappings-and-managed-membership.md)
- [Baselines, classification, and status](./baseline-classification-and-status.md)
- [Mapping registry publication](./mapping-registry-publication.md)
- [Force mapping replacement](./force-mapping-replacement.md)
