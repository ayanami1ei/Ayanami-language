# 模式匹配增强（Phase 1.3）

> 状态：**atb.1（字面量 / `_` / `|` / guard）与 atb.2（嵌套 / struct 解构 / range）已实现**（2026-10）；
> tuple 无对应类型（语言无元组，暂不做）；atb.3（match 表达式与语句统一 + 穷尽检查增强）待做。

## 语法（atb.1）

```
pattern   := or_pattern
or_pattern:= atom ('|' atom)*
atom      := '_'                       // 通配
           | literal                   // 0 / -1 / 1.5 / 'a' / true
           | literal '..' literal      // 区间（半开）
           | literal '..=' literal     // 区间（闭）
           | ident                     // 绑定整值（顶层仅 Copy）或枚举变体（无载荷）
           | ident '(' pattern (',' pattern)* ')'          // 枚举变体 + 子模式（可嵌套）
           | ident '{' field (',' field)* '}'              // 结构体解构
field     := ident                     // 简写（绑定同名字段）
           | ident ('=' | ':') pattern // 重命名/嵌套子模式
arm       := pattern ('if' expr)? '=>' expr (',' | ';')?
```

- 枚举 scrutinee 下裸标识符若匹配变体名 → 视为无载荷变体（`None => ...`）；
  否则视为整值绑定（仅 Copy 类型；拥有类型报错，避免与 scrutinee 双重释放）。
- 标量 scrutinee（int/char/bool/float/IntN）：字面量等值比较；float 支持整数/浮点字面量。
- guard 在臂作用域内求值（可引用载荷绑定）；guard 失败自动 fallthrough 到后续臂。

## 降级

- 目标求值一次存入 `__match_val` 临时局部；
- 逐臂顺序 `if !__match_matched && cond { bindings; [if guard] body; __match_matched = true }`
  （guard 支持 fallthrough；LLVM 优化合并）；
- 穷尽性：不可反驳臂（`_` / 绑定，无 guard）或枚举全变体覆盖 / bool 双值覆盖；
  不满足报 `non-exhaustive match`。

## 实现位置

| 环节 | 位置 |
|---|---|
| 模式 AST | `src/parser/ast/pattern.rs`（`Pattern` + `MatchArm.pattern/guard`） |
| 解析 | `src/parser/parser/pattern.rs`、`decl.rs`（臂循环） |
| 降级 | `src/hir/lower/body/match_lower.rs`（编排/穷尽）、`match_pattern.rs`（条件/绑定/变体 tag） |
| 回归 | `example/test_match_literals.aya`、负例 `match_non_exhaustive` |

## atb.2 补充（2026-10）

- **嵌套模式**：`Some(Some(v))`、`Some(None)`（裸变体名在任意层级归一为变体模式）、
  `Some(Point { x, y })`；
- **结构体解构**：`Point { x, y }`（简写）、`Point { x = a, y = b }` / `Point { x: a }`；
  字段绑定允许拥有类型（随 scrutinee 移动，与枚举载荷绑定一致）；
- **区间**：`1..10`（半开）、`10..=20`（闭），限整数/char（scrutinee 类型检查）；
- 顺带修复：嵌套泛型枚举载荷未递归实例化（`Opt[Opt[int]]` 的 `_0: Opt<int>`），
  以及 `instantiate_enum_value` 不递归导致无载荷变体回退基名后类型名残缺
  （`Opt_Some<Opt><Opt<int>>`）；
- 回归：`example/test_match_destructure.aya`。

## 待做

- atb.3：match 语句/表达式统一、穷尽检查增强（区间/字面量枚举）、`|` 绑定一致性诊断；
- tuple：语言无元组类型，暂不做；
- 结构体解构的字段级部分移动在 drop 侧的完备性（当前与枚举载荷同策略）。
