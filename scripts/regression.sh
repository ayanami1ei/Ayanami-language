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
if [ ! -x "$BIN" ] || [ "$BIN" = "target/debug/ayanami" ]; then cargo build -q; fi

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
    # .expected 每行是一个候选子串；命中任意一个即通过
    # （借用冲突等诊断可能有多种等价报法，顺序不确定）
    hit=0
    while IFS= read -r pat; do
        [ -z "$pat" ] && continue
        if grep -qF -- "$pat" <<<"$out"; then
            hit=1
            break
        fi
    done < "$base.expected"
    if [ "$hit" != "1" ]; then
        echo "FAIL $f: none of the expected substrings found:"
        sed 's/^/  - /' "$base.expected"
        fail=$((fail + 1))
    fi
done

rt=0
while read -r name code pat; do
    [ -z "${name:-}" ] && continue
    case "$name" in \#*) continue ;; esac
    rt=$((rt + 1))
    out=$(timeout 60 "$BIN" run "tests/runtime_safety/$name" 2>&1)
    got=$?
    if [ "$got" != "$code" ]; then
        echo "FAIL tests/runtime_safety/$name: exit $got, want $code"
        fail=$((fail + 1))
    fi
    if [ -n "${pat:-}" ] && ! grep -qF -- "$pat" <<<"$out"; then
        echo "FAIL tests/runtime_safety/$name: output missing: $pat"
        fail=$((fail + 1))
    fi
done < tests/runtime_safety/manifest.txt

# #106：零参函数不得发射裸符号（避免与 libc 冲突）
if [ -f build/test_zero_arg_fn ]; then
    if nm build/test_zero_arg_fn 2>/dev/null | grep -qE ' T time$'; then
        echo "FAIL #106: zero-arg fn emitted bare symbol 'time'"
        fail=$((fail + 1))
    fi
fi

# #100：源模块调用 .lcl 泛型方法不得重复单态化（multiple definition）
if [ -d tests/lcl_mono ]; then
    mono_dir=tests/lcl_mono
    "$BIN" package "$mono_dir/mlib.aya" >/dev/null 2>&1 || true
    if [ -f "$mono_dir/mlib.lcl" ]; then
        cp "$mono_dir/mlib.lcl" "$(dirname "$BIN")/std/" 2>/dev/null || true
        timeout 120 "$BIN" run "$mono_dir/entry.aya" >/dev/null 2>&1
        got=$?
        if [ "$got" != 0 ]; then
            echo "FAIL $mono_dir/entry.aya: exit $got, want 0（#100 .lcl 泛型特化重复定义）"
            fail=$((fail + 1))
        fi
        rm -f "$(dirname "$BIN")/std/mlib.lcl"
    else
        echo "FAIL $mono_dir/mlib.aya: package failed"
        fail=$((fail + 1))
    fi
fi

# #104：传递 .lcl 依赖（libb → liba）的符号/泛型 impl 注册
if [ -d tests/lcl_transitive ]; then
    tdir=tests/lcl_transitive
    "$BIN" package "$tdir/liba.aya" >/dev/null 2>&1 || true
    cp "$tdir/liba.lcl" "$(dirname "$BIN")/std/" 2>/dev/null || true
    "$BIN" package "$tdir/libb.aya" >/dev/null 2>&1 || true
    cp "$tdir/libb.lcl" "$(dirname "$BIN")/std/" 2>/dev/null || true
    timeout 120 "$BIN" run "$tdir/entry.aya" >/dev/null 2>&1
    got=$?
    if [ "$got" != 0 ]; then
        echo "FAIL $tdir/entry.aya: exit $got, want 0（#104 传递 .lcl 依赖注册）"
        fail=$((fail + 1))
    fi
    rm -f "$(dirname "$BIN")/std/liba.lcl" "$(dirname "$BIN")/std/libb.lcl"
fi

echo "regression: positive=$pos negative=$neg runtime=$rt failures=$fail"
[ "$fail" -eq 0 ]
