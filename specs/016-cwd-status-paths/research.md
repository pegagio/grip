# Research: Current-Directory Status Paths

## Preserve machine-readable status data

**Decision**: Keep the existing classification records and JSON status payload unchanged. Construct a non-serialized human-status projection from the raw source paths and the invocation working directory.

**Rationale**: JSON is a compatibility surface, while `SafePath.display` is a canonical absolute representation intended for structured output and diagnostics. Reconstructing a path from that display string would mishandle non-UTF-8 paths and would couple the human renderer to a serialized representation.

**Alternatives considered**:

- Rewrite `source_path` in the classification record before rendering: rejected because it changes JSON output.
- Derive paths from `SafePath.display` in the renderer: rejected because display text is not the authoritative path value.

## Use Git-style human path presentation

**Decision**: Render ordinary source paths relative to the process current working directory. Preserve `..` segments when the source is outside that directory. Quote unusual names in Git-style C notation where needed, without promising a universally shell-safe token.

**Rationale**: This makes the status output readable from a nested directory and supports the ordinary copy-and-use `grip push PATH` workflow. Git-style display remains familiar while avoiding a misleading claim that every output string can be pasted into every shell unchanged.

**Alternatives considered**:

- Always show project-relative paths: rejected because a user running from a nested directory would need to translate the displayed path before using it.
- Emit shell-specific escaping: rejected because quoting rules vary by shell and are outside the status command's display contract.

## Make source-side push selectors CWD-relative

**Decision**: Resolve relative source selectors supplied to `push`, including force and dry-run variants, relative to the invocation current working directory. Preserve the existing rejection of absolute source selectors. Destination selectors and every non-push command retain their current portable project-relative behavior.

**Rationale**: A normal source path printed by `status` can then be supplied directly to `push` without expanding the existing source-selector surface. Treating force and dry-run push variants consistently prevents a surprising command-mode difference.

**Alternatives considered**:

- Change all source selector commands to CWD-relative: rejected because it expands the feature and breaks existing portable-selector contracts for status, diff, pull, sync, and other commands.
- Add a new explicit flag for CWD-relative push selectors: rejected because it would not provide direct status-to-push continuity.

## Enforce project containment before path inspection

**Decision**: Resolve a relative CWD-based push selector to a candidate source path, validate lexical and canonical containment in the selected project, and only then hand it to existing selection and mutation logic.

**Rationale**: The new resolution route must not allow a path outside the selected project to reach unmanaged-path inspection or mutation. A lexical check catches direct traversal; a bounded canonical check covers symlink traversal where the target exists without adding a recursive tree scan.

**Alternatives considered**:

- Rely on later selection validation: rejected because later helpers may inspect the candidate path before reporting that it is invalid.
- Check only lexical containment: rejected because a symlink can resolve outside the project.
