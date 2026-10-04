#!/usr/bin/env bash
# ir_snapshot.sh — 各阶段 IR 回归快照（ast/hir/mir/lir + 一个 llvm 代表）
#
#   生成/更新：./scripts/ir_snapshot.sh --update
#   校验：    ./scripts/ir_snapshot.sh
set -uo pipefail
cd "$(dirname "$0")/.."

BIN="target/debug/ayanami"
if [ ! -x "$BIN" ]; then cargo build -q; fi
DIR="tests/ir_snapshots"
UPDATE=0
[ "${1:-}" = "--update" ] && UPDATE=1

FILES="test_struct test_ref_auto test_macro_fn test_bounds test_oop"
STAGES="ast hir mir lir"
fail=0
n=0

sanitize() {
    sed -e "s#$PWD#<ROOT>#g" -e "s#$HOME#<HOME>#g" -e "s#/tmp/ayanami-dump#<TMP>#g"
}

for f in $FILES; do
    for st in $STAGES; do
        n=$((n + 1))
        out=$("$BIN" dump "$st" "example/$f.aya" 2>/dev/null | sanitize)
        snap="$DIR/$f.$st.txt"
        if [ "$UPDATE" = 1 ]; then
            mkdir -p "$DIR"
            printf '%s\n' "$out" > "$snap"
            continue
        fi
        if [ ! -f "$snap" ]; then
            echo "MISSING $snap (run ./scripts/ir_snapshot.sh --update)"
            fail=$((fail + 1))
        elif ! diff -q <(printf '%s\n' "$out") "$snap" >/dev/null; then
            echo "IR SNAPSHOT DIFF: example/$f.aya ($st) — ./scripts/ir_snapshot.sh --update"
            fail=$((fail + 1))
        fi
    done
done

n=$((n + 1))
out=$("$BIN" dump llvm example/test_struct.aya 2>/dev/null | sanitize)
snap="$DIR/test_struct.llvm.txt"
if [ "$UPDATE" = 1 ]; then
    printf '%s\n' "$out" > "$snap"
elif ! diff -q <(printf '%s\n' "$out") "$snap" >/dev/null 2>&1; then
    echo "IR SNAPSHOT DIFF: example/test_struct.aya (llvm)"
    fail=$((fail + 1))
fi

echo "ir snapshots: $n checked, $fail diff(s)"
[ "$fail" -eq 0 ]
