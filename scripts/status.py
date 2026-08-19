#!/usr/bin/env python3
"""Prints the v0.1.0 build-graph state: per-node status and readiness.

Readiness = status `todo` and every dependency `done`.

    python3 scripts/status.py             per-node status (the default)
    python3 scripts/status.py --graph     the same graph drawn by layer, with edges
    python3 scripts/status.py --mermaid   a mermaid flowchart on stdout

Deliberately dependency-free (no PyYAML) — the executor runs this on a clean
clone before any toolchain is set up. Python rather than Node because Trestle is
a Rust project (D6) and requiring one language's runtime to read the build graph
of a tool that plans repos in any language is the thing D6 rejected. The drawing
modes hold the same line: no graphviz, no renderer, no network.

This is bootstrap scaffolding. It is replaced by `trestle status` (T12) and
`trestle next` (T10), and deleting it is the milestone.
"""

import os
import re
import shutil
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


def print_status():
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


def layer_of(node_id, seen=None):
    """Longest path from a root, so a node always sits below every dep it has.
    Shortest path would put T03 next to T02a and hide that it waits on T05."""
    seen = seen or ()
    if node_id in seen:
        cycle = " -> ".join(seen[seen.index(node_id):] + (node_id,))
        print(
            f"graph.yaml has a dependency cycle: {cycle}\n"
            f"Break it in plan/v0.1.0/graph.yaml — a cycle means no node is ever ready.",
            file=sys.stderr,
        )
        sys.exit(2)
    deps = by_id[node_id]["deps"]
    if not deps:
        return 0
    return 1 + max(layer_of(d, seen + (node_id,)) for d in deps)


def dependents_of(node_id):
    return sorted(n["id"] for n in nodes if node_id in n["deps"])


def print_graph():
    """The same graph as `status`, drawn as layers: everything on a row can run in
    parallel, and each row needs at least one node from the row above it."""
    colour = sys.stdout.isatty() and os.environ.get("NO_COLOR") is None
    dim, bold, green, yellow, off = (
        ("\033[2m", "\033[1m", "\033[32m", "\033[33m", "\033[0m") if colour else ("",) * 5
    )
    term = shutil.get_terminal_size((100, 24)).columns

    rows = {}
    for n in nodes:
        rows.setdefault(layer_of(n["id"]), []).append(n)

    print(f"\n{bold}Trestle v0.1.0 — build graph by layer{off}")
    print(f'{dim}  ✓ done   ▸ ready   ○ waiting   ⊘ split   ⛔ human gate'
          f'   ← needs   → unlocks{off}\n')

    for layer in sorted(rows):
        members = rows[layer]
        head = f"  layer {layer}"
        print(f'{dim}{head} {"─" * max(0, min(term, 90) - len(head) - 1)}{off}')
        for i, n in enumerate(members):
            last = i == len(members) - 1
            elbow, gutter = ("└", " ") if last else ("├", "│")
            if is_done(n):
                mark, tint = "✓", green
            elif is_ready(n):
                mark, tint = "▸", bold
            elif n["status"] == "split":
                mark, tint = "⊘", dim
            elif n["status"] == "blocked":
                mark, tint = "✗", yellow
            else:
                mark, tint = "○", dim
            gate = f" {yellow}⛔{off}" if n.get("gate") == "human" else ""
            unlocks = dependents_of(n["id"])
            needs = " ".join(n["deps"]) or "—"
            title = n["title"]
            if len(title) > 46:
                title = title[:45] + "…"
            print(f'  {dim}{elbow}{off} {tint}{mark} {n["id"]:<4}{off} {title:<46}{gate}'.rstrip())
            print(f'  {dim}{gutter}   ← {needs}{off}')
            if unlocks:
                print(f'  {dim}{gutter}   → {" ".join(unlocks)}{off}')
        print()

    ready = [n["id"] for n in nodes if is_ready(n)]
    print(f'  {bold}ready now:{off} {" ".join(ready) if ready else "nothing"}\n')


def print_mermaid():
    """Mermaid rather than DOT: GitHub, this repo's own markdown and every editor
    preview render it without installing graphviz."""
    print("```mermaid")
    print("flowchart TD")
    for n in nodes:
        gate = " ⛔" if n.get("gate") == "human" else ""
        # A double quote in a title would end the label early and break the diagram.
        title = n["title"].replace('"', "'")
        if is_done(n):
            cls = "done"
        elif is_ready(n):
            cls = "ready"
        elif n["status"] == "split":
            cls = "split"
        elif n["status"] == "blocked":
            cls = "blocked"
        else:
            cls = "todo"
        print(f'  {n["id"]}["{n["id"]} · {title}{gate}"]:::{cls}')
    print()
    for n in nodes:
        for d in n["deps"]:
            print(f'  {d} --> {n["id"]}')
    print()
    print("  classDef done fill:#d7f0d7,stroke:#3a7d3a,color:#16321a")
    print("  classDef ready fill:#fff3c4,stroke:#a37f00,color:#3d2f00,stroke-width:2px")
    print("  classDef todo fill:#f2f2f2,stroke:#9a9a9a,color:#3a3a3a")
    print("  classDef blocked fill:#ffd8d8,stroke:#a33,color:#3a1616")
    print("  classDef split fill:#ececec,stroke:#bbb,color:#777,stroke-dasharray:4 3")
    print("```")


MODES = {"--graph": print_graph, "--mermaid": print_mermaid}

args = sys.argv[1:]
if not args:
    print_status()
elif len(args) == 1 and args[0] in MODES:
    MODES[args[0]]()
else:
    print(
        f'unknown argument: {" ".join(args)}\n'
        f"Usage: status.py [--graph | --mermaid]   (no argument prints per-node status)",
        file=sys.stderr,
    )
    sys.exit(2)
