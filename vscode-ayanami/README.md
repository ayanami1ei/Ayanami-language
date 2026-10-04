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

## 类型提示（Inlay Hints）

变量类型提示由编译器提供（`ayanami types <file>` 输出 HIR 推断结果），
不是编辑器端猜测；编译器不可用或旧版本时回退到启发式扫描。
支持 `: 类型` 标注（赋值声明处、`self`、`for` 迭代变量）。

## 快捷修复（Quick Fix）

类型/函数找不到且来自某个包时，报错处会提供 **`引入 import "包名"`**：

- 扫描编译器旁的 `std/*.lcl` 符号表与工作区 `*.aya`，定位符号所在包
- 光标放到错误行 → `Ctrl+.`（或点击灯泡）→ 自动在 import 区插入 `import "arraylist";`
- 排序：包名与符号同名优先、非 `std` 优先、依赖更少的包优先

## 诊断与补全（编译器驱动）

- **诊断范围**：波浪线覆盖完整标识符；声明级问题（缺 return、注解缺失）覆盖整条 `fn ...` 声明
- **效应/注解告警**：`check` 成功时也读取 stderr 告警，并锚定到函数声明
- **成员补全**：`变量.` 使用编译器 `types` 推断的真实类型；方法来自 std `.lcl` 方法表（含泛型 impl 方法，如 `ArrayList.push`）与本地/导入 `impl`
- **方法悬停**：显示 `.lcl` 方法表中的完整签名
- 顶层补全过滤 `__` 内部符号并去重

## 悬停（Hover）

类 rust-analyzer / clangd 的 Markdown 展示：

- **签名代码块**：`` ```ayanami `` 包裹完整签名（函数/方法/结构体/枚举/变量）
- **标注列表**：`#[alloc]` 等附一行说明（悬停标注本身也会显示分组/说明/示例）
- **文档注释**：声明上方的 `//` / `///` 原样作为 Markdown 渲染（支持列表、代码块、空行分段）
- **结构体/枚举**：字段与变体以 4 空格缩进列在代码块中
- **来源位置**：`*file.aya:行号*`
