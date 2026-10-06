# never 类型 `!`（M1.9）

> 状态：已实现（2026-10）。目标：发散表达式类型 `!`，可强转到任意类型；函数 `-> !` 发射 `noreturn`。

## 语法

- 类型位置：`!`（返回类型、`fn(...) -> !`、局部标注）。
- 函数：`fn die() -> ! { panic_at(...) }`。
- 表达式发散来源：
  - `#[noreturn]` 标注函数的调用（如 std `panic_at`）；
  - `-> !` 函数的调用；
  - 显式 `return`（在 if/match 分支中作为发散分支）。

## 语义

- **类型**：`HirType::Never`（显示 `!`）。`!` 是任意类型的子类型：
  - `coerce_expr(expr, T)`：`expr: !` 直接通过（无运行期转换）；
  - `if`/`match` 结果类型折叠把 `!` 当单位元（`fold(!, T) = T`）；
  - 全部分支发散 → 整个 if/match 表达式类型为 `!`。
- **分支不发散赋值**：发散分支不写结果变量（其后的代码不可达）。
- **return/赋值**：`return <!>` / `x = <!>` 先求值（副作用：panic 调用等）再发射
  `unreachable`（赋值则跳过 store）。
- **发射**：
  - `!` → LLVM `void`；`-> !` 函数自动加 `noreturn`；
  - 函数末尾/`ret !` → `unreachable`（新增 `SLirUnreachable`）；
  - `#[noreturn]` 信息随 `.lcl` flags token `noreturn` 导出/导入。

## 范围外（后续）

- 无限循环发散分析（`while true {}` 无 break）；
- `!` 作为参数/字段类型（语法允许但无意义，暂不诊断）；
- 泛型上下文中的 `!` 推断。

## 实现位置

| 环节 | 位置 |
|---|---|
| 类型 | `src/hir/ty.rs`（`Never`）、`src/parser/ast/ty.rs`（`Never`）、`src/parser/parser/types.rs`（`!`） |
| 调用类型 | `FnSig.is_noreturn`（`collect_fns`/导入 flags/特化）、`expr_call*.rs` |
| 折叠/强转 | `match_lower.rs`/`if_expr.rs`（分支分类 Void/Value/Never）、`helpers/coerce.rs` |
| 发射 | `SLirUnreachable`（tag 35）、`mir_stmts.rs`（return/assign）、`fn_lower.rs`、`emit/functions.rs`（`noreturn`） |
| 导出 | `package/symbols.rs` flags `noreturn`、`collect_import.rs` |
| 回归 | `example/test_never.aya`、`tests/compile_fail/never_*` |
