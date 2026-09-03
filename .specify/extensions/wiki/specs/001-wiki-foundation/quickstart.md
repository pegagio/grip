# Quickstart: Validate Wiki Foundation

This guide validates initialization in disposable projects so existing repository knowledge is never placed at risk.

## Prerequisites

- A local checkout of this extension.
- Spec Kit compatible with the version declared in `extension.yml`.
- A supported AI integration in which the extension commands are available.
- A disposable initialized Spec Kit project for each scenario.

## Scenario 1: Default Initialization

1. Install this extension into a disposable project using the local development-install workflow.
2. Run `/speckit.wiki.init Project decisions and constraints`.
3. Verify that `wiki/SCHEMA.md`, `wiki/INDEX.md`, and `wiki/sources.md` exist.
4. Verify that no `wiki/pages/` directory exists.
5. Verify that the schema contains the supplied scope and documented default policies.
6. Verify that the index has no page entries and the registry has no source rows.

**Expected outcome**: Initialization reports the three artifacts, effective scope and limits, and concrete ingest and status next actions.

## Scenario 2: Repeat Without Data Loss

1. Add recognizable user-authored text to the existing schema, index, and registry.
2. Record checksums for all three artifacts.
3. Run `/speckit.wiki.init` again.
4. Compare the artifacts with their recorded checksums.

**Expected outcome**: All checksums remain unchanged, and the response summarizes existing wiki status.

## Scenario 3: Append Scope

1. Start with an initialized wiki containing one scope item.
2. Run `/speckit.wiki.init Operational lessons`.
3. Inspect the schema and all other wiki artifacts.

**Expected outcome**: The schema contains a second numbered scope item with the supplied text; every other existing byte remains unchanged.

## Scenario 4: Configuration Precedence

1. Configure one wiki directory in saved project settings.
2. Provide a different directory through the documented environment override.
3. Invoke initialization with a third directory as an invocation override.
4. Repeat while removing the higher-priority sources one at a time.

**Expected outcome**: The invocation value wins first, then the environment value, then the saved value, and finally the extension default.

## Scenario 5: Reject Path Escape

1. In a disposable project, invoke initialization with a directory that resolves above the project root, such as `directory=../outside-wiki`.
2. Inspect both the project and the proposed outside target.

**Expected outcome**: Initialization stops with a clear containment error, creates no foundation artifact, and changes no outside file.

## Static Validation

After editing the extension source, verify:

```bash
git diff --check
```

Review the command against [the command contract](./contracts/wiki-init-command.md) and confirm the extension manifest still registers exactly one initialization command using the documented configuration template.
