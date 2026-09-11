# Destination Path Contract

This contract defines destination-valued input accepted by `grip add` and destination-space selectors after Feature 014.

## Accepted forms

| Form | Example | Stored declaration | Operational interpretation |
|---|---|---|---|
| Absolute | `/opt/grip-dst/app` | Exact input | The same absolute path. |
| Home | `~` | Exact input | The selected home directory. |
| Home-relative | `~/deploy/../app` | Exact input | Selected-home expansion followed by lexical normalization. |
| Project-relative | `grip-dst/app` | Exact input | Selected-project-root expansion followed by lexical normalization. |
| Current-directory spelling | `./grip-dst//app` | Exact input | Interpreted from the selected project, not the process current directory. |
| Parent-directory spelling | `../grip-dst/app/` | Exact input | Interpreted from the selected project and may resolve outside it. |

## Rejected forms

The following inputs must fail before mapping publication and must leave the descriptor unchanged.

| Form | Example | Required result |
|---|---|---|
| Empty | `` | Reject as an invalid destination declaration. |
| Other-user home | `~someone/app` | Reject as an unsupported home form. |
| Environment expansion | `$HOME/app` | Reject as an unsupported expansion-like form. |
| Non-UTF-8 | Platform-specific non-text path input | Reject as an invalid portable declaration. |

## Resolution and safety

Grip retains the exact accepted destination spelling in mapping intent. For every validation, selection, state, or synchronization operation, it derives a separate absolute operational path from the selected project root or selected home as defined above. It lexically normalizes that operational path without filesystem canonicalization or symbolic-link following.

Existing endpoint-kind, ownership, topology, reserved-metadata, drift, and baseline validation applies to the resolved endpoint. Equivalent resolved destinations conflict even when their declarations have different lexical spellings. A relative declaration must resolve identically for the same selected project regardless of the command’s current directory.

## Compatibility

Existing absolute, `~`, and `~/` declarations remain valid and retain their stored spelling. Existing source declarations remain project-relative and strict. No migration, command alias, remote path syntax, or current-working-directory-relative destination mode is introduced.
