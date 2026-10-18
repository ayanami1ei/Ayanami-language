# 编译期求值与约束（const fn / 类型级）— P4e 接口预留

> 状态：**P4e-1 接口边界确认（接入点在位）；P4e-2/P4e-3 待做（#58 / bd `nku.2`）**（2026-10-17 / 2026-10-18）
> 关联：GitHub #58（const fn / 编译期断言 / type-level constants）、#48/#49（Phase 4 统一推断）、
> bd `Ayanami-language-upt.2`（P4e）；文档：`docs/generic-inference.md`、`docs/annotations.md`（效应）
> 目标：为 `const fn`、编译期断言、类型级常量建立**统一求值/约束接口**，并与
> 泛型约束、宏、effect 收敛到同一条链路（#58 的「与泛型约束/宏/effect 统一」）。

## 1. 现状（2026-10-17 盘点）

| 能力 | 位置 | 说明 |
|---|---|---|
| `const NAME = expr` 项 | `hir/lower/body/const_eval.rs` | 小型求值器：字面量/一元/二元/`const` 引用；`checked_neg` 等**溢出即编译错**；导出到 HIR `consts` |
| 类型位置常量 | **解析期**（`Type::ArraySized(_, n, _)` 中 `n: usize`） | `[int; N]`、`[int; 2 + 2]` 已可用（实测 ✓）；解析器内解析 const 表——**求值点分散**（P4e 要收敛的对象） |
| 泛型约束 | `hir/lower/body/generic_types.rs`、`generic_check.rs` | `[T: A + B]`（接口约束，调用点合并检查） |
| 效应 | `hir/effects/*`、`#[pure]` 等标注 | `EffectDecl`/`EffectSet`；纯函数判定 |
| 宏 | `hir/lower/body/macro_call.rs`、`compiler/macro_expand/*` | 编译期展开（HIR 级） |
| 编译期断言 | — | 未实现（`#[test]`/panic 为运行期） |

结论：**求值能力已有，接口未统一**——解析期（类型尺寸）、HIR 期（const 项）、约束检查
（泛型）各自为政，正是 P4c/P4e 要合并的重复规则来源。

## 2. 接口设计（P4e-1）

统一边界（后续实现收敛到这些入口，调用方不再各写求值/比较规则）：

```rust
/// 编译期值（首版仅整数/布尔/字符；后续扩展字符串/数组）
pub enum ConstValue { Int(i64), Bool(bool), Char(u32) }

/// 求值入口：表达式 → 常量（不可求值返回 None，调用方给出位置化诊断）
/// 现为 const_eval 私有逻辑，P4e-1 提升为 crate 级接口（含溢出/除零检查）。
fn eval_const(ctx: &Ctx, expr: &Expr, env: &ConstEnv) -> Option<ConstValue>;

/// 类型位置求值：`[T; N]` 的 N（解析期/ HIR 期共用；解析期实现先保留，逐步迁移）
fn eval_const_usize(ctx: &Ctx, expr: &Expr, env: &ConstEnv) -> Option<usize>;

/// 约束求解接口（与 P4c 的 unify 对接；P4e 只预留，不实现求解器）
trait ConstConstraint {
    /// 泛型参数为 const 参数时的候选值（const generics，P4e-3）
    fn const_arg(&self) -> Option<ConstValue>;
}
```

约束链路的**最小接入点**（不改变现有语义）：

1. `generic_check.rs` 的 `check_generic_constraint`：接口约束已就位；const 参数检查将来
   在**同一处**追加（避免新开一条规则链）。
2. `generic_specialize.rs` 的替换表 `HashMap<Symbol, HirType>`：const 参数将来以
   `HirType::ConstValue`（或 `ArraySized` 的 `n`）形式进入同一替换表。
3. 效应/纯度：`const fn` 的准入条件直接复用效应系统（`#[pure]` ⇒ 可编译期求值），
   不新增标注。

## 3. 阶段与验收

| 阶段 | 内容 | 验收 |
|---|---|---|
| **P4e-1**（本文档） | 接口边界 + 现状盘点 + 最小接入点说明 | 文档评审通过；后续实现不得新开求值链 |
| P4e-2 | `const fn`（`#[pure]` 函数编译期求值）+ 编译期断言（`static_assert`/`const if`） | 正例：const fn 求值进 `const`/数组尺寸；负例：非纯函数/溢出位置化报错 |
| P4e-3 | const generics（`[T; N]` 参数化 + 单态化） | 正例：`struct Buf[T; N]` 按 N 单态化；与 P4c unify 共用替换表 |

## 4. 实现状态

- [x] P4e-1 接口边界与现状盘点（本文档）
- [ ] P4e-2 `const fn` + 编译期断言
- [ ] P4e-3 const generics
- 备注：`const_eval` 的求值函数在 P4e-2 落地时提升为 crate 级接口（`eval_const`/`eval_const_usize`）。
