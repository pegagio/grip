# Data Model: Destination Path Forms

## Destination Declaration

This value is the user-authored destination stored in a mapping declaration.

| Attribute | Rules |
|---|---|
| `spelling` | Valid UTF-8 text preserved exactly as accepted. |
| `form` | One of absolute, home, or home-relative. |
| Absolute form | Begins at the filesystem root. |
| Home form | Is exactly `~`. |
| Home-relative form | Begins with `~/`; every lexical suffix is accepted, including `.`, `..`, and repeated separators. |
| Invalid form | Empty, relative, other-user tilde, environment-variable, or non-UTF-8 input. |

## Portable Mapping

| Attribute | Rules |
|---|---|
| `kind` | Existing file or tree mapping kind. |
| `source` | Existing normalized project-relative source declaration. |
| `destination` | Destination Declaration, retained exactly in project mapping intent. |
| Identity | Includes declared source, kind, and preserved destination spelling for declaration-level lookup and persistence. |

## Resolved Mapping

This runtime-only value pairs a Portable Mapping with inspected source and destination endpoints.

| Attribute | Rules |
|---|---|
| `source` | Resolved beneath the selected project root using existing source rules. |
| `destination` | Absolute path directly, or a home-expanded lexical operational path for `~` and `~/` declarations. |
| Operational normalization | Removes lexical `.` and `..` components without filesystem canonicalization or symlink following. |
| Ownership identity | Uses resolved endpoints so declarations that resolve to overlapping namespaces still conflict. |

## Lifecycle

1. The operator submits `grip add SOURCE DESTINATION`.
2. Grip validates source as project-relative and destination as an authorized declaration form.
3. Grip preserves the destination declaration and derives a resolved operational path for endpoint and topology checks.
4. On successful validation, Grip publishes the original declarations and records existing runtime evidence/state bindings.
5. Later commands reload the declaration unchanged, resolve it using current command context, and apply ordinary safety checks.
6. `remove` removes the matching declaration and related active evidence without rewriting unrelated destination spellings.
