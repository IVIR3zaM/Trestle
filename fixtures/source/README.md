# Source fixtures

Two plans written in the **native styles people use today**, before Trestle
existed. They are the input corpus for the plan format (T02): the format is not
done until both can be expressed in it without losing anything.

They are deliberately *not* written in Trestle's format. That is the point — a
format proved only against examples designed for it has been proved against
nothing.

| Fixture | Shape | Why it's here |
|---|---|---|
| `graph-shape/` | dependency graph | index + per-unit files + oracles + gates |
| `loop-shape/` | iterative loop | goal + status queue + append-only journal |

Both describe plausible, generic software efforts. Neither is anyone's real
project; they were written to exercise the awkward parts of each shape — a human
gate, a blocked decision, a `blocked(user)` queue item, a journal entry carrying
a discovery, and a superseded rule marked in place.

See [`../../docs/PRIOR-SHAPES.md`](../../docs/PRIOR-SHAPES.md) for what each
shape is and why it looks like this.
