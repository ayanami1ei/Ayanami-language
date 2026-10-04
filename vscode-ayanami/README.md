# Ayanami Language Support for VS Code

Syntax highlighting and basic language support for `.aya` files.

## Features

- Syntax highlighting (keywords, types, strings, comments, numbers, operators)
- **Annotation highlighting** for `#[...]`（如 `#[inline]`、`#[io]`、`#[follow_with(x)]`、`#[throws(E)]`）
- **Annotation completion**：输入 `#[` 后建议内置标注（优化/效应/契约/生命周期/宏）
- Keyword/type/field/method completion（含跨文件 `import` 解析）
- **Hover**：函数完整签名（含返回类型/泛型/文档注释/标注）、结构体字段与类型、枚举变体、变量类型；标准库从 `.lcl` 符号表解析
- **诊断**：编译器 `check` 输出解析到精确行列（支持 `at l:c`、`(at l:c)`、`--> file:l:c` 与跨文件错误定位）
- Comment toggling (`//` and `/* */`)
- Bracket matching and auto-closing pairs
- Format command：`Ayanami: Format Code`（Shift+Alt+F）

## Built-in annotations

| 类别 | 标注 |
|---|---|
| 优化 | `inline` `inline(always)` `cold` `noreturn` `pure` `readonly` `nounwind` `willreturn` `noalias` `nonnull` `no_error` |
| 效应 | `io` `state` `alloc` |
| 契约 | `assume(cond)` `requires(cond)` `ensures(result)` `invariant(cond)` `throws(E)` `throws(_)` `throws()` |
| 条件 | `cfg(target = "linux")` `cfg(arch = "x86_64")` `cfg(unix)` |
| 生命周期 | `follow_with(source, ...)` |
| 宏 | `macro` |

详见仓库 `docs/annotations.md`、`docs/lifetimes.md`。

## Configuration

- `ayanami.compilerPath`：编译器路径（留空则从 PATH 查找）。
