#!/usr/bin/env bash
# bench_record.sh — 跑基准并追加一节到 bench/RESULTS.md（每次优化落地后使用）
#
# 用法:
#   ./scripts/bench_record.sh "M-opt.11：xxx"
#   BENCH_RUNS=5 ./scripts/bench_record.sh "说明"
#   BENCH_SKIP_COMPARE=1 ./scripts/bench_record.sh "只记 std 基准"
#
# 记录内容：跨语言 5 内核（bench_compare.sh）+ std 密集（bench_std.sh，release）
# + std 密集 +LTO。均为交错轮转 min-of-N，checksum 一致性由脚本自身校验。
set -euo pipefail
cd "$(dirname "$0")/.."

NOTE="${1:-}"
DATE=$(date +%F)
COMMIT=$(git rev-parse --short HEAD)
RUNS="${BENCH_RUNS:-3}"
RESULTS="bench/RESULTS.md"
[ -f "$RESULTS" ] || { echo "缺少 $RESULTS"; exit 1; }

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

echo "== std 密集（release）=="
BENCH_RUNS="$RUNS" ./scripts/bench_std.sh > "$tmp/std.txt"
cat "$tmp/std.txt"

echo "== std 密集（+LTO）=="
BENCH_RUNS="$RUNS" AYANAMI_LTO=1 ./scripts/bench_std.sh > "$tmp/std_lto.txt"
cat "$tmp/std_lto.txt"

if [ "${BENCH_SKIP_COMPARE:-0}" != "1" ]; then
    echo "== 跨语言（release，无 LTO）=="
    BENCH_RUNS="$RUNS" ./scripts/bench_compare.sh > "$tmp/cmp.txt"
    tail -20 "$tmp/cmp.txt"
fi

{
    echo
    echo "## $DATE \`$COMMIT\`${NOTE:+ — $NOTE}"
    echo
    echo '### std 密集（release）'
    echo
    echo '```'
    cat "$tmp/std.txt"
    echo '```'
    echo
    echo '### std 密集（+LTO）'
    echo
    echo '```'
    cat "$tmp/std_lto.txt"
    echo '```'
    if [ "${BENCH_SKIP_COMPARE:-0}" != "1" ]; then
        echo
        echo '### 跨语言（release，无 LTO）'
        echo
        echo '```'
        cat "$tmp/cmp.txt"
        echo '```'
    fi
    echo
} >> "$RESULTS"

echo
echo "已追加到 $RESULTS（记得更新顶部趋势总览表）"
