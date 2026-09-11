# Status Presentation Model

This feature adds no persisted state and does not change the classification data model. It derives a transient human presentation from the existing classification records after inspection has completed.

## Status summary

| Field | Source | Rule |
| --- | --- | --- |
| Entries checked | Status records | Total selected records. |
| Current | Status records | Records with `attention` false. These require no operator action in the selected scope. |
| Changes to push | Status records | Records with `attention` true, `blocking` false, and source-to-destination direction. |
| Changes to pull | Status records | Records with `attention` true, `blocking` false, and destination-to-source direction. |
| Conflicts | Status records | Records with `blocking` true. |
| Needs baseline | Status records | Remaining non-blocking attention records with no payload-copy direction, including baseline establishment, refresh, deletion convergence, and metadata-migration readiness. |

The displayed categories are mutually exclusive and their counts sum to entries checked. A clean scope has zero actionable categories. An empty scope reports no managed entries rather than a clean count.

## Actionable status entry

| Field | Source | Validation and presentation rule |
| --- | --- | --- |
| Group | `attention`, `blocking`, and prospective direction | Conflicts, changes to push, changes to pull, or needs baseline; current entries are omitted from itemized output. |
| Source and destination paths | Existing safe display paths | Both paths appear on every actionable row. They are display evidence, not a generated command selector. |
| Symbol | Group | `->` push, `<-` pull, `<->` conflict, or `>-<` non-directional reconciliation attention. |
| Description | Blocking compatibility finding, when present | Must be concise plain language and must not expose raw enum names, reason identifiers, comparison dimensions, or capability-profile values. |
| Order | Existing identity-sorted records | Render only nonempty sections and preserve source order within each section. |

## Compatibility finding treatment

| Finding type | Default human status behavior |
| --- | --- |
| Informational, non-blocking finding | Omit it. |
| Blocking finding | Include the affected entry, a plain-language explanation, and the existing corrective guidance when it is safe to show. |

JSON serialization keeps every existing classification and compatibility field, including informational findings. The presentation model is not serialized or persisted.
