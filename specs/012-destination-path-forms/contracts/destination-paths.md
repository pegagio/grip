# Destination Path Contract

This contract defines every destination-valued input accepted by Feature 012, including `grip add SOURCE DESTINATION` and existing destination-space selectors.

## Accepted Forms

| Form | Example | Stored declaration | Operational interpretation |
|---|---|---|---|
| Absolute | `/opt/grip-dst/README.md` | Exact input text | The same absolute path. |
| Home | `~` | `~` | The invoking user's selected home directory. |
| Home-relative | `~/Working/../grip-dst//README.md` | Exact input text | Home-expanded then lexically normalized only for the operation. |

## Rejected Forms

| Form | Example | Required result |
|---|---|---|
| Relative | `grip-dst/README.md` | Reject before mapping publication; describe accepted absolute and home forms. |
| Current-directory relative | `./grip-dst/README.md` | Reject before mapping publication. |
| Parent-directory relative | `../grip-dst/README.md` | Reject before mapping publication. |
| Other-user tilde | `~someone/grip-dst` | Reject before mapping publication. |
| Environment expansion | `$HOME/grip-dst` | Reject before mapping publication. |

## Persistence and Resolution

Grip stores the accepted destination spelling exactly. It must not convert home-relative input to an absolute path or normalize lexical segments in the project descriptor. For each command that validates, selects, or operates on a mapping, Grip derives a separate lexical operational path and applies existing endpoint, ownership, topology, and revalidation rules to that path.

Equivalent resolved destinations remain equivalent for ownership checks. For example, a mapping to `~/a/../b` conflicts with an overlapping mapping to `~/b` even though their stored spellings differ.

## Compatibility

Existing `~` and normalized `~/...` declarations remain valid and retain their stored spelling. Relative source declarations remain governed by the existing project-root contract. No command aliases, source-form expansion, automatic migration, or remote destination forms are introduced.
