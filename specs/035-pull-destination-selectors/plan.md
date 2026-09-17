# Implementation Plan

Replace pull's destination opt-in with destination-space default selection and a source-space opt-in. Reuse the existing destination-winning force path with the inverted selector flag, update emitted guidance, and cover parser, selection, permission restoration, and documentation behavior.

The change remains inside the existing CLI, selection, mutation-plan, and renderer boundaries. No persistent state or new synchronization authority is introduced.
