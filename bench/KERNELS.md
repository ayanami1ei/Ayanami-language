# 基准测试项说明（bench/KERNELS.md）

两套基准：**跨语言五内核**（`bench/bench.aya`，Ayanami/Rust/Java/Zig/C 同题对比）与
**std 密集四内核**（`bench/bench_std.aya`，Ayanami 专用，衡量 std/所有权/内联优化）。
结果记录见 `bench/RESULTS.md`；运行方式见 `bench/README.md` 与 `scripts/bench_record.sh`。

## 总览

| 内核 | 文件 | 测什么 | 规模/次 | 校验和（当前） | Ayanami 现状 |
|---|---|---|---|---:|---:|
| nbody | bench.aya | 浮点标量、依赖链、`sqrt` | 2M 步（5 体） | -169 | 94ms（C 70） |
| matmul | bench.aya | 内存分块、SIMD 向量化 | 1024² f64 | 804172106625 | 198ms（C 157） |
| sieve | bench.aya | 分支密集 + 步长写内存 | n=50M | 3001134 | 194ms（全语言≈194） |
| qsort | bench.aya | 递归 + `ref mut [T]` 所有权 | 2M int | 48006597130597 | 121ms（Zig 112） |
| mandelbrot | bench.aya | 分支浮点 + 自动向量化 | 1024²×300 | 101690050 | 67ms（C 207） |
| list_push | bench_std.aya | 动态数组增长/边界检查/临时分配 | 2M push+index | 999000000 | **5ms**（原 35） |
| str_build | bench_std.aya | 分配+memcpy、ToString 泛型 | 1 万次拼接 | 620283 | 25ms |
| map_ops | bench_std.aya | 哈希/增长/Option 解包 | 50 万 insert+get | 374999250000 | 7ms（原 12） |
| iface | bench_std.aya | 接口分派（装箱+虚调用） | 500 万次调用 | 39999995 | 3ms |

## 跨语言五内核（bench/bench.aya）

### 1. nbody —— 5 体引力积分
- **目的**：浮点标量运算与依赖链（`d2*sqrt(d2)` 每次迭代强依赖），衡量后端标量代码质量与 `sqrt` 调用。
- **算法**：经典 Benchmarks Game 常量（太阳+四大行星，`dt=0.01`，`days=365.24`），
  每次 2,000,000 步；数据为 7 个 `[float; 5]` 数组。
- **校验和**：末态总能量 ×1000 截断 → `-169`（跨语言一致）。
- **读表**：C 最快（70ms）；Ayanami 94ms（约 1.35×）。差距来自标量调度/FMA（基线 x86-64 无 FMA）。
- **优化关联**：`ref mut [float]` 形参 `noalias`（M-opt.1/6）。

### 2. matmul —— 1024×1024 矩阵乘
- **目的**：SIMD 向量化与内存访问（ikj 顺序：内层对 `c[ib+j]` 连续累加）。
- **规模**：3 个 `[float; 1048576]`；初始化 `a[i]=(i%1000)/1000+0.5`、`b[i]=(i%997)/997+0.25`。
- **校验和**：结果矩阵元素和 ×1000 → `804172106625`。
- **读表**：C 157ms、Rust 185、Ayanami 198、Zig 276（基线 CPU）、Java 305。
- **优化关联**：数组分配 `malloc` + `noalias`（M-opt.10）让别名推理更准。

### 3. sieve —— 埃拉托斯特尼筛
- **目的**：分支密集 + 非连续写（步长 `i` 写标记），内存带宽主导。
- **规模**：n=50,000,000（`[u8; 50M]` 标记数组），计数 2..n 素数。
- **校验和**：素数个数 → `3001134`。
- **读表**：全部语言 ≈194ms（带宽/分支天花板，编译器差异被抹平）。

