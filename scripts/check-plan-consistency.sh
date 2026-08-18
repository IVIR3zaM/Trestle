#!/usr/bin/env bash
set -euo pipefail

# Consistency check for the build graph itself.
#
# graph.yaml and the node files each state a node's deps, tier and gate. Two
# statements of the same fact drift, and this one drifts silently: nothing reads
# a node file's frontmatter, so a stale `deps:` there misleads only the human or
# agent who opens it — which is exactly when it matters most. T07's node file had
# drifted from the graph before this check existed.
#
# This is the bootstrap rehearsal for T07's gauntlet. Every rule below is a rule
# a user's plan will need too, and the point of `AGENTS.md` §5 is that a rule
# with no command behind it is a wish.
#
# Dependency-free on purpose — see the note at the top of scripts/status.py.

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

python3 - "plan/v0.1.0" << 'PYTHON'
import re
import sys
from pathlib import Path

plan = Path(sys.argv[1])
graph_text = (plan / "graph.yaml").read_text(encoding="utf-8")

VALID_STATUS = {"todo", "in_progress", "blocked", "done", "split"}
VALID_TIER = {"cheap", "standard", "deep"}

# Same minimal scan scripts/status.py uses; anything else is out of contract.
nodes, current = {}, None
for raw in graph_text.splitlines():
    line = re.sub(r"\s+#.*$", "", raw)
    start = re.match(r"^\s*-\s+id:\s*(\S+)", line)
    if start:
        current = start.group(1)
        nodes[current] = {"deps": [], "tier": "", "gate": "", "status": "todo", "oracle": ""}
        continue
    if current is None:
        continue
    kv = re.match(r"^\s+(\w+):\s*(.*)$", line)
    if not kv:
        continue
    key, value = kv.group(1), kv.group(2).strip().strip("\"'")
    if key == "deps":
        nodes[current]["deps"] = [d.strip() for d in value.strip("[]").split(",") if d.strip()]
    elif key in nodes[current]:
        nodes[current][key] = value

errors = []


def frontmatter(path):
    text = path.read_text(encoding="utf-8")
    if not text.startswith("---"):
        return None
    block = text.split("---", 2)[1]
    found = {}
    for line in block.splitlines():
        kv = re.match(r"^(\w+):\s*(.*)$", line.strip())
        if not kv:
            continue
        key, value = kv.group(1), kv.group(2).strip().strip("\"'")
        found[key] = (
            [d.strip() for d in value.strip("[]").split(",") if d.strip()]
            if key == "deps"
            else value
        )
    return found


files = {}
for path in sorted((plan / "nodes").glob("*.md")):
    node_id = path.name.split("-", 1)[0]
    files.setdefault(node_id, []).append(path)

for node_id, node in sorted(nodes.items()):
    if node["status"] not in VALID_STATUS:
        errors.append(f"{node_id}: status '{node['status']}' is not one of {sorted(VALID_STATUS)}")
    if node["tier"] and node["tier"] not in VALID_TIER:
        errors.append(f"{node_id}: tier '{node['tier']}' is not abstract — {sorted(VALID_TIER)}")
    if not node["oracle"]:
        errors.append(f"{node_id}: no oracle. No oracle, no node — make it a human gate or merge it")

    paths = files.get(node_id, [])
    if not paths:
        errors.append(f"{node_id}: named in graph.yaml but has no node file in {plan}/nodes/")
        continue
    if len(paths) > 1:
        errors.append(f"{node_id}: {len(paths)} node files — {', '.join(p.name for p in paths)}")
        continue

    path = paths[0]
    front = frontmatter(path)
    if front is None:
        errors.append(f"{node_id}: {path.name} has no frontmatter block")
        continue
    if front.get("id") != node_id:
        errors.append(f"{node_id}: {path.name} frontmatter says id '{front.get('id')}'")
    # The drift this check exists for. Order-insensitive: the graph is the source
    # of truth for the SET of edges, not for how they were typed.
    if sorted(front.get("deps", [])) != sorted(node["deps"]):
        errors.append(
            f"{node_id}: {path.name} deps {front.get('deps', [])} != graph.yaml deps {node['deps']}"
        )
    if front.get("tier", "") != node["tier"]:
        errors.append(f"{node_id}: {path.name} tier '{front.get('tier','')}' != graph.yaml '{node['tier']}'")
    if front.get("gate", "") != node["gate"]:
        errors.append(f"{node_id}: {path.name} gate '{front.get('gate','')}' != graph.yaml '{node['gate']}'")

for node_id in sorted(files):
    if node_id not in nodes:
        errors.append(f"{node_id}: node file exists but nothing in graph.yaml names it")

for node_id, node in sorted(nodes.items()):
    for dep in node["deps"]:
        if dep not in nodes:
            errors.append(f"{node_id}: depends on '{dep}', which is not a node")
        elif nodes[dep]["status"] == "split":
            errors.append(
                f"{node_id}: depends on '{dep}', which is split — depend on its sub-nodes instead"
            )

if errors:
    for err in errors:
        print(f"  ✗ {err}")
    print(f"\nERROR: {len(errors)} plan-consistency problem(s)", file=sys.stderr)
    sys.exit(1)
print(f"  ✓ {len(nodes)} nodes: node files, deps, tiers, gates and oracles all agree with graph.yaml")
PYTHON
