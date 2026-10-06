# 优化设计：debug / release 双模式（M-opt）

> 状态：M-opt.1 已实现（2026-10）。目标：**release 利用所有权模型与标注系统做激进优化**。

## 模式定义

| 维度 | debug（默认） | release（`--release`） |
|---|---|---|
| 整数溢出 | 检查并 panic（101） | 直接回绕 |
| 契约 `#[requires]`/`#[ensures]`/`#[invariant]` | 运行检查 | 退化为 `llvm.assume` |
| 中端优化 | 跳过（编译快、便于调试） | `opt -O3` + `llc -O3` |
| 语义属性 | 仅显式标注 | 标注 + **所有权/语义推断**（`noalias` / `nounwind`） |
| 内部化 | 无 | 非导出函数/全局 `internal`（M-opt.2） |

- 开关：CLI `--release`（进程级 `hir::contracts::set_release`）；`AYANAMI_OPT=0` 仍可强制关闭中端 opt。
- 两种模式**语义一致**（溢出行为按设计不同），结果必须一致；仅性能与检查强度不同。

## 所有权模型可推导的优化（我们自己的）

| 语言事实 | 优化 | 阶段 |
|---|---|---|
| `ref mut T` 独占（借用检查保证） | 形参 `noalias` → GVN/LICM/重排 | M-opt.1 |
| 拥有值 `Unique`（`[T]`/`String`/拥有胖指针）无别名 | 形参 `noalias` | M-opt.1 |
| 无异常、无栈展开（panic 为 noreturn 退出） | 全函数 `nounwind` | M-opt.1 |
| 移动语义、无 RC/GC | 无写屏障/引用计数开销（天然） | — |
| `#[pure]` / 推断无 io·state·alloc | `memory(none)` / `memory(read)`（A3b 已有） | — |
| `#[noreturn]` / 发散 | `noreturn`（已有） | — |
| `#[inline(always)]` | `alwaysinline`（已有） | — |
| 非导出函数/全局 | `internal` 链接 → opt 可内联/删除 | M-opt.2 |
| 数组长度常量 / 循环归纳变量 | 边界检查消除 | M-opt.3 |

## 分阶段

| 阶段 | 内容 | 状态 |
|---|---|---|
| M-opt.1 | 模式分离（debug 无 opt / release `-O3`）+ release 推断 `noalias`（`ref mut`/owned）与 `nounwind` | ✅ 已实现 |
| M-opt.2 | 内部化：非导出函数/vtable wrapper/`static` 全局 `internal`（LIR4 携带 `is_pub`） | ✅ 已实现 |
| M-opt.3 | 边界检查消除 | 已评估关闭：编译器不发射数组边界检查（内建 `[T]` 不检查；`String`/`ArrayList` 检查在 std 源码） |
| M-opt.4 | 效应驱动的跨函数优化（`#[pure]` 常量折叠、DCE） | ✅ 已具备（A3 推断效应 → `memory(none/read)`/`nounwind` 自动属性） |
| M-opt.5 | 基准套件（与 C 对比，纳入回归的可选性能项） | 待做 |

## 验收（M-opt.1）

- release 构建的 `.ll`：无 `__ayanami_ovf_*` 调用；含 `noalias`（`ref mut`/owned 形参）与 `nounwind`；
- debug 构建的 `.ll`：有 `__ayanami_ovf_*`；无推断 `noalias`；
- `example/test_release_opt.aya` 两种模式退出码一致；std 测试（debug）与抽样 release 运行通过。

## 实现位置

| 环节 | 位置 |
|---|---|
| 模式开关 | `src/cli/mod.rs`（`set_release`）、`src/hir/contracts.rs`（`is_release`/`checks_enabled`） |
| 溢出回绕 | `src/hir/lower/body/expr_ops1.rs`（`!is_release()` 才插检查） |
| 中端/后端档位 | `src/driver/mod.rs::ir_to_object`（release: `opt -O3` + `llc -O3`；debug: 跳过 opt） |
| 推断属性 | `src/lir/emit/functions.rs`（形参 `noalias`、函数 `nounwind`） |
| 内部化 | `src/lir/emit/functions.rs`（linkage）、`src/lir/emit/vtable.rs`（wrapper）、`src/lir/serialize/*`（LIR4 `is_pub`） |
| 回归 | `scripts/regression.sh`（#M-opt.1 段：两种模式 IR 检查 + release 运行） |

## 风险与边界

- `noalias` 推断对「`ref mut` 指向 `static mut` 且另一参数同指」这类全局别名不健全——release 激进模式的已知取舍；
  M5 unsafe/裸指针落地后收紧。
- 内部化需区分导出/被跨对象引用（weak 特化、extern "C"、`main`），M-opt.2 处理。
- debug 跳过 opt 后，非法 IR 由 `llc` 校验兜底（回归仍覆盖）。
