# Data Model: Wiki Health Checks

## Lint Scope

Full wiki, one named check, or one page plus directly linked/shared-source neighbors.

## Finding

| Field | Rule |
|---|---|
| Check | One of six declared check names |
| Severity | Mechanical, structural, or semantic |
| Page | Exact affected filename or structural artifact |
| Evidence | Verifiable claim, marker, link, citation, date, or index line |
| Suggested action | Concrete and user-controlled |
| Disposition | Applied, suggested, or unresolved |

## Mechanical Fix

Allowlisted index regeneration or one unambiguous link-target change, with before and after values.

## Lint Report

Current-run date, deterministic finding rows, and No findings when empty. It replaces the previous derived report.

## Next Action

Exactly one unresolved finding action, prioritized semantic then structural then mechanical and by expected value within severity.

## Lifecycle

```text
scope -> analyze -> findings -> prepare allowlisted fixes -> validate set -> apply -> write accurate report -> recommend one action
```
