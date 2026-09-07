# Domain Contract: Explicit Retirement

## Selection and eligibility

Retirement requires a path selector or `--all`. Complete registry validation remains mandatory. The plan evaluates retained accepted identities as follows:

| Classification/evidence | Without `--force` | With `--force` |
|---|---|---|
| `converged_deletion` | retire | retire |
| Pending retirement with absent or equivalent survivors | retire | retire |
| Pending retirement with differing survivors | block as `force_required` | retire |
| Active managed record | block | block |
| Unsupported, unsafe, or incomplete evidence | block | block |

Force authorizes only removal of comparison history after differences are reported. It never authorizes a payload change, unsafe access, incomplete inspection, unsupported node, or ownership violation.

## Execution

1. Resolve the exact path or explicit all scope and observe both sides of every retained identity without changing its ignored or untracked membership classification.
2. Build a deterministic state-only plan, reporting surviving differences and every blocker.
3. Preview, blocked, empty, and already-retired outcomes remain lock-free and publish nothing.
4. Execution acquires the mutation lock, revalidates registry and State V2, re-observes membership/policy/payload evidence, rebuilds the same plan, and initializes an operation record.
5. Clone accepted state, remove only the selected eligible identities, and publish one new generation through the existing state lock/publication boundary.
6. Report every retirement reason, force use, prior and authoritative generation, visibility, and durability.

Retirement never changes payload, mapping intent, ignore policy, recovery bytes, or operation history. If later policy admits a retired path, ordinary fresh classification applies; retired evidence is not implicitly revived.
