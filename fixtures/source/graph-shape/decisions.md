# Open decisions

Questions an unattended agent must not answer. Each names the units it blocks.

---

## D1 — Embedded database driver

**Blocks:** G03 — **RESOLVED: the runtime's built-in driver.**

The store interface is fully async, so a synchronous driver works behind it. The
built-in option adds no dependency and needs no native compile step, which keeps
the container small and its build reproducible.

---

## D2 — Does the shared core become a workspace root?

**Blocks:** G05, G06 — **OPEN**

Nothing currently installs the extracted core, so consumers fail to build. Either
the core becomes a workspace root (one install, CI also gains coverage of the
core's own tests, larger diff) or each consumer declares it as a local file
dependency (smaller diff, one more install step per package in CI).

Found while running the full suite as a collateral check for an unrelated unit.
Note that G05's oracle does **not** run the build, so a future pass could mark it
done over a still-broken build — the oracle is narrower than the problem.
