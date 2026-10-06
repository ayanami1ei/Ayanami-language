# 性能记录（bench/RESULTS.md）

每次优化落地前后跑同一套基准并记录（**表格**）。约定：

- 跨语言：`./scripts/bench_compare.sh`（5 内核 × Ayanami/Rust/Java/Zig/C，交错轮转 min-of-N，checksum 全一致）
- std 密集：`./scripts/bench_std.sh`（list_push / str_build / map_ops / iface，release；`AYANAMI_LTO=1` 可对比 LTO）
- 自动追加：`./scripts/bench_record.sh "说明"`（追加一节到本文件）
- 机器：i7-13650HX（20 线程）；构建/运行均在 systemd scope（MemoryMax=3G）内。
  绝对耗时受后台负载影响，**看相对与趋势**；同节内的跨语言数字可横向比较。

## 趋势总览（Ayanami，ms）

| 日期 | commit | 说明 | nbody | matmul | sieve | qsort | mandelbrot | list_push | str_build | map_ops | iface |
|---|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 2026-10-06 | `04b391c` | M-opt.5 基线（debug/release 分离+内部化） | 89 | 194 | 172 | 120 | 68 | — | — | — | — |
| 2026-10-06 | `9effd85` | M-opt.6/7/8（所有权/导入/运行时属性） | 92 | 187 | 194 | 119 | 67 | — | — | — | — |
| 2026-10-06 | `752501d` | 栈 alloca 修复 + std 基准建立 | — | — | — | — | — | 35 | 25 | 12 | 3 |
| 2026-10-06 | `c1628f4` | M-opt.9/10（LTO + malloc/free 堆提升） | 94 | 198 | 194 | 121 | 67 | **5** | 25 | **7** | 3 |
| 2026-10-06 | `c1628f4` | 同上 + LTO | — | — | — | — | — | 5 | 25 | 7 | 3 |

## 2026-10-06 `04b391c` — M-opt.5 基线

跨语言（min-of-3 外部重复，无 scope）：

| kernel | Ayanami | Rust | Java 17 | Zig | C (gcc) |
|---|---:|---:|---:|---:|---:|
| nbody | 89 | 79 | 86 | 77 | 72 |
| matmul | 194 | 189 | 317 | 279 | 171 |
| sieve | 172 | 172 | 180 | 176 | 174 |
| qsort | 120 | 121 | 137 | 114 | 126 |
| mandelbrot | 68 | 67 | 251 | 69 | 214 |

## 2026-10-06 `9effd85` — M-opt.6/7/8（所有权/导入/运行时属性）

跨语言（交错轮转 min-of-3，systemd scope）：

| kernel | Ayanami | Rust | Java 17 | Zig | C (gcc) |
|---|---:|---:|---:|---:|---:|
| nbody | 92 | 77 | 84 | 75 | 71 |
| matmul | 187 | 186 | 319 | 284 | 167 |
| sieve | 194 | 189 | 196 | 191 | 196 |
| qsort | 119 | 120 | 138 | 111 | 122 |
| mandelbrot | 67 | 66 | 247 | 68 | 207 |

（属性对本套循环型内核中性；价值在引用/接口/跨对象密集代码。）

## 2026-10-06 `752501d` — 栈 alloca 修复 + std 密集基准

`bench_std.sh`（release，min-of-5）：

| kernel | checksum | ms |
|---|---:|---:|
| list_push | 999000000 | 35 |
| str_build | 620283 | 25 |
| map_ops | 374999250000 | 12 |
| iface | 39999995 | 3 |

## 2026-10-06 `c1628f4` — M-opt.9/10（跨对象内联 LTO + 堆提升）

`bench_std.sh`（release，min-of-5）：malloc/free 直接发射后
`index` 特化的 `__file` String 分配/释放对可见 → 提升为栈。

| kernel | 无 LTO | +LTO | 说明 |
|---|---:|---:|---|
| list_push | **5**（35→5，7×） | 5 | 每次调用 `__file` String 提升为栈 |
| str_build | 25 | 25 | O(N²) memcpy 主导 |
| map_ops | 8 | **7** | 分配/内联小幅收益 |
| iface | 3 | 3 | 已完全去虚化+内联 |

跨语言（release 默认，无 LTO；交错轮转 min-of-3）：

| kernel | Ayanami | Rust | Java 17 | Zig | C (gcc) |
|---|---:|---:|---:|---:|---:|
| nbody | 94 | 77 | 81 | 74 | 70 |
| matmul | 198 | 185 | 305 | 276 | 157 |
| sieve | 194 | 195 | 201 | 196 | 194 |
| qsort | 121 | 121 | 139 | 112 | 123 |
| mandelbrot | 67 | 67 | 248 | 67 | 207 |

相对 Ayanami：Rust 0.82~1.01、Zig 0.79~1.39、C 0.74~3.09、Java 0.86~3.70。
（M-opt.9/10 对纯数组内核中性——这些内核几乎不经过 std 分配路径；收益见上方 std 表。）
