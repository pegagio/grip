# Quickstart: Relative Destination Paths

Use this guide to validate the Feature 014 behavior in an isolated temporary filesystem. The declaration and resolution rules are defined in [the destination-path contract](contracts/destination-paths.md) and the data flow is described in [the data model](data-model.md).

## Prerequisites

Run these commands from the repository root after the implementation is complete.

```sh
mise trust
mise install
mise run validate
```

The complete validation command must finish successfully before manual scenarios are considered passed.

## Relative declaration scenario

Create a temporary project with an eligible `app` source directory and an adjacent destination directory. Initialize the project, then run:

```sh
grip add ./app/ ../grip-dst/app/
grip list
```

Expected results:

- `add` succeeds without converting the destination to an absolute or home-relative form.
- `list` shows the declared `../grip-dst/app/` spelling.
- The project descriptor contains the same declared spelling.

## Stable-base scenario

From a descendant of the initialized project, run a read-only command that selects the mapping in destination space using its project-relative spelling. Repeat with an explicit project selector from another working directory.

Expected results:

- Both commands select the same resolved destination endpoint.
- Neither command interprets the destination from its process current directory.
- A lexical equivalent such as `../grip-dst/./app/` resolves to the same endpoint and remains subject to ordinary ownership checks.

## Portability and compatibility scenarios

Copy the project and its adjacent destination layout to a different parent directory, then inspect the mapping in the copy. Separately add and inspect mappings using an absolute path, `~`, and a non-normalized `~/` spelling.

Expected results:

- The copied project uses the unchanged relative declaration against its new project root.
- Existing absolute and home-relative declarations retain their previous behavior and exact stored spelling.
- Empty, other-user-home, expansion-like, and non-text destination inputs fail without changing the descriptor.

## Safety regression scenarios

Attempt a mapping whose relative destination resolves to its source or overlaps another resolved mapping. Remove an unrelated mapping while at least one relative destination remains.

Expected results:

- Existing equal-endpoint, recursive-topology, and ownership failures still block unsafe mappings.
- Removing a mapping leaves every retained relative, absolute, and home-relative declaration unchanged in the descriptor.
