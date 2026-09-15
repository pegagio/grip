# Quickstart: Forced Missing-Peer Restoration

Use isolated temporary source and destination trees. Do not run these destructive endpoint scenarios against a working project without a backup strategy.

## Restore a missing destination

1. Create an accepted tree mapping with at least two files and run `grip push` once.
2. Remove one destination file while leaving the corresponding source file present.
3. Confirm `grip status` identifies the selected member as a one-sided missing-peer condition.
4. Run `grip push --dry-run --force app/selected-file` and verify it reports one source-to-destination action without changing either endpoint.
5. Run `grip push --force app/selected-file`.
6. Confirm the selected destination file matches the source, the other mapped entry is unchanged, and `grip status` reports the restored entry as current.

## Restore a missing source

1. Create an accepted tree mapping with at least two files and run `grip push` once.
2. Remove one source file while leaving the corresponding destination file present.
3. Run `grip pull --dry-run --force --destination ../destination/app/selected-file` and verify it reports one destination-to-source action without changing either endpoint.
4. Run `grip pull --force --destination ../destination/app/selected-file`.
5. Confirm the selected source file matches the destination, the other mapped entry is unchanged, and `grip status` reports the restored entry as current.

## Preserve safety boundaries

1. Confirm forced push and pull reject an omitted selector, a mapping root, and a subtree containing more than one managed entry.
2. Confirm a selected absent winner continues to remove its present peer through the existing forced deletion behavior.
3. Run the focused force tests followed by `mise run validate`.
