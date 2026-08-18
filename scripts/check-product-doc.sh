#!/usr/bin/env bash
set -euo pipefail

# Oracle for T01: product contract + privacy threat model
#
# Asserts that docs/PRODUCT.md and docs/THREAT-MODEL.md exist and say the things
# the rest of the graph is checked against: the seven-step flow with step ownership,
# the three architectural invariants, the v0.1.0 non-goals, and a channel table in
# which every row carries a countermeasure and either a named test or a declared
# gap.
#
# Every failure names the assertion that failed. "check-product-doc.sh failed" is
# useless to whoever has to fix it.

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

PRODUCT="docs/PRODUCT.md"
THREATS="docs/THREAT-MODEL.md"

failures=0

fail() {
    echo "  ✗ $1" >&2
    failures=$((failures + 1))
}

pass() {
    echo "  ✓ $1"
}

# Asserts a pattern appears in a file. Used for every "the document must say this"
# assertion, so each one reports which claim is missing and where it was expected.
assert_in() {
    local label=$1 pattern=$2 file=$3
    if [ ! -f "$file" ]; then
        fail "$label: $file does not exist"
        return 0
    fi
    if grep -qiE -- "$pattern" "$file"; then
        pass "$label"
    else
        fail "$label: $file has no line matching /$pattern/"
    fi
}

echo "Checking the documents exist..."
for file in "$PRODUCT" "$THREATS"; do
    if [ -f "$file" ]; then
        pass "$file exists"
    else
        fail "$file does not exist"
    fi
done

# Nothing below can say anything useful without the files. Stop here rather than
# emitting thirty identical "file does not exist" lines.
if [ $failures -ne 0 ]; then
    echo "ERROR: $failures assertion(s) failed" >&2
    exit 1
fi

echo "Checking the seven-step flow and its ownership..."
# The flow is only useful if a reader can tell whose step each one is: under D5 the
# agent and Trestle alternate, and a reader who gets that backwards designs the next
# component backwards.
for step in survey interrogate shape absorb "pre-mortem" show "write"; do
    assert_in "flow names the '$step' step" "^\|.*$step" "$PRODUCT"
done
assert_in "flow table has an owner column" "^\|[^|]*step[^|]*\|[^|]*(whose|owner|run by)" "$PRODUCT"

echo "Checking the three architectural invariants are unmissable..."
# One grep per invariant, as the node requires. Each must be a heading — a phrase
# buried in a paragraph is missable, and "cannot miss" is the actual criterion.
assert_in "invariant 1 (no inference, D0) has a heading" \
    "^#+ .*invariant 1.*no inference.*D0" "$PRODUCT"
assert_in "invariant 2 (inverted control, D5) has a heading" \
    "^#+ .*invariant 2.*inverted control.*D5" "$PRODUCT"
assert_in "invariant 3 (unforgeable progress, D9) has a heading" \
    "^#+ .*invariant 3.*unforgeable progress.*D9" "$PRODUCT"

# "Cannot miss" also means stated before the detail. All three must appear in the
# document's opening, not only in their own sections two screens down.
opening=$(head -60 "$PRODUCT")
for invariant in "no inference" "inverted control" "unforgeable progress"; do
    if echo "$opening" | grep -qiE -- "$invariant"; then
        pass "opening 60 lines state '$invariant'"
    else
        fail "opening 60 lines of $PRODUCT do not state '$invariant'"
    fi
done

echo "Checking the limit of the unforgeable-progress claim is stated..."
# D9 is "loud, not prevented". A document that claims the invariant without its
# limit is overclaiming, which is the specific failure decisions.md warns about.
assert_in "override path is stated" "override" "$PRODUCT"
assert_in "override is described as loud rather than prevented" \
    "loud.*(not|rather than).*prevent" "$PRODUCT"

echo "Checking the pre-mortem's limit is stated..."
# D18: write refuses without the block, but an agent can write one without
# thinking. A doc claiming the step is guaranteed would be overclaiming exactly
# as a doc claiming the override is prevented would.
assert_in "pre-mortem step is stated" "pre-mortem" "$PRODUCT"
assert_in "the limit of the pre-mortem is stated" \
    "(visible and deliberate|cannot make an agent think|cannot observe whether)" "$PRODUCT"

