# Contract: Project Resolution and Path Binding

## Explicit Selection

Given `--project PATH`, Grip resolves relative input against the invocation directory without following a final symlink, validates the existing directory, and requires `.grip/config.toml` directly beneath it. The input must be the exact project root. A descendant, metadata directory, descriptor path, missing path, regular file, symlink, unsafe directory, or invalid descriptor fails without ancestor discovery or global fallback.

## Implicit Discovery

Grip obtains the invocation directory and inspects it and every ancestor through the filesystem root, retaining non-following identity evidence for each boundary.

An ancestor containing `.grip` is a candidate. Candidate metadata must contain safe, current-user-owned, non-symlink `.grip`, `config.toml`, and `.gitignore` nodes; the descriptor must be valid V2 and the ignore file must have canonical bytes. An unsafe, partial, malformed, or unsupported candidate blocks rather than being skipped.

| Candidate result | Outcome |
|---|---|
| No candidate | `project_not_found` |
| One valid candidate | Select that project. |
| More than one valid candidate | `ambiguous_project` |
| Any invalid candidate boundary | `invalid_project_metadata` or `unsupported_schema` |

Grip does not stop at the nearest candidate. A deleted or renamed invocation directory, inaccessible ancestor, or identity substitution produces an operational or changed-project failure.

## Context Binding

Selection produces one `ProjectContext`. The descriptor is decoded, every portable mapping is resolved beneath the context roots, and the complete ownership graph is validated before project data access.

Before project-owned or payload mutation, Grip revalidates:

1. Project root, `.grip`, descriptor, and ignore-file identities and safety.
2. Exact accepted descriptor bytes and digest.
3. Canonical invoking-user home identity.
4. Each selected mapping's portable identity, endpoint ancestry, topology, and node evidence.
5. Accepted state, operation, and recovery binding needed by the command.

Any drift invalidates the plan. No newly discovered project replaces the retained context.

## Home Binding

The existing trusted platform lookup determines the invoking user's home; `GRIP_HOME` is not consulted. The home must resolve to a safe canonical absolute ordinary directory. Destination `~` resolves to that directory and `~/PATH` beneath it. Failure or change blocks resolution and mutation.

## Nested Boundaries

Ordinary `grip init` rejects a target beneath an existing project. If multiple valid projects nevertheless appear through manual changes, implicit discovery fails as ambiguous. An invalid inner boundary is not crossed to select a valid outer project.

## Independent Commands

Clap-provided help/version and the application `version` command do not inspect the invocation directory, ancestors, home, `.grip`, or legacy global paths.
