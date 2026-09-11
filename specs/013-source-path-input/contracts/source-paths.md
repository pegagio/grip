# Source Path Contract

This contract defines source-valued CLI inputs accepted by Feature 013. It applies to `grip add SOURCE DESTINATION`, `grip list [SOURCE]`, `grip remove SOURCE`, and source-space selectors for `status`, `diff`, `push`, `pull`, and `sync`.

## Accepted Forms

| Submitted source | Normalized declaration | Required result |
|---|---|---|
| `app` | `app` | Existing canonical source behavior. |
| `./app` | `app` | Accept and use `app`. |
| `./app/` | `app` | Accept and use `app`. |
| `nested/./app` | `nested/app` | Accept and use `nested/app`. |
| `nested/../app` | `app` | Accept when normalization remains in the project. |
| `.` | `.` | Accept only when the eventual mapping kind is tree. |

## Rejected Forms

| Submitted source | Required result |
|---|---|
| Empty input | Reject before mapping lookup or publication. |
| `/absolute/path` | Reject before mapping lookup or publication. |
| `../escape` | Reject because it escapes the project. |
| `nested/../../escape` | Reject because it escapes the project. |
| `.grip` or `.grip/config.toml` | Reject because Grip metadata is reserved. |
| `nested/../.grip` | Reject because the normalized result is reserved. |
| `$HOME/app`, `${HOME}/app`, or `~/app` | Reject as expansion-like source input. |
| Non-UTF-8 path | Reject before normalization. |

## Persistence and Selection

Grip persists the normalized declaration, never the submitted source spelling. `grip add ./app/ DESTINATION` and `grip add app DESTINATION` therefore propose the same source identity. All source-space selectors normalize their submitted spelling before selection. Destination-space selectors retain the existing destination-path contract and do not accept source-relative syntax.

## Descriptor Compatibility

The descriptor remains a strict canonical storage format. A stored source containing `./`, a trailing separator, repeated separator, or lexical parent component is invalid; Grip must not normalize or rewrite it during loading.
