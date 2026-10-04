#!/usr/bin/env bash
# regression.sh — 语言 feature 正/负回归
#
#   正例：example/*.aya，按 tests/positive_exit.txt 校验退出码（含 panic 101 等）
#   负例：tests/compile_fail/*.aya，按同名 .expected（每行一个子串）校验报错内容
#
# 用法：./scripts/regression.sh [--binary target/debug/ayanami]
set -uo pipefail
cd "$(dirname "$0")/.."

BIN="target/debug/ayanami"
if [ "${1:-}" = "--binary" ] && [ -n "${2:-}" ]; then BIN="$2"; fi
if [ ! -x "$BIN" ]; then cargo build -q; fi

fail=0
pos=0
while read -r name code; do
    [ -z "${name:-}" ] && continue
    case "$name" in \#*) continue ;; esac
    pos=$((pos + 1))
    timeout 180 "$BIN" run "example/$name" >/dev/null 2>&1
    got=$?
    if [ "$got" != "$code" ]; then
        echo "FAIL example/$name: exit $got, want $code"
        fail=$((fail + 1))
    fi
done < tests/positive_exit.txt

neg=0
for f in tests/compile_fail/*.aya; do
    [ -e "$f" ] || continue
    neg=$((neg + 1))
    base="${f%.aya}"
    if [ ! -f "$base.expected" ]; then
        echo "FAIL $f: missing $base.expected"
        fail=$((fail + 1))
        continue
    fi
    out=$(timeout 60 "$BIN" check "$f" 2>&1)
    while IFS= read -r pat; do
        [ -z "$pat" ] && continue
        if ! grep -qF -- "$pat" <<<"$out"; then
            echo "FAIL $f: expected substring not found: $pat"
            fail=$((fail + 1))
        fi
    done < "$base.expected"
done

echo "regression: positive=$pos negative=$neg failures=$fail"
[ "$fail" -eq 0 ]