echo "Checking the v0.1.0 non-goals..."
assert_in "non-goals section exists" "^#+ .*not (in|do).*v0\.1\.0|^#+ .*out of scope" "$PRODUCT"
assert_in "non-goal: unattended runs" "unattended" "$PRODUCT"
assert_in "non-goal: scheduling" "schedul" "$PRODUCT"
assert_in "non-goal: observed token usage" "observed token usage|actual (token )?usage" "$PRODUCT"
assert_in "non-goals link to the v0.2.0 plan" "\.\./plan/v0\.2\.0/README\.md" "$PRODUCT"

if [ -f "plan/v0.2.0/README.md" ]; then
    pass "the v0.2.0 link resolves"
else
    fail "the v0.2.0 link target plan/v0.2.0/README.md does not exist"
fi

echo "Checking the threat model's party table..."
for party in "Trestle itself" "harness" "LAN"; do
    assert_in "party table has a row for '$party'" "^\|.*$party" "$THREATS"
done

echo "Checking the four inverted-control channels are present..."
# These four postdate the original node text. They are named individually because
# a table that silently loses one produces an egress suite (T16) that silently
# stops testing it.
assert_in "channel: MCP server is stdio-only" "stdio" "$THREATS"
assert_in "channel: trestle init writes outside .trestle/" \
    "init.*outside|outside .*\.trestle" "$THREATS"
assert_in "channel: embedded assets" "embedded asset|compiled into the binary" "$THREATS"
assert_in "channel: update checks" "update check" "$THREATS"

echo "Checking no row is a placeholder..."
if grep -qiE '\bTODO\b|\bTBD\b|\bFIXME\b' "$THREATS"; then
    fail "$THREATS contains TODO/TBD/FIXME; a placeholder countermeasure is not a countermeasure"
    grep -niE '\bTODO\b|\bTBD\b|\bFIXME\b' "$THREATS" | sed 's/^/      /' >&2
else
    pass "no TODO/TBD/FIXME in $THREATS"
fi

echo "Checking the channel table..."
# The table is T16's input: it turns each row into a named test. That makes the
# table a machine-read interface rather than prose, so it is parsed here — with
# the parse itself asserting the contract T16 will rely on.
table_report=$(python3 - "$THREATS" << 'PYTHON'
import re
import sys

path = sys.argv[1]
lines = open(path, encoding="utf-8").read().splitlines()

HEADER = ["id", "channel", "countermeasure", "check"]


def cells(line):
    return [c.strip() for c in line.strip().strip("|").split("|")]


# Find the channel table by its header row rather than by position, so reordering
# the document does not silently disable this check.
start = None
for i, line in enumerate(lines):
    if line.startswith("|") and [c.lower() for c in cells(line)] == HEADER:
        start = i
        break

if start is None:
    print("no channel table found: expected a header row | " + " | ".join(HEADER) + " |")
    sys.exit(0)

rows = []
for line in lines[start + 2:]:
    if not line.startswith("|"):
        break
    rows.append(cells(line))

errors = []
if len(rows) < 4:
    errors.append(f"channel table has only {len(rows)} row(s); the node enumerates more than that")

seen = set()
gap_ids = []
for row in rows:
    if len(row) != 4:
        errors.append(f"row has {len(row)} cells, expected 4: {row}")
        continue
    cid, channel, countermeasure, check = row
    where = cid or channel or "<empty row>"
    if not re.fullmatch(r"CH-\d\d", cid):
        errors.append(f"{where}: id must look like CH-01, got '{cid}'")
    elif cid in seen:
        errors.append(f"{cid}: duplicate id")
    else:
        seen.add(cid)
    if not channel:
        errors.append(f"{where}: empty channel")
    # The node's own wording: a row without a countermeasure is the failure this
    # assertion exists for, and an em-dash or "n/a" is an empty cell wearing a hat.
    if not countermeasure or countermeasure in {"-", "—", "n/a", "N/A", "none", "None"}:
        errors.append(f"{where}: no countermeasure")
    if not check:
        errors.append(f"{where}: empty check; name a test or write GAP")
    elif check == "GAP":
        gap_ids.append(cid)
    elif not re.search(r"[A-Za-z_][A-Za-z0-9_:]*", check.strip("`")):
        errors.append(f"{where}: check '{check}' is neither a test name nor GAP")

# Every GAP must be justified in the Gaps section. T01 requires a channel with no
# automated check to be named as a gap; without this cross-check, GAP would be the
# cheapest way to make any row pass.
gaps_section = []
in_gaps = False
for line in lines:
    if re.match(r"^##\s+Gaps\b", line, re.I):
        in_gaps = True
        continue
    if in_gaps and re.match(r"^##\s+", line):
        break
    if in_gaps:
        gaps_section.append(line)

documented = set(re.findall(r"CH-\d\d", "\n".join(gaps_section)))
for cid in gap_ids:
    if cid not in documented:
        errors.append(f"{cid}: check is GAP but no entry for it under '## Gaps'")

if not errors:
    print(f"OK {len(rows)} rows, {len(gap_ids)} declared gap(s)")
else:
    for err in errors:
        print(err)
PYTHON
)

