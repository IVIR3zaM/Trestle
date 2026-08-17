#!/usr/bin/env python3
"""Prints the v0.1.0 build-graph state: per-node status and readiness.

Readiness = status `todo` and every dependency `done`.

Deliberately dependency-free (no PyYAML) — the executor runs this on a clean
clone before any toolchain is set up. Python rather than Node because Trestle is
a Rust project (D6) and requiring one language's runtime to read the build graph
of a tool that plans repos in any language is the thing D6 rejected.

This is bootstrap scaffolding. It is replaced by `trestle status` (T12) and
`trestle next` (T10), and deleting it is the milestone.
"""

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
GRAPH = ROOT / "plan" / "v0.1.0" / "graph.yaml"

NODE_START = re.compile(r"^\s*-\s+id:\s*(\S+)")
KEY_VALUE = re.compile(r"^\s+(\w+):\s*(.*)$")
TRAILING_COMMENT = re.compile(r"\s+#.*$")


def parse_graph(text):
    """Minimal parser for the fixed shape graph.yaml uses: a `nodes:` list of
    `- id:` blocks with scalar `key: value` fields and a `deps: [A, B]` inline
    array. Anything else is out of contract on purpose."""
    nodes = []
    cur = None
    for raw in text.splitlines():
        line = TRAILING_COMMENT.sub("", raw)
        stripped = line.strip()
        if not stripped or stripped.startswith("#"):
            continue

        start = NODE_START.match(line)
        if start:
            cur = {"id": start.group(1), "deps": [], "title": "", "status": "todo"}
            nodes.append(cur)
            continue
        if cur is None:
            continue

        kv = KEY_VALUE.match(line)
        if not kv:
            continue
        key, raw_val = kv.group(1), kv.group(2).strip().strip("\"'")
        if key == "deps":
            cur["deps"] = [d.strip() for d in raw_val.strip("[]").split(",") if d.strip()]
        else:
            cur[key] = raw_val
    return nodes


nodes = parse_graph(GRAPH.read_text(encoding="utf-8"))
by_id = {n["id"]: n for n in nodes}

missing = [f'{n["id"]} -> {d}' for n in nodes for d in n["deps"] if d not in by_id]
if missing:
    print(f'graph.yaml references unknown nodes: {", ".join(missing)}', file=sys.stderr)
    sys.exit(2)


def is_done(node):
    return node["status"] == "done"


def is_ready(node):
    return node["status"] == "todo" and all(is_done(by_id[d]) for d in node["deps"])


MARK = {"done": "✓", "in_progress": "·", "blocked": "✗", "todo": " ", "split": "⊘"}
width = max(len(n["title"]) for n in nodes)

print("\nTrestle v0.1.0 — build graph\n")
for n in nodes:
    gate = " [human gate]" if n.get("gate") == "human" else ""
    if is_ready(n):
        state = "READY"
    elif n["status"] == "todo":
        waits = ",".join(d for d in n["deps"] if not is_done(by_id[d]))
        state = f"waits on {waits}"
    else:
        state = n["status"]
    tier = n.get("tier", "")
    print(f'  {MARK.get(n["status"], "?")} {n["id"]}  {n["title"]:<{width}}  {tier:<8}  {state}{gate}')

ready = [n for n in nodes if is_ready(n)]
auto = [n for n in ready if n.get("gate") != "human"]

# A `split` node is a container, not work — its sub-nodes carry the load and it can
# never be `done`. Counting it in the denominator would make a fully-built graph
# read as permanently incomplete.
countable = [n for n in nodes if n["status"] != "split"]
done = sum(1 for n in countable if is_done(n))

print(
    f"\n  {done}/{len(countable)} done · {len(ready)} ready "
    f"({len(auto)} unattended, {len(ready) - len(auto)} gated)"
)
blocked = [n["id"] for n in nodes if n["status"] == "blocked"]
if blocked:
    print(f'  blocked: {", ".join(blocked)} — see plan/v0.1.0/decisions.md')
print()

# Machine-readable tail for the executor.
print(f'READY_UNATTENDED={",".join(n["id"] for n in auto)}')
print(f'READY_GATED={",".join(n["id"] for n in ready if n.get("gate") == "human")}')
