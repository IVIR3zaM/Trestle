#!/usr/bin/env bash
set -euo pipefail

# Oracle for T02a: plan format — normative spec, JSON schema, expressed fixtures
#
# The load-bearing assertion here is LOSSLESSNESS. Both source fixtures were written
# before this format existed, so expressing them is the only real test of whether the
# format can hold what people actually write. That test is mechanical on purpose:
# "we read it and it looked complete" is how a format ships having lost the journal.
#
# Deliberately dependency-free — python3 stdlib and standard shell tools only, for
# the reason at the top of scripts/status.py: this runs on a clean clone before any
# toolchain exists. In particular **no PyYAML**, even where it happens to be
# installed. Full structural validation of the fixtures against the schema is T02b's
# job, in Rust, where the parser lives. This oracle checks what a text-level tool can
# check honestly, and nothing it cannot.
#
# Every failure names the assertion that failed.

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

SPEC="docs/PLAN-FORMAT.md"
SCHEMA="schema/plan.schema.json"
EXPRESSED="fixtures/expressed"

failures=0
fail() { echo "  ✗ $1" >&2; failures=$((failures + 1)); }
pass() { echo "  ✓ $1"; }

echo "Checking the deliverables exist..."
for path in "$SPEC" "$SCHEMA"; do
    [ -f "$path" ] && pass "$path exists" || fail "$path does not exist"
done
for shape in graph-shape loop-shape hybrid forward-compat; do
    if [ -f "$EXPRESSED/$shape/plan.yaml" ]; then
        pass "$EXPRESSED/$shape/plan.yaml exists"
    else
        fail "$EXPRESSED/$shape/plan.yaml does not exist"
    fi
done

# Nothing below says anything useful without the files.
if [ $failures -ne 0 ]; then
    echo "ERROR: $failures assertion(s) failed" >&2
    exit 1
fi

echo "Checking the schema is a valid, dialect-declaring JSON Schema..."
schema_report=$(python3 - "$SCHEMA" << 'PYTHON'
import json
import sys

try:
    schema = json.load(open(sys.argv[1], encoding="utf-8"))
except json.JSONDecodeError as exc:
    print(f"not valid JSON: {exc}")
    sys.exit(0)

errors = []
if "$schema" not in schema:
    errors.append("no $schema key: the JSON Schema dialect must be declared")
if schema.get("type") != "object":
    errors.append("top-level type must be 'object'")
if "properties" not in schema:
    errors.append("no top-level 'properties'")
print("OK" if not errors else "\n".join(errors))
PYTHON
)
[ "$schema_report" = OK ] && pass "schema is valid JSON and declares its dialect" \
    || { fail "schema is malformed:"; echo "$schema_report" | sed 's/^/      /' >&2; }

echo "Checking the status vocabulary is complete..."
# One assertion per status, so a missing `done(overridden)` fails here rather than
# surfacing in T11 as a retrofit across every consumer. These are the states
# constraint 7 of the node requires, plus `n_a` — which the loop fixture's own status
# legend uses and the node's list omitted.
for status in draft todo in_progress blocked verified done superseded n_a; do
    if grep -qF "\"$status\"" "$SCHEMA"; then
        pass "schema declares status '$status'"
    else
        fail "schema does not declare status '$status'"
    fi
done
# The override is a distinct permanent state (D9), not a boolean on `done`.
grep -qF '"override"' "$SCHEMA" \
    && pass "schema declares the override record" \
    || fail "schema has no 'override' — D9 requires done(overridden) be a distinct recorded state"

echo "Checking all three shapes are expressed..."
for shape in graph loop hybrid; do
    if grep -rqE "^shape: $shape\b" "$EXPRESSED"/*/plan.yaml; then
        pass "a fixture declares shape '$shape'"
    else
        fail "no fixture in $EXPRESSED declares shape '$shape'"
    fi
done

echo "Checking the journal is present wherever a loop is..."
# D2's rejected option (c) was collapsing a loop into a chain-shaped graph, which
# loses the journal. This asserts the format did not quietly do the same thing.
for dir in "$EXPRESSED"/*/; do
    shape=$(grep -hE '^shape: ' "$dir/plan.yaml" 2>/dev/null | head -1 | awk '{print $2}')
    case "$shape" in
    loop | hybrid)
        if [ ! -f "$dir/journal.md" ]; then
            fail "$(basename "$dir") has shape '$shape' but no journal.md; a loop without a journal is a list someone will lose track of"
            continue
        fi
        missing=""
        for section in Did Verified Learned Next Blocked Commit; do
            grep -qiE "^\*\*$section" "$dir/journal.md" || missing="$missing $section"
        done
        [ -z "$missing" ] && pass "$(basename "$dir") journal has every required section" \
            || fail "$(basename "$dir") journal is missing section(s):$missing"
        ;;
    esac