### 4. qsort —— 递归快排
- **目的**：递归调用 + **`ref mut [T]` 所有权/借用**路径（数组经引用传递、原地交换）。
- **规模**：2,000,000 个 int，LCG 数据（`seed=(seed*1103515245+12345)%2^31`），每次重新填充。
- **校验和**：Σ `a[k]*(k%97)`（对顺序敏感）→ `48006597130597`。
- **读表**：Zig 112、Ayanami/Rust 121、C 123；数组 `noalias` 下 opt 可重排加载。

### 5. mandelbrot —— 逃逸时间
- **目的**：分支浮点 + 自动向量化（像素循环独立）。
- **规模**：1024×1024，max 300 迭代；`x0 = px*2.5/W - 2 - r*0.001`（r 为运行序号，防 LICM）。
- **校验和**：总迭代数 → `101690050`。
- **读表**：Ayanami/Rust/Zig 67ms（LLVM 向量化），C 207ms（gcc 未向量化）、Java 248ms。
- **反优化措施**：全局 `MB_TOTAL`/`MB_SEED`（外部时钟调用可能读写全局 → 计时区不可被提升）。

## std 密集四内核（bench/bench_std.aya）

> 这四项走 std `.lcl` 路径（ArrayList/String/HashMap/接口），衡量跨对象内联（LTO）、
> 堆分配提升、track_caller 临时量、动态分派等**非纯算法**开销。

### 6. list_push —— ArrayList 增长 + 索引
- **测**：2,000,000 次 `push`（容量翻倍、边界检查）+ 2,000,000 次 `index`（含
  track_caller 的 `__file` String 构造/释放）。
- **校验和**：索引和 → `999000000`。
- **结果**：35ms → **5ms**。`index` 特化在调用方模块内，分配/释放对可见 →
  每次调用的 `__file` String 被提升为栈（M-opt.10）。
- **下一步瓶颈**：`__file` 会逃逸到 panic 路径（`panic_bounds_at`），堆提升无法
  再进一步；需 track_caller 约定改为「静态 file 视图（借用）」才能消除分配。

### 7. str_build —— String 拼接
- **测**：10,000 次 `s = s.add("ab")`（每次分配新缓冲并拷贝，O(N²) memcpy）
  + 一次 `s.add(123456)`（`ToString` 泛型路径）。
- **校验和**：`(len*31 + s[0])` → `620283`。
- **结果**：25ms，memcpy 主导；分配提升无感。

### 8. map_ops —— HashMap 插入/查询
- **测**：50 万 `insert`（哈希、扩容、冲突探测）+ 50 万 `get`（`Option` 解包）。
- **校验和**：查询值之和 → `374999250000`。
- **结果**：12ms → 7ms（malloc/free + LTO 小幅收益；哈希函数与探测是主要成本）。

### 9. iface —— 接口动态分派
- **测**：`interface Op { fn apply(ref self, int) -> int }`，int/float 两实现，
  500 万次经 `ref Op` 调用（值类型需装箱为 FatPtr）。
- **校验和**：返回值和 → `39999995`。
- **结果**：3ms。opt 在 `call_apply` 内联后看到常量 vtable → **完全去虚化并内联**，
  装箱也被消解；测的是「分派已被优化掉」的残余成本。

## 协议与可信度

- **计时**：`bench.aya` 每内核 1 预热 + 3 次计时取平均（内部），外部再重复取最小；
  `bench_std.aya` 单次计时（防 LICM 提出计时区），`scripts/bench_std.sh` 外部重复取最小。
- **反优化措施**：全局种子/累加器锚定计时区；`AYANAMI_OPT=0` 可验证非中端因素。
- **校验和**：跨语言/跨模式必须一致，否则对比无效（脚本自动断言）。
- **资源隔离**：systemd scope（`MemoryMax=3G`）+ `timeout`，防泄漏吃满整机。
- **读表**：绝对毫秒受负载/频率影响，优先看**同节内的相对关系**与**趋势表**。