if [[ "$table_report" == OK* ]]; then
    pass "channel table well-formed (${table_report#OK })"
else
    fail "channel table is malformed:"
    echo "$table_report" | sed 's/^/      /' >&2
    failures=$((failures))
fi

echo "Checking README claims trace to the documents..."
# The node's acceptance says every claim in README.md must trace to a statement in
# one of these documents. Full traceability of every sentence is a review
# judgement and is listed as such in AGENTS.md §5; what is checked here is the
# load-bearing set — the claims that promise the user something. If one of these
# is deleted from a document while the README still makes the promise, that is the
# README lying, and this catches it.
check_claim() {
    local claim=$1 pattern=$2
    if grep -qiE -- "$pattern" "$PRODUCT" "$THREATS"; then
        pass "README claim traced: $claim"
    else
        fail "README claim '$claim' traces to nothing in $PRODUCT or $THREATS (/$pattern/)"
    fi
}

check_claim "makes no outbound network connections" "no outbound network connection"
check_claim "no telemetry, analytics or crash reporting" "telemetr"
check_claim "no crash reporting" "crash report"
check_claim "no update checks" "update check"
check_claim "no HTTP client in the dependency tree" "no HTTP client"
check_claim "dashboard is loopback-bound" "127\.0\.0\.1|loopback"
check_claim "dashboard is the only listener" "only listener|sole listener"
check_claim "dashboard assets are compiled in" "compiled into the binary|embedded asset"
check_claim "no API key, no inference" "no API key"
check_claim "the agent drives Trestle" "agent drives Trestle"
check_claim "three human-facing commands" "trestle init.*trestle status.*trestle ui|init.*status.*ui"
check_claim "verify is the sole writer of done" "only writer of .done|sole writer of .done"
check_claim "a reviewer can withhold done but never grant it" "withhold"
check_claim "tiers are abstract, never a vendor model name" "vendor model name"
check_claim "tiering is inert where the harness has no subagents" "inert"
check_claim "actual token usage records as unknown" "unknown"
check_claim "Trestle adds no new recipient of your code" "no new recipient"
check_claim "the egress guarantee is tested, with a planted violation" "planted"
check_claim "no harness-subprocess exemption in v0.1.0" "exemption"
check_claim "init is reversible with trestle uninstall" "trestle uninstall"
# The README names three gaps outright. Each depends on a party Trestle does not
# ship, so none can be closed by a test — which is exactly why the document must
# keep saying so. Deleting one of these rows while the README still promises the
# honesty is the failure this catches.
check_claim "the oracle command is not sandboxed by Trestle" "cannot sandbox it|does not pretend"
check_claim "gaps are named as gaps" "^#+ .*Gaps"

echo
if [ $failures -ne 0 ]; then
    echo "ERROR: $failures assertion(s) failed" >&2
    exit 1
fi
echo "✓ All checks passed"
