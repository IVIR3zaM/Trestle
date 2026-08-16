# DEFERRED

Consciously postponed. Distinct from forgotten — every entry says why, and what
would change the answer.

| Item | Why deferred | Revisit when |
|---|---|---|
| Adopt upstream's test runner | Migration touches every test file; unrelated to convergence and would dominate the diff. | After the convergence effort closes. |
| Contribute the local caching layer | Genuinely useful upstream, but it depends on a local-only config key that would have to be generalised first. | If upstream adds a plugin point for it. |
| Delete the compatibility shim | Two downstream consumers still import it. | Once both have migrated. |