done

echo "Checking status is separable from definition..."
# Constraint 3, and the structural half of D9: if execution never writes a plan body,
# then any diff to one is by definition a human amending the plan.
for dir in "$EXPRESSED"/*/; do
    name=$(basename "$dir")
    [ "$name" = "forward-compat" ] && continue
    if [ -f "$dir/status.yaml" ]; then
        pass "$name has a separate status.yaml"
    else
        fail "$name has no status.yaml; status must not live in the plan body"
    fi
    # Scoped to unit blocks, not the whole file: a *rule* legitimately carries
    # `status: superseded` in plan.yaml, and a blanket grep for `status:` would
    # reject the very supersession the loop fixture exists to exercise. What must
    # not appear is a status on a unit.
    if python3 - "$dir/plan.yaml" << 'PYTHON'
import re
import sys

# Section-aware, because `- id:` blocks appear under `rules:`, `phases:` and
# `deferred:` as well as `units:` — and a rule's `status: superseded` is exactly
# what the loop fixture exists to exercise. Only a unit may not carry status.
section = None
in_unit = False
for raw in open(sys.argv[1], encoding="utf-8"):
    line = raw.rstrip("\n")
    top = re.match(r"^([a-z_]+):", line)
    if top:
        section = top.group(1)
        in_unit = False
        continue
    if re.match(r"^\s*-\s+id:", line):
        in_unit = section == "units"
        continue
    # `queue:` nests units inside a unit, so stay in unit context through it.
    if in_unit and re.match(r"^\s+status:", line):
        print(line.strip())
        sys.exit(1)
sys.exit(0)
PYTHON
    then
        pass "$name/plan.yaml carries no unit status"
    else
        fail "$name/plan.yaml carries a status on a unit; definition and status must stay separable"
    fi
done

echo "Checking spec and schema agree..."
# Two definitions of one format is the defect that will actually happen: someone
# adds a key to the schema and the spec keeps describing the old shape. The spec is
# the document a reviewer hand-writes a plan from, so a key it does not mention is a
# key nobody knows about.
agreement=$(python3 - "$SCHEMA" "$SPEC" << 'PYTHON'
import json
import re
import sys

schema = json.load(open(sys.argv[1], encoding="utf-8"))
spec = open(sys.argv[2], encoding="utf-8").read()


def property_names(node, found):
    if isinstance(node, dict):
        for key, value in node.items():
            if key == "properties" and isinstance(value, dict):
                found.update(value.keys())
            if isinstance(value, (dict, list)):
                property_names(value, found)
    elif isinstance(node, list):
        for item in node:
            property_names(item, found)
    return found


keys = property_names(schema, set())
# Backticked identifiers in the spec. A key mentioned in prose counts as documented;
# requiring a particular table shape would make the check about formatting.
documented = set(re.findall(r"`([A-Za-z_][A-Za-z0-9_]*)`", spec))

undocumented = sorted(k for k in keys if k not in documented)
if undocumented:
    print("schema keys absent from the spec: " + ", ".join(undocumented))
else:
    print(f"OK {len(keys)} schema keys, all documented")
PYTHON
)
[[ "$agreement" == OK* ]] && pass "spec documents every schema key (${agreement#OK })" \
    || { fail "spec and schema disagree:"; echo "$agreement" | sed 's/^/      /' >&2; }

echo "Checking the graph fixture's structure survived, edge for edge..."
# Ids and edges are extractable without a YAML parser, using the same minimal
# id/deps scan scripts/status.py uses on this repo's own graph. Comparing edge SETS
# rather than counts is what catches a dependency quietly re-parented.
edges=$(python3 - fixtures/source/graph-shape/plan.yaml "$EXPRESSED/graph-shape/plan.yaml" << 'PYTHON'
import re
import sys


def units(path):
    """Minimal scan for `- id:` blocks with an inline `deps: [A, B]`. The fixed
    shape both files use; anything else is out of contract on purpose."""
    found = {}
    current = None
    for raw in open(path, encoding="utf-8"):
        line = re.sub(r"\s+#.*$", "", raw.rstrip("\n"))
        start = re.match(r"^\s*-\s+id:\s*\"?([^\"\s]+)\"?", line)
        if start:
            current = start.group(1)
            found[current] = set()
            continue
        if current is None:
            continue
        deps = re.match(r"^\s+deps:\s*\[([^\]]*)\]", line)
        if deps:
            found[current] = {d.strip().strip("\"'") for d in deps.group(1).split(",") if d.strip()}
    return found


source, expressed = units(sys.argv[1]), units(sys.argv[2])
errors = []

for uid, deps in source.items():
    if uid not in expressed:
        errors.append(f"unit {uid} is missing from the expressed fixture")
        continue
    lost = deps - expressed[uid]
    gained = expressed[uid] - deps
    if lost:
        errors.append(f"unit {uid} lost dependency edge(s): {', '.join(sorted(lost))}")
    if gained:
        errors.append(f"unit {uid} gained dependency edge(s) not in the source: {', '.join(sorted(gained))}")

print("OK" if not errors else "\n".join(errors))
PYTHON
)
[ "$edges" = OK ] && pass "every unit and dependency edge survived" \
    || { fail "the graph fixture lost structure:"; echo "$edges" | sed 's/^/      /' >&2; }

echo "Checking every unit's status survived..."
# The source records status inline; the expressed form separates it (constraint 3),
# so this asserts the MAPPING, which is the actual semantic claim being made. The
# renames are deliberate: the format uses identifier-safe names, and blocked(user)
# becomes a status plus a sibling question rather than data inside an enum value.
statuses=$(python3 - "$EXPRESSED/graph-shape" "$EXPRESSED/loop-shape" << 'PYTHON'
import re
import sys
from pathlib import Path

RENAMED = {"in-progress": "in_progress", "n/a": "n_a"}


def blob(directory):
    """Every file in the expressed fixture, concatenated. Which file a fact landed
    in is the format's business; that it survived at all is this check's."""
    return "\n".join(
        p.read_text(encoding="utf-8", errors="replace")
        for p in sorted(Path(directory).rglob("*"))
        if p.is_file()
    )

source_status = {}
for raw in open("fixtures/source/graph-shape/plan.yaml", encoding="utf-8"):
    uid = re.match(r"^\s*-\s+id:\s*(\S+)", raw)
    if uid:
        current = uid.group(1)
    st = re.match(r"^\s+status:\s*\"?([^\"\n]+)\"?", raw)
    if st:
        source_status[current] = st.group(1).strip()

# The loop's queue lives in a markdown table: | ID | Item | Status | Notes |
for raw in open("fixtures/source/loop-shape/STATE.md", encoding="utf-8"):
    row = re.match(r"^\|\s*(\d+\.\d+)\s*\|([^|]*)\|([^|]*)\|", raw)
    if row:
        source_status["loop:" + row.group(1)] = row.group(3).strip()

expressed = blob(sys.argv[1])
expressed_loop = blob(sys.argv[2])

errors = []
for key, status in source_status.items():
    blob = expressed_loop if key.startswith("loop:") else expressed
    uid = key.split(":", 1)[-1]
    # blocked(user): <question> — the status and the question are asserted apart,
    # because the whole point of the chosen representation is that they are apart.
    if status.startswith("blocked(user)"):
        question = status.split(":", 1)[1].strip()
        if "blocked" not in blob:
            errors.append(f"{uid}: status blocked(user) did not survive")
        # A distinctive fragment of the question, not the whole sentence, so
        # rewrapping the line does not read as data loss.
        if "signing step" not in blob:
            errors.append(f"{uid}: the blocking question was lost ({question[:40]}...)")
        continue
    want = RENAMED.get(status, status)
    if not re.search(rf"\b{re.escape(want)}\b", blob):
        errors.append(f"{uid}: status '{status}' (as '{want}') did not survive")

print("OK" if not errors else "\n".join(errors))
PYTHON
)
[ "$statuses" = OK ] && pass "every unit status survived the separation" \
    || { fail "statuses were lost:"; echo "$statuses" | sed 's/^/      /' >&2; }

echo "Checking the awkward parts survived..."
# The fixtures' README says they exist to exercise the awkward parts, and those are
# prose rather than identifiers: a superseded rule's reason, a journal entry that
# reorders the queue, a deferred item's revisit condition. Curated rather than
# extracted, because there is no pattern to extract — and each phrase names itself
# when it goes missing, which is the property that matters.
survivals=$(python3 - "$EXPRESSED" << 'PYTHON'
import re
import sys
from pathlib import Path

# Whitespace is normalised before comparing. YAML folds long prose across lines and
# so does markdown, so a phrase check that respects line breaks would be asserting
# formatting rather than content — and would fail the moment someone rewrapped a
# paragraph, which is the kind of false alarm that gets an oracle switched off.
def blob(directory):
    text = "\n".join(
        p.read_text(encoding="utf-8", errors="replace")
        for p in sorted(Path(directory).rglob("*"))
        if p.is_file()
    )
    return re.sub(r"\s+", " ", text).lower()


CURATED = [
    ("loop-shape", "the goal", "indistinguishable from a fresh clone"),
    ("loop-shape", "the plan-level done_when", "dry-run of the upstream update flow"),
    ("loop-shape", "hard rule 4", "Never add the upstream as a permanent remote"),
    ("loop-shape", "the superseded rule's reason", "Batching lost the reasoning"),
    ("loop-shape", "the blocking question", "signing step"),
    ("loop-shape", "the queue-reordering discovery", "1.3 can proceed independently"),
    ("loop-shape", "a journal entry's stated uncertainty", "A guess"),
    ("loop-shape", "a note cross-referencing the journal", "two local keys kept"),
    ("loop-shape", "in-progress detail", "18 of 24 applied"),
    ("loop-shape", "the classification count", "31 files"),
    ("loop-shape", "a deferred item", "Adopt upstream's test runner"),
    ("loop-shape", "a deferred item's reason", "Migration touches every test file"),
    ("loop-shape", "a deferred item's revisit condition", "Once both have migrated"),
    ("loop-shape", "a phase title", "reconcile divergence"),
    ("graph-shape", "the goal", "self-hostable container option"),
    ("graph-shape", "a resolved decision", "runtime's built-in driver"),
    ("graph-shape", "an open decision", "workspace root"),
    ("graph-shape", "the oracle-adequacy warning", "oracle is narrower than the problem"),
    ("graph-shape", "a unit's requirement", "monotonic sequence numbers"),
    ("graph-shape", "a unit's acceptance detail", "50 parallel writes"),
    ("graph-shape", "a unit's oracle command", "STORE=embedded npm test"),
    ("graph-shape", "a human gate's oracle", "check-arch-doc.sh"),
]

root = sys.argv[1]
blobs = {}
for directory, what, phrase in CURATED:
    if directory not in blobs:
        blobs[directory] = blob(f"{root}/{directory}")
    needle = re.sub(r"\s+", " ", phrase).lower()
    print(("OK  " if needle in blobs[directory] else "MISSING  ") + f"{directory}: {what} — \"{phrase}\"")
PYTHON
)
while IFS= read -r line; do
    case "$line" in
    OK*) pass "${line#OK  }" ;;
    *) fail "lost ${line#MISSING  }" ;;
    esac
done <<< "$survivals"

echo "Checking journal entries are addressable..."
# STATE.md's notes point at journal entries ("see LOG 2"), so an entry needs a
# stable id to point at. Without one, the cross-reference is a comment.
if grep -qiE '^#+ .*\b(entry|iteration)[ -]?[0-9]+' "$EXPRESSED/loop-shape/journal.md"; then
    pass "journal entries carry addressable ids"
else
    fail "journal entries have no addressable id; STATE.md's 'see LOG 2' has nothing to point at"
fi

echo "Checking multi-repo is reserved, not precluded..."
if grep -qE '^\s+repo:' "$EXPRESSED/forward-compat/plan.yaml" && grep -qF 'repo' "$SCHEMA"; then
    pass "a unit may name its repo, so v0.2.0 multi-repo needs no breaking change"
else
    fail "no forward-compatibility fixture naming a unit's repo; multi-repo would become a breaking change"
fi

echo "Checking tiers stayed abstract..."
# Same rule this repo holds for its own plan: a tier is cheap/standard/deep, never a
# vendor's model name. A plan naming one stops being portable between harnesses.
if grep -rniE '\b(gpt|claude|sonnet|opus|haiku|gemini|llama|mistral|codex)\b' \
    "$SPEC" "$SCHEMA" "$EXPRESSED" > /tmp/vendor-hits.$$ 2>/dev/null; then
    fail "a vendor model name appears in the format or its fixtures:"
    sed 's/^/      /' /tmp/vendor-hits.$$ >&2
    rm -f /tmp/vendor-hits.$$
else
    rm -f /tmp/vendor-hits.$$
    pass "no vendor model name in the spec, schema or fixtures"
fi

echo
if [ $failures -ne 0 ]; then
    echo "ERROR: $failures assertion(s) failed" >&2
    exit 1
fi
echo "✓ All checks passed"
