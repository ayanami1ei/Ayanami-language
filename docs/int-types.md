# 定宽整数类型（M1）

> 状态：M1.1 核心已实现（2026-10-04）。字面量后缀（0i6.5）与显式 `as`（0i6.6）待做。

## 目标

Rust 风格的基本整数类型：显式位宽与符号、无隐式提升、溢出语义明确，
为 std 迁移（usize 索引）与自举打基础。

## 类型集合

| 类型 | 位宽 | 有符号 | LLVM 类型 | 大小/对齐 |
|---|---|---|---|---|
| `i8` / `i16` / `i32` / `i64` / `i128` | 8/16/32/64/128 | 是 | `iN` | N/8 字节；i128 对齐 16 |
| `u8` / `u16` / `u32` / `u64` / `u128` | 8/16/32/64/128 | 否 | `iN` | 同上 |
| `isize` | 64 | 是 | `i64` | 8 |
| `usize` | 64 | 否 | `i64` | 8 |

- `int` 暂时保持现有默认整数（i64 别名）；默认类型迁移见 0i6.10。
- x86_64 Linux 下 `isize`/`usize` 为 64 位；当前类型身份与 `i64`/`u64` 相同（显示为 `i64`/`u64`）。

## 字面量

- 无后缀整数字面量类型为 `int`，在**有期望类型**的位置自动适配为对应定宽整数：
  - 函数实参（`add8(1, 2)`，形参 `i8`；重载解析允许字面量匹配任意整数形参）
  - 赋值 / 返回 / 结构体字段（`coerce_expr` 路径）
  - 二元运算另一侧（`x + 1`、`x == -1`，含一元负号字面量）
- 超出目标类型范围的适配字面量**暂未做范围检查**（后续字面量任务处理）。
- 后缀字面量（`1u8`）与显式 `as` 转换见 0i6.5 / 0i6.6。

## 语义

- **无隐式转换**：定宽整数与 `int`/`float`/其他宽度之间必须显式转换；
  未实现 `as` 前直接报 `cannot implicitly convert ...`。
- **算术**：`+`/`-`/`*` 为二补数回绕（LLVM `add`/`sub`/`mul`）；
  `/`、`%` 按符号选 `sdiv`/`srem` 或 `udiv`/`urem`；比较按符号选 `slt`/`ult` 等。
- 除零、`i8::MIN / -1` 溢出等行为见 `docs/runtime-safety.md`（后续任务补齐诊断）。
- 位运算与移位：0i6.2。

## 位运算（M1.2）

- 运算符：`&` `|` `^` `<<` `>>`（二元）、`~`（一元）；优先级（紧→松）
  `* / %` > `+ -` > `<< >>` > `&` > `^` > `|` > 比较 > `&&` > `||`。
- 适用类型：整数（`int` / 定宽整数 / `char`）；`&` `|` `^` 也适用于 `bool`（发射 `i1`）。
  `float` 用位运算、`~bool`、移位 `bool` 直接报错。
- 右移：有符号（`int` / `iN`）用算术右移 `ashr`，无符号（`uN`）与 `char` 用逻辑右移 `lshr`。
- 移位量须与左操作数同类型（无后缀字面量自动适配）；超范围按位宽掩码（`x << (n & (bits-1))`，与 Rust release 一致）。
- 常量折叠：`example/constfold_lib.aya` 已支持位运算折叠（op 编码 14–18）。

## 显式转换 `as`（M1.3）

- 语法：`expr as T`；优先级高于 `* / %`、低于一元 `- ! ~`（`(a as i32) + b`、`-1 as u8` 均为 `(-1) as u8`）。
- 允许：整数 ↔ 整数（含 `char` / `bool` 作为源）、整数 ↔ `float`；`x as bool` 与结构体/引用转换报错。
- 语义：
  - 整数间：同宽 `bitcast`，变宽按源符号 `sext`/`zext`，变窄 `trunc`；
  - 整数 → 浮点：`sitofp` / `uitofp`（按符号）；
  - 浮点 → 整数：**饱和转换**（`llvm.fptosi.sat` / `fptoui.sat`，Rust 语义）。
- 隐式提升（char→int / int→float）暂保留；去掉需 std 迁移（见任务 0i6.3b）。

## 实现位置

| 环节 | 位置 |
|---|---|
| HIR 类型变体 | `src/hir/ty.rs` → `HirType::IntN { bits, signed }` |
| 类型名解析 | `src/hir/lower/helpers/types.rs` → `fixed_width_int`（AST `Named` 与 `.lcl` 签名共用） |
| 字面量适配 | `src/hir/lower/helpers/coerce.rs`（`as_int_literal` / `retype_int_literal`）、`expr_ops1.rs`、`wrap.rs`、`part_05.rs`（字面量感知重载解析） |
| 负数数 | `HirNode::as_neg_int_literal`（`src/hir/lower/to_mir/basic.rs`） |
| LLVM 发射 | `src/lir/emit/types.rs`、`src/lir/ir/nodes_a.rs`（算术 / 比较 / 一元负号） |
| 布局与名字 | `src/lir/ir/helpers.rs`、`src/lir/lower/names.rs`、`src/lir/serialize/reader.rs`（类型 tag 13） |

## 验收

- `example/test_int_width.aya`：12 种类型全链路 + 回绕 / 有符号无符号除法与比较（exit 0）。
- `tests/compile_fail/int_width_mismatch.aya`：`i32 + int` 隐式混用报错。
