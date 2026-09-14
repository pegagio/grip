---
title: Large-file observation performance
type: component
sources: [S021]
updated: 2026-09-14
---

# Large-file observation performance

For a normal file mapping represented by an eligible discovery record, one inspection pass reuses that record's complete source and destination observation instead of independently observing the same endpoints in the file-mapping pre-pass. The reuse is limited to that same pass; file mappings without a discovery record retain direct observation. (S021)

The optimization retains complete descriptor-bound SHA-256 and supported-metadata evidence for each retained observation, two independent stable-observation passes, and `grip add`'s fenced post-publication reinspection. It does not introduce persistent fingerprints, caching, indexes, parallelism, locks, watchers, daemons, or changed baseline semantics. (S021)

The representative isolated local macOS ARM64 release workload uses differing 19 MiB regular source and destination files. It runs `grip add` and JSON `grip status` for 100 warm samples each and enforces a one-second p95 threshold; final measured p95 values were 573.669375 ms for `grip add` and 184.788375 ms for `grip status`. (S021)

## Related pages

- [Local development workflows](./local-development-workflows.md)
- [Mapping addition and initial baselines](./mapping-addition-and-initial-baselines.md)
