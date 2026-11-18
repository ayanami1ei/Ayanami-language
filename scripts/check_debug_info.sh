#!/usr/bin/env bash
# check_debug_info.sh — DWARF 行表、局部变量与 GDB/LLDB 源码断点回归
set -euo pipefail
cd "$(dirname "$0")/.."

BIN="${AYANAMI_BIN:-target/debug/ayanami}"
if [[ ! -x "$BIN" ]]; then
    cargo build -q
fi

SRC="example/test_debug_info.aya"
EXE="build/test_debug_info"

run_expect_42() {
    set +e
    "$@" >/dev/null 2>&1
    local code=$?
    set -e
    if [[ "$code" -ne 42 ]]; then
        echo "FAIL debug fixture: exit $code, want 42" >&2
        exit 1
    fi
}

if ! command -v readelf >/dev/null; then
    echo "FAIL debug check: readelf is required" >&2
    exit 1
fi

AYANAMI_DEBUG_INFO=0 "$BIN" run "$SRC" >/dev/null 2>&1 || code=$?
if [[ "${code:-0}" -ne 42 ]]; then
    echo "FAIL disabled debug info changed program behavior" >&2
    exit 1
fi
if readelf -S "$EXE" | grep -q '\.debug_info'; then
    echo "FAIL AYANAMI_DEBUG_INFO=0 still emitted .debug_info" >&2
    exit 1
fi
unset code

run_expect_42 "$BIN" run "$SRC"
readelf --debug-dump=decodedline "$EXE" | grep -q 'test_debug_info\.aya'

if command -v gdb >/dev/null; then
    gdb_out=$(gdb -q -batch \
        -ex 'set debuginfod enabled off' \
        -ex 'break example/test_debug_info.aya:4' \
        -ex run -ex 'info line' -ex 'info locals' "$EXE")
    grep -q 'Line 4' <<<"$gdb_out"
    grep -q 'a = 40' <<<"$gdb_out"
    grep -q 'b = 2' <<<"$gdb_out"
fi

if command -v lldb >/dev/null; then
    lldb_out=$(lldb --batch \
        -o 'breakpoint set --file test_debug_info.aya --line 4' \
        -o run -o 'frame variable' "$EXE")
    grep -q 'a = 40' <<<"$lldb_out"
    grep -q 'b = 2' <<<"$lldb_out"
fi

run_expect_42 "$BIN" run --release "$SRC"
readelf --debug-dump=decodedline "$EXE" | grep -q 'test_debug_info\.aya'
if command -v gdb >/dev/null; then
    gdb_out=$(gdb -q -batch \
        -ex 'set debuginfod enabled off' \
        -ex 'info line example/test_debug_info.aya:4' "$EXE")
    grep -q 'Line 4' <<<"$gdb_out"
fi

echo "debug info: DWARF lines, locals, GDB/LLDB source breakpoints OK"
