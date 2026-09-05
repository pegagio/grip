# Gripignore Contract

This contract defines the initial normative `.gripignore` behavior. Only source-side files with this exact basename are policy; they are never managed payload.

## Policy discovery

Before evaluating a traversable source directory's children, Grip checks that directory for one exact `.gripignore` entry without following it. An existing policy entry must be a readable ordinary regular UTF-8 file. A symlink, directory, hard-linked file, sparse file, special node, unreadable file, invalid UTF-8 sequence, or parse error prevents a complete inventory.

The mapping root policy applies to the complete tree. A nested policy applies to its containing directory and descendants. Policies beneath a directory excluded from traversal are not discovered or read.

## Line and pattern behavior

Grip supports the Gitignore-compatible behavior required by the specification:

- blank lines match nothing;
- an unescaped leading `#` begins a comment;
- backslash escapes a leading `#` or `!` and preserves otherwise significant trailing spaces;
- unescaped trailing spaces are ignored;
- a leading slash anchors a pattern to the directory containing the policy;
- a pattern containing a slash other than a trailing slash is relative to that policy directory, while a pattern without another slash may match at any level beneath the policy scope;
- a trailing slash matches directories and their contents, not ordinary files or symbolic links;
- `*`, `?`, bracket ranges, and the documented leading, trailing, and infix `**` forms retain Gitignore meanings;
- an unescaped leading `!` negates an earlier exclusion;
- within one policy, the last matching rule wins;
- among applicable policies, the deepest policy with a non-neutral match wins;
- a descendant cannot be re-included when an ancestor directory itself was excluded and therefore was not traversed.

A UTF-8 BOM is stripped only at the start of the first line. CRLF input and a final line without a newline are accepted. Unclosed character classes retain the selected pattern engine's documented Git-compatible behavior and are covered by the fixed corpus.

## Authority exclusions

Discovery must not consult or infer rules from:

- `.gitignore`;
- `.ignore`;
- repository `.git/info/exclude`;
- user or system global ignore configuration;
- parent directories outside the mapping root;
- destination-side `.gripignore`;
- hidden-file status.

Hidden source entries are eligible unless `.gripignore` excludes them.

## Policy-only exclusion

Every source-side `.gripignore` is policy-only and does not appear as `eligible` or `ignored` payload. This structural rule runs before pattern evaluation, so `!.gripignore`, `**/.gripignore`, or another pattern cannot opt the policy file into payload.

Feature 003 does not synchronize policy or infer retirement from a newly ignored path. A later change that makes policy synchronizable requires a new feature specification.

## Conformance boundary

The fixed conformance corpus must cover comments, escaping, spaces, anchors, directory-only patterns, all wildcard forms, bracket ranges, precedence, nested overrides, valid negation, excluded-parent pruning, BOM, CRLF, and final unterminated lines. Authority-poison fixtures must configure every excluded ignore source to hide a sentinel that Grip still discovers. Policy-only fixtures must prove that negation never emits `.gripignore` as payload.

The selected matching library is an implementation aid, not the authority for silently expanding or narrowing this contract. A library upgrade must rerun the complete corpus.
