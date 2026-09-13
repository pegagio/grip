---
title: Mapping addition and initial baselines
type: component
sources: [S020]
updated: 2026-09-13
---

# Mapping addition and initial baselines

`grip add` never copies endpoint payloads. For an equal compatible pair, it retains the ordinary accepted baseline. For an unequal source-defined member, it records the destination's complete supported state as the initial comparison reference, so existing classification presents the source as a normal pending push; a source-only member remains a pending addition and destination-only content remains unmanaged. (S020)

The policy applies independently to every source-defined, non-ignored member of a tree mapping. A later destination change before the first push remains a divergent conflict, so initial source authority does not become a permanent source-wins rule. (S020)

An unequal add creates and verifies a mapping-scoped incomplete-add fence before publishing its descriptor or matching State V4 comparison state. The fence binds the mapping identity and prior and candidate descriptor and state digests. A failure retains the fence for that mapping until a retry verifies completion or restores the prior descriptor; selected unfenced mappings remain available. (S020)

## Related pages

- [Mappings and managed membership](./mappings-and-managed-membership.md)
- [Baselines, classification, and status](./baseline-classification-and-status.md)
- [Mapping registry publication](./mapping-registry-publication.md)
