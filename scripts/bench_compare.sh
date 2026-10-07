#!/usr/bin/env bash
# bench_compare.sh — 同套算法跨语言速度对比（Ayanami / Rust / Java / Zig / C）
#
# 用法:
#   ./scripts/bench_compare.sh            # 默认每语言外部重复 3 次（取各内核最小值）
#   BENCH_RUNS=5 ./scripts/bench_compare.sh
#   AYANAMI_BIN=install/ayanami ./scripts/bench_compare.sh
#
# 协议见 bench/README.md：每程序内部 1 次预热 + 3 次计时取平均，输出
#   <kernel> <checksum> <ms>
# 本脚本再做 3 次外部重复并取最小值，同时校验各语言 checksum 一致。
set -euo pipefail
cd "$(dirname "$0")/.."

RUNS="${BENCH_RUNS:-3}"
OUT="build/bench-cmp"
AYANAMI_BIN="${AYANAMI_BIN:-target/debug/ayanami}"
# 资源隔离：优先 systemd-run scope（MemoryMax 限制物理内存，防泄漏吃满整机）；
# 回退 ulimit -v（虚拟内存；JVM 预留大地址空间，故 Java 单独用 -Xmx）。
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
mkdir -p "$OUT"

echo "== 构建（资源上限 $LIMIT_MEM / ${LIMIT_TIME}s）=="
# `build` 默认产出静态库；用 `run` 链接可执行文件（跑一次不影响计时协议）
run_limited "$AYANAMI_BIN" run --release bench/bench.aya >/dev/null 2>&1 || true
cp build/bench "$OUT/bench_ayanami"
run_limited rustc --edition 2021 -C opt-level=3 bench/bench.rs -o "$OUT/bench_rust"
run_limited javac -d "$OUT" bench/Bench.java
run_limited zig build-exe bench/bench.zig -O ReleaseFast -mcpu=x86_64 -lc -femit-bin="$OUT/bench_zig" >/dev/null
run_limited gcc -O3 -ffp-contract=off bench/bench.c -o "$OUT/bench_c" -lm
echo "构建完成 -> $OUT"

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
declare -A CMD=(
    [ayanami]="$OUT/bench_ayanami"
    [rust]="$OUT/bench_rust"
    [java]="java -Xmx1g -cp $OUT Bench"
    [zig]="$OUT/bench_zig"
    [c]="$OUT/bench_c"
)
langs=(ayanami rust java zig c)
for lang in "${langs[@]}"; do : > "$tmp/$lang.txt"; done
# 交错轮转：每轮各语言轮流跑一次，消除时段负载偏差（配合取最小值）
for r in $(seq 1 "$RUNS"); do
    for lang in "${langs[@]}"; do
        run_limited ${CMD[$lang]} >> "$tmp/$lang.txt"
    done
    echo "第 $r/$RUNS 轮完成"
done

python3 - "$tmp" <<'PY'
import sys
tmp = sys.argv[1]
langs = ["ayanami", "rust", "java", "zig", "c"]
kernels = ["nbody", "matmul", "sieve", "qsort", "mandelbrot"]
best = {l: {} for l in langs}
for l in langs:
    for line in open(f"{tmp}/{l}.txt"):
        parts = line.split()
        if len(parts) != 3:
            continue
        name, cks, ms = parts[0], int(parts[1]), int(parts[2])
        if name not in best[l] or ms < best[l][name][1]:
            best[l][name] = (cks, ms)

ref = {k: best["ayanami"][k][0] for k in kernels}
bad = [k for k in kernels for l in langs if best[l][k][0] != ref[k]]

w = 11
print()
print(f"{'kernel':<12}" + "".join(f"{l:>{w}}" for l in langs))
for k in kernels:
    row = f"{k:<12}"
    for l in langs:
        row += f"{best[l][k][1]:>{w}}"
    print(row)
print()
print("相对 Ayanami 的倍数:")
print(f"{'kernel':<12}" + "".join(f"{l:>{w}}" for l in langs))
for k in kernels:
    base = best["ayanami"][k][1] or 1
    row = f"{k:<12}"
    for l in langs:
        row += f"{best[l][k][1]/base:>{w}.2f}"
    print(row)
print()
if bad:
    print("checksum 不一致（!）:", ", ".join(f"{l}/{k}" for k in kernels for l in langs if best[l][k][0] != ref[k]))
    sys.exit(1)
print("checksum 全部一致（5 内核 x 5 语言）")
PY
