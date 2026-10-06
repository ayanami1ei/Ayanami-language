# bench — 跨语言速度对比（M-opt.5）

同一套算法在 **Ayanami / Rust / Java / Zig / C** 上的性能对比，用于验证 release 优化的实际收益。
所有实现保持相同的常量、运算顺序与工作量，**校验和必须跨语言一致**（否则对比无效）。

## 内核与规模

| 内核 | 内容 | 规模 |
|---|---|---|
| `nbody` | 5 体引力积分（经典 Benchmarks Game 常量） | 每次 2,000,000 步，dt=0.01 |
| `matmul` | 1024×1024 f64 矩阵乘（ikj 顺序） | 每次 1 遍（2.15 GFLOP） |
| `sieve` | 埃拉托斯特尼筛（u8 标记） | n = 50,000,000 |
| `qsort` | 递归快排（Hoare 分区）+ LCG 填充 | 2,000,000 int / 次 |
| `mandelbrot` | 逃逸时间算法 | 1024×1024，max 300 iter |

## 协议

- 每个内核：**1 次预热（不计时）+ 3 次计时取平均**，每次计时运行包含该次的数据初始化。
- 输出 5 行：`<kernel> <checksum> <ms>`，ms 为整数平均毫秒。
- 校验和：nbody/matmul 为 `(double 结果 × 1000)` 截断取整，其余为整数。
- mandelbrot 用全局累加器/种子（Ayanami `static mut`、Rust `static mut`、Java static 字段、Zig `var`、C 全局），
  防止优化器把纯计算提出计时区。
- 浮点不做重结合；C 用 `-ffp-contract=off` 避免 FMA 收缩。

## 构建与运行

```bash
# 一键对比（本脚本再做 3 次外部重复取最小，并校验 checksum 一致）
./scripts/bench_compare.sh              # 默认 3 次外部重复
BENCH_RUNS=5 ./scripts/bench_compare.sh # 更稳

# 手工单语言
target/debug/ayanami build --release bench/bench.aya && ./build/bench
rustc --edition 2021 -C opt-level=3 bench/bench.rs -o bench_rust && ./bench_rust
javac -d build bench/Bench.java && java -cp build Bench
zig build-exe bench/bench.zig -O ReleaseFast -mcpu=x86_64 -lc -femit-bin=bench_zig && ./bench_zig
gcc -O3 -ffp-contract=off bench/bench.c -o bench_c -lm && ./bench_c
```

## 结果（2026-10-06，i7-13650HX，5 语言校验和全部一致）

ms，外部 3 次取最小；每语言编译器设置见上（均为各自 release/优化模式，目标 CPU 基线 x86-64）。

| kernel | Ayanami | Rust | Java 17 | Zig | C (gcc) |
|---|---:|---:|---:|---:|---:|
| nbody | 89 | 79 | 86 | 77 | 72 |
| matmul | 194 | 189 | 317 | 279 | 171 |
| sieve | 172 | 172 | 180 | 176 | 174 |
| qsort | 120 | 121 | 137 | 114 | 126 |
| mandelbrot | 68 | 67 | 251 | 69 | 214 |

相对 Ayanami（x 倍，<1 更快）：

| kernel | Rust | Java | Zig | C |
|---|---:|---:|---:|---:|
| nbody | 0.89 | 0.97 | 0.87 | 0.81 |
| matmul | 0.97 | 1.63 | 1.44 | 0.88 |
| sieve | 1.00 | 1.05 | 1.02 | 1.01 |
| qsort | 1.01 | 1.14 | 0.95 | 1.05 |
| mandelbrot | 0.99 | 3.69 | 1.01 | 3.15 |

说明：

- Ayanami（`--release`：`opt -O3` + `llc -O3` + 所有权/标注推断属性 + 内部化）与 Rust/Zig/C 处于同一档；
  内存/整数类内核（sieve/qsort/nbody）与 C 差距 1~24%。
- matmul 慢于 C 约 13%（向量化/循环布局差异），好于 Java 与 Zig（基线 CPU）。
- mandelbrot：三个 LLVM 后端（Ayanami/Rust/Zig）都自动向量化了像素循环（SSE2），gcc/Java 没有，
  所以 C 与 Java 明显更慢——这是编译器自动向量化差异，不是语言本身。
- Java 为 JIT（预热后测量），数值波动比 AOT 大。

## 文件

| 文件 | 说明 |
|---|---|
| `bench.aya` | 参考实现（协议与常量权威） |
| `bench.rs` / `Bench.java` / `bench.zig` / `bench.c` | 各语言移植（由本地模型生成，人工修复编译问题后校验） |
| `../scripts/bench_compare.sh` | 构建 + 重复运行 + 表格 + checksum 一致性校验 |
