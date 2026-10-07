#!/usr/bin/env bash
# bench_std.sh — Ayanami std 密集基准（跨对象内联 / 堆提升收益测量）
#
# 用法:
#   ./scripts/bench_std.sh              # 当前编译器，3 次外部重复取最小
#   AYANAMI_BIN=install/ayanami ./scripts/bench_std.sh
#   AYANAMI_LTO=1 ./scripts/bench_std.sh    # 若支持 LTO 开关则对比
#
# 资源隔离：优先 systemd-run --user --scope（MemoryMax=3G）；回退 timeout。
set -euo pipefail
cd "$(dirname "$0")/.."

RUNS="${BENCH_RUNS:-3}"
AYANAMI_BIN="${AYANAMI_BIN:-target/debug/ayanami}"
LIMIT_MEM="${BENCH_MEM:-3G}"
LIMIT_TIME="${BENCH_TIMEOUT:-300}"
have_scope=0
if command -v systemd-run >/dev/null 2>&1 \
   && systemd-run --user --scope -p MemoryMax=64M -p MemorySwapMax=0 --quiet true >/dev/null 2>&1; then
    have_scope=1
fi
run_limited() {
    if [ "$have_scope" = 1 ]; then
        timeout "$LIMIT_TIME" systemd-run --user --scope \
            -p MemoryMax="$LIMIT_MEM" -p MemorySwapMax=0 --quiet -- "$@"
    else
        timeout "$LIMIT_TIME" "$@"
    fi
}

# `build` 对裸文件默认静态库；用 `run` 链接可执行文件（跑一次不影响计时协议）
# 先清理 build/：否则 `run --release` 会复用陈旧（debug/LTO）产物导致计时失真
rm -rf build
run_limited "$AYANAMI_BIN" run --release bench/bench_std.aya >/dev/null 2>&1 || true
cp build/bench_std build/bench-std

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
for _ in $(seq 1 "$RUNS"); do
    run_limited ./build/bench-std >> "$tmp/out.txt"
done

python3 - "$tmp/out.txt" <<'PY'
import sys
best = {}
order = []
for line in open(sys.argv[1]):
    parts = line.split()
    if len(parts) != 3:
        continue
    name, cks, ms = parts[0], int(parts[1]), int(parts[2])
    if name not in best:
        order.append(name)
        best[name] = (cks, ms)
    elif ms < best[name][1]:
        best[name] = (cks, ms)
print(f"{'kernel':<12}{'checksum':>16}{'ms':>8}")
for k in order:
    print(f"{k:<12}{best[k][0]:>16}{best[k][1]:>8}")
PY
