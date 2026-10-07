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
| 2026-10-07 | `a5a4e32` | 所有权健全性（赋值/字段覆盖 drop、实参移动、移出清零；跨语言测量时负载偏高） | 122 | 213 | 227 | 133 | 73 | 11 | 2 | 13 | 4 |
| 2026-10-07 | `8c3172f` | 借用临时量提升 + match/if 非 Copy 结果修复（bench 清理陈旧 build/） | 106 | 223 | 264 | 131 | 75 | 8 | 2 | 10 | 4 |

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

## 2026-10-07 `a5a4e32` — 所有权健全性：赋值/字段覆盖旧值 drop、拥有实参移动跟踪、局部移出清零

### std 密集（release）

```
kernel              checksum      ms
list_push          999000000      11
str_build             620283       2
map_ops         374999250000      13
iface               39999995       4
```

### std 密集（+LTO）

```
kernel              checksum      ms
list_push          999000000      11
str_build             620283       2
map_ops         374999250000      13
iface               39999995       4
```

### 跨语言（release，无 LTO）

```
== 构建（资源上限 3G / 300s）==
构建完成 -> build/bench-cmp
第 1/3 轮完成
第 2/3 轮完成
第 3/3 轮完成

kernel          ayanami       rust       java        zig          c
nbody               122         91         95         80         75
matmul              213        232        394        306        170
sieve               227        214        213        223        214
qsort               133        129        150        119        131
mandelbrot           73         71        267         71        216

相对 Ayanami 的倍数:
kernel          ayanami       rust       java        zig          c
nbody              1.00       0.75       0.78       0.66       0.61
matmul             1.00       1.09       1.85       1.44       0.80
sieve              1.00       0.94       0.94       0.98       0.94
qsort              1.00       0.97       1.13       0.89       0.98
mandelbrot         1.00       0.97       3.66       0.97       2.96

checksum 全部一致（5 内核 x 5 语言）
```


## 2026-10-07 `8c3172f` — 借用临时量提升（ref 实参字面量/调用临时量语句后 drop）+ match/if 表达式非 Copy 结果修复；bench 脚本清理陈旧 build/

### std 密集（release）

```
kernel              checksum      ms
list_push          999000000       8
str_build             620283       2
map_ops         374999250000      10
iface               39999995       4
```

### std 密集（+LTO）

```
kernel              checksum      ms
list_push          999000000       7
str_build             620283       1
map_ops         374999250000      10
iface               39999995       4
```

### 跨语言（release，无 LTO）

```
== 构建（资源上限 3G / 300s）==
构建完成 -> build/bench-cmp
第 1/3 轮完成
第 2/3 轮完成
第 3/3 轮完成

kernel          ayanami       rust       java        zig          c
nbody               106         85         91        101         76
matmul              223        225        382        341        177
sieve               264        264        270        203        255
qsort               131        131        172        119        132
mandelbrot           75         72        319         71        225

相对 Ayanami 的倍数:
kernel          ayanami       rust       java        zig          c
nbody              1.00       0.80       0.86       0.95       0.72
matmul             1.00       1.01       1.71       1.53       0.79
sieve              1.00       1.00       1.02       0.77       0.97
qsort              1.00       1.00       1.31       0.91       1.01
mandelbrot         1.00       0.96       4.25       0.95       3.00

checksum 全部一致（5 内核 x 5 语言）
```

