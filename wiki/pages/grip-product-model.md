---
title: Grip product model
type: concept
sources: [S001]
updated: 2026-09-10
---

# Grip product model

Grip is a local, selective, bidirectional deployment synchronizer. Explicit file or tree mappings define what it manages; destination-only tree content remains unmanaged. Baseline evidence enables detection of safe one-sided changes and explicit handling of conflicts. (S001)

The public CLI is flat: `init`, `version`, `add`, `list`, `remove`, `status`, `diff`, `push`, `pull`, and `sync`. Normal operations are conservative. `push -f` and `pull -f` make an explicit directional choice for one managed entry, including intentional absence propagation. Git remains responsible for history and recovery. (S001)
