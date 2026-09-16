# Quickstart: Operational Output and State Rebinding

1. Configure a recording diff tool and run `grip diff PATH`; confirm standard output has no Grip header or footer.
2. Change managed metadata and run `grip diff -v PATH`; confirm standard error contains property-oriented source, baseline, and destination values.
3. Push a tree mapping with nested files; confirm the count equals copied files rather than created directories.
4. Block a no-selector `grip push --force`; confirm its transcript identifies blockers and managed path evidence.
5. Establish a baseline, change source content, append a valid diff profile, and run `grip push`; confirm JSON status returns `bound`.
