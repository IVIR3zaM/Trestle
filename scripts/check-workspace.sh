#!/usr/bin/env bash
set -euo pipefail

# Oracle for T00: Cargo workspace, binary shell, lints and CI
# Validates: fmt, clippy, test, deny checks; binary naming; --version output; CI consistency

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

fail() {
    echo "ERROR: $1" >&2
    exit 1
}

check_tool() {
    local tool=$1
    local install_cmd=$2
    if ! command -v "$tool" &> /dev/null; then
        fail "tool '$tool' not found; install with: $install_cmd"
    fi
}

# Define the four core checks that must run in CI
declare -a CHECKS=(
    "cargo fmt --check"
    "cargo clippy -- -D warnings"
    "cargo test --workspace"
    "cargo deny check"
)

echo "Checking required tools..."
check_tool "cargo" "rustup update"
check_tool "rustfmt" "rustup component add rustfmt"
check_tool "cargo-deny" "cargo install cargo-deny"
check_tool "rsync" "install via your OS package manager (e.g. apt install rsync, brew install rsync)"
if ! cargo clippy --version &> /dev/null; then
    fail "clippy not found; install with: rustup component add clippy"
fi

echo "Running core checks..."
for check in "${CHECKS[@]}"; do
    echo "Running $check..."
    eval "$check" || fail "$check failed"
done

echo "Building release binary..."
cargo build --release || fail "cargo build --release failed"

echo "Checking binary is named 'trestle'..."
if [ ! -f "target/release/trestle" ]; then
    fail "binary 'target/release/trestle' not found; check [[bin]] name in Cargo.toml"
fi

echo "Checking trestle --version output..."
version_output=$(./target/release/trestle --version 2>&1 || true)
if ! echo "$version_output" | grep -qE "^trestle [0-9]+\.[0-9]+\.[0-9]+ "; then
    fail "trestle --version does not match expected format (version): got '$version_output'"
fi
if ! echo "$version_output" | grep -qE "[a-zA-Z0-9]*-[a-zA-Z0-9]*-[a-zA-Z0-9]*"; then
    fail "trestle --version does not include target triple: got '$version_output'"
fi
if ! echo "$version_output" | grep -qE "[0-9a-f]{7,}"; then
    fail "trestle --version does not include git SHA: got '$version_output'"
fi

# T00 creates no library crates — the nodes that own them do, so a crate here
# with no code in it would be a lie about progress. Asked of cargo rather than
# grepped out of the manifest, so a member added by any means is caught.
echo "Checking workspace membership..."
members=$(cargo metadata --no-deps --format-version 1 \
    | python3 -c 'import json,sys; print(" ".join(sorted(p["name"] for p in json.load(sys.stdin)["packages"])))')
if [ "$members" != "trestle-cli" ]; then
    fail "workspace must contain trestle-cli and nothing else; T00 creates no library crates. Found: $members"
fi

echo "Checking CI workflow runs same commands as oracle..."
workflow_file=".github/workflows/check.yaml"
if [ ! -f "$workflow_file" ]; then
    fail "CI workflow not found at $workflow_file"
fi

# Check that the workflow file runs each command verbatim, not just a prefix of
# it — a prefix match would let CI drop flags like `-D warnings` or narrow
# `--workspace` to a single crate and still read as "unchanged".
for check in "${CHECKS[@]}"; do
    if ! grep -qF -- "$check" "$workflow_file"; then
        fail "CI workflow does not include command: $check"
    fi
done

echo "Testing planted violations..."

# Each planted-violation test below mutates a workspace file to prove the
# corresponding check actually fires. That mutation must never touch the real
# working tree — a future node's uncommitted edits to those same files (most
# nodes touch main.rs and Cargo.toml) must survive an oracle run intact. So
# each test runs against a throwaway copy of the workspace in its own temp
# directory; the real tree is only ever read (via rsync), never written.
#
# All scratch directories are recorded here and removed by a single EXIT trap
# — never a `git checkout`/`restore`/`clean` — so cleanup is correct whether
# the script exits normally, exits early under `set -e`, or is killed by
# SIGINT mid-test.
# One root directory holds every scratch copy, and the trap removes that root.
# Registering each copy individually does not work: `make_violation_scratch` is
# called in a command substitution, so any array it appends to belongs to the
# subshell and the parent's trap sees an empty list. That bug leaked a full
# workspace copy per test per run.
violation_scratch_root=$(mktemp -d "${TMPDIR:-/tmp}/trestle-t00-violations.XXXXXX")

cleanup_violation_scratch_root() {
    rm -rf "$violation_scratch_root"
}
trap cleanup_violation_scratch_root EXIT INT TERM

make_violation_scratch() {
    # mktemp, not a counter: this runs in a command substitution, so a counter
    # variable would increment only in the subshell and every test would be
    # handed the same directory, each one inheriting the last one's mutations.
    local scratch
    scratch=$(mktemp -d "$violation_scratch_root/case.XXXXXX")
    rsync -a --exclude='.git' --exclude='target' "$REPO_ROOT/" "$scratch/"
    echo "$scratch"
}

