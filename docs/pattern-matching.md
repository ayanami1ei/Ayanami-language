# 模式匹配增强（Phase 1.3）

> 状态：**atb.1（字面量 / `_` / `|` / guard）已实现**（2026-10）；atb.2（struct 解构 / 嵌套 / tuple / range）、
> atb.3（match 表达式与语句统一 + 穷尽检查增强）待做。

## 语法（atb.1）

```
pattern   := or_pattern
or_pattern:= atom ('|' atom)*
atom      := '_'                       // 通配
           | literal                   // 0 / -1 / 1.5 / 'a' / true
           | ident                     // 绑定整值（Copy）或枚举变体（无载荷）
           | ident '(' ident (',' ident)* ')'   // 枚举变体 + 载荷绑定（`_` 跳过）
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

## 待做

- atb.2：struct 解构、嵌套模式、tuple、range（`1..10`）、`Option` 嵌套；
- atb.3：match 语句/表达式统一、穷尽检查增强（区间/字面量枚举）、`|` 绑定一致性诊断；
- 结构体解构需要字段级绑定与部分移动语义设计。
