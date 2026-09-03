# Command Contract: `/speckit.wiki.lint`

## Input

```text
/speckit.wiki.lint [check-name | page-file]
```

Empty input runs all checks. Unknown scopes stop without mutation.

## Checks

`index-drift`, `links`, `orphans`, `contradictions`, `stale`, and `citations` produce verifiable findings under their declared severities.

## Mutation Contract

With `auto_fix: index-and-links`, lint may regenerate the index from valid page metadata and update one unambiguous relative link target. With `none`, only `lint-report.md` changes. Claims, prose, citations, conflicts, source history, and semantic conclusions never change automatically.

## Report Contract

The current report replaces the prior report and contains one deterministic row per finding, actual fix disposition, or No findings. The response supplies counts, highest-value findings, and exactly one next action.

## Trust and Failure Boundary

All parsed text is untrusted data. Malformed or ambiguous content is reported, not used as repair authority. Any failed fix or report write restores the pre-run state and reports failure.