violation_test_failed=0

# Each test below asserts two things: that the guard's command fails, and that
# it failed *for the stated reason*. The second half is not pedantry. An earlier
# version of this script planted a forbidden dependency by overwriting the root
# workspace manifest; `cargo deny check` then died on "no targets specified in
# the manifest" without ever reaching the ban list, so the test passed green and
# would have passed identically against an empty deny.toml. A guard seen to fail
# for the wrong reason is worse evidence than no guard at all, because it reads
# as proof.
assert_violation() {
    local label=$1 expected=$2 scratch=$3
    shift 3
    local output status
    output=$(cd "$scratch" && "$@" 2>&1) && status=0 || status=$?

    if [ "$status" -eq 0 ]; then
        echo "  ✗ $label: '$*' succeeded; the guard did not fire"
        violation_test_failed=1
        return 0
    fi
    if ! echo "$output" | grep -qE -- "$expected"; then
        echo "  ✗ $label: '$*' failed, but not for the expected reason"
        echo "      expected output matching: $expected"
        echo "      got:"
        echo "$output" | sed 's/^/        /' | head -15
        violation_test_failed=1
        return 0
    fi
    echo "  ✓ $label"
}

# Test 1: an unformatted file fails `cargo fmt --check`.
echo "  Testing format violation..."
scratch=$(make_violation_scratch)
cat > "$scratch/crates/trestle-cli/src/main.rs" << 'EOF'
fn main(){let x=1;println!("{}",x);}
EOF
assert_violation "unformatted file fails cargo fmt" "Diff in" "$scratch" \
    cargo fmt --check

# Test 2: a function over the `too_many_lines` threshold fails clippy.
#
# The planted function is called from main on purpose. Left uncalled, rustc's
# dead_code fires first and aborts the compile before clippy's own lints run —
# the test would then pass on dead_code while proving nothing about the lint it
# names.
echo "  Testing clippy too_many_lines violation..."
scratch=$(make_violation_scratch)
python3 - "$scratch/crates/trestle-cli/src/main.rs" << 'EOF'
import sys
path = sys.argv[1]
source = open(path).read()
source = source.replace("fn main() {\n", "fn main() {\n    let _ = planted_long();\n", 1)
body = "".join(f"    n += {i};\n" for i in range(1, 90))
open(path, "w").write(source + "\nfn planted_long() -> u32 {\n    let mut n = 0u32;\n" + body + "    n\n}\n")
EOF
assert_violation "long function fails clippy::too_many_lines" \
    "clippy::too_many_lines" "$scratch" \
    cargo clippy -- -D warnings

# Test 3: a function over the `cognitive_complexity` threshold fails clippy.
echo "  Testing clippy cognitive_complexity violation..."
scratch=$(make_violation_scratch)
python3 - "$scratch/crates/trestle-cli/src/main.rs" << 'EOF'
import sys
path = sys.argv[1]
source = open(path).read()
source = source.replace("fn main() {\n", "fn main() {\n    let _ = planted_complex(3);\n", 1)
body = "".join(
    f"    if x > {i} {{ n += {i}; }} else if x < {i} {{ n -= {i}; }} else {{ n *= 2; }}\n"
    for i in range(1, 30)
)
open(path, "w").write(source + "\nfn planted_complex(x: i64) -> i64 {\n    let mut n = 0i64;\n" + body + "    n\n}\n")
EOF
assert_violation "complex function fails clippy::cognitive_complexity" \
    "clippy::cognitive.complexity" "$scratch" \
    cargo clippy -- -D warnings

# Test 4: `-D warnings` escalates an ordinary rustc warning to a hard failure.
# This is a different property from tests 2 and 3 — those prove two specific
# clippy lints are switched on, this proves warnings are fatal at all.
echo "  Testing -D warnings escalation..."
scratch=$(make_violation_scratch)
cat >> "$scratch/crates/trestle-cli/src/main.rs" << 'EOF'

fn never_called() {}
EOF
assert_violation "dead code fails under -D warnings" "dead_code" "$scratch" \
    cargo clippy -- -D warnings

# Test 5: a dependency the policy bans fails `cargo deny check bans`.
#
# The dependency goes on the member crate, leaving the workspace structure
# intact, and the assertion names the banned crate. Both details matter: a
# broken manifest would fail the command without consulting the policy, and so
# would having no network to resolve the dependency. Asserting on the
# `error[banned]` line is what separates the policy working from the scratch
# copy being broken. `check bans` rather than a bare `check` because an advisory
# firing on some transitive crate is not the ban list firing, and the RUSTSEC
# database changes on days nobody touches this repo.
echo "  Testing banned dependency violation..."
scratch=$(make_violation_scratch)
printf '\n[dependencies]\nreqwest = "0.11"\n' >> "$scratch/crates/trestle-cli/Cargo.toml"
assert_violation "banned dependency fails cargo deny check bans" \
    "error\[banned\]: crate 'reqwest" "$scratch" \
    cargo deny check bans

if [ $violation_test_failed -ne 0 ]; then
    fail "one or more planted violation tests failed"
fi

echo "✓ All checks passed"
