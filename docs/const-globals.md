# const / static / 编译期求值设计（M6）

> 状态：M6.1（const 基础）已实现；M6.1b（`pub const` 导出）/ M6.2（static）/ M6.3（const fn）
> 设计见文末排期。

## 目标与用途

- `const`：编译期常量——数组大小、协议 magic、数学常量、编译期配置。
- `static` / 全局：可寻址存储——计数器、查找表、运行时状态。
- 编译期求值：`const TABLE = make_table()` 生成常量表（lookup table、字符串/字节表、SIMD 常量）。

## 现状（2026-10）

| 能力 | 现状 |
|---|---|
| 字面量 | `0x`/`0b`/`0o`/下划线/类型后缀/指数已支持（M1.5） |
| 数组字面量计数 | `[T; n]` 中 n 为编译期常量时生成 `HirType::ArraySized`（`expr_lower.rs` 的 `as_const` 判定） |
| `const` | ✅ 已支持（M6.1，文件作用域，编译期替换） |
| `static` / 全局 | 无 |
| 编译期函数求值 | 无（MIR/LIR 有局部折叠，但不暴露给用户） |

## M6.1 const（已实现）

### 语法

```ayanami
const MAX_SIZE = 1024
const MAGIC = 0x12345678
const MASK = (1 << 8) - 1
const PI: float = 3.14159
pub const VERSION = 3
```

- 顶层（文件作用域）；`pub` 可选（`.lcl` 导出见 M6.1b）；类型标注可选；行尾 `;` 可选。
- 命名空间内 `const` 暂不支持（与 M6.1b 一起做）。

### 常量表达式（M6.1 范围）

- 字面量：int / float / char / bool，含类型后缀（`1u8`）与各进制；
- 一元：`-` `!` `~`；
- 二元：`+ - * / % & | ^ << >>` 与比较 `== != < > <= >=`；
- 引用此前声明的 const（按文件顺序求值）；
- 括号（解析树直接体现优先级）。

其他形式（变量、函数调用、cast 等）报
`const initializer must be a compile-time constant`。

### 语义

- **编译期替换**：const 不占运行时存储、不可取址；使用点内联为 `SConst`。
- **标识符解析顺序**：局部变量 > const > 函数（局部可遮蔽 const）。
- **类型**：由初始化式推断（整数字面量 → `int`；浮点 → `float`；`char` / `bool`）。
  带类型标注时按标注类型适配（字面量重定型），不匹配报错。
- **数组大小**：`[T; N]` 中 N 为 const / 字面量时生成 `ArraySized`；负数或非整数报错。
- 除零 / 非法移位等在编译期报错，语义与运行时一致（移位按位宽掩码）。

### 验收

- `example/test_const.aya`：标量、0x、位运算、引用其他 const、数组大小、循环边界；
- 负例 `tests/compile_fail/const_not_constant.aya`（非常量初始化式）、
  `const_float_size.aya`（浮点作数组大小）。

### 实现位置

| 环节 | 位置 |
|---|---|
| 词法 | `src/lexer/keyword.rs`、`src/lexer/lexer/next.rs`（`const`） |
| 语法 | `src/parser/parser/decl.rs::parse_const_decl`、`ast/stmt.rs::ConstDecl` |
| 收集/求值 | `src/hir/lower/body/const_eval.rs`（新增）、`collect_ns.rs` |
| 替换 | `src/hir/lower/body/expr_lower.rs`（Ident → `SConst`） |
| 格式化 | `src/formatter/stmt.rs` |

## M6.1b `pub const` 导出（设计）

- `PackageSymbol::Const { name, ty, value }` → `.lcl` `const="name,ty,value"` 文本行；
- 导入时注册到 `Ctx.consts`（限定名 `pkg.CONST` 或裸名）；
- 命名空间内 const 同时支持（限定名 `ns.NAME`）。

## M6.2 static / global（M6.2a 已实现，2026-10）

已实现（标量、常量初始化）：

- `static NAME [: T] = <常量表达式>`（不可变）与 `static mut NAME [: T] = ...`（可写）；
- 读取 = 全局地址 + 载入；`ref NAME` / `ref mut NAME` = 全局地址；`f(NAME)` 传 `ref`/`ref mut` 形参自动借用全局；
- `static mut` 赋值 → 穿透引用写入（`DerefAssign`）；
- 不可变 static 赋值/取 `ref mut` 报错；初始化复用 M6.1 常量求值器（含 IntN 范围检查）；
- 发射：`@name = global <ty> <init>`；引用全局的 ref 局部不参与局部 loan 跟踪（全局始终存活）；
- 回归：`example/test_static.aya` + 两个负例。

待做（M6.2b）：`pub static` 跨模块导出/导入（`.lcl` 符号 + `external global`）、
复合初始化（数组/结构体）、`static mut` 的 unsafe 门控（M5）。

## M6.2 static / global（设计）

```ayanami
static COUNTER = 0
static TABLE: [int; 16] = [0; 16]
static mut STATE = 0        // 可变全局（写入需 unsafe，见 M5）
```

- **可寻址**：LLVM 全局变量（`@name = global`），支持取址/`ref`。
- **初始化**：M6.2a 只允许常量初始化式（M6.1 求值器）；M6.3 后允许 `const fn`。
- **所有权**：static 值不可含堆所有权（`String` / `[T]` 动态数组）——避免无 drop 的全局；
  需要表时用固定大小 `[T; n]` 或 M6.3 生成的常量。
- **跨模块**：`pub static` 经 `.lcl` 导出符号（`static="name,ty"`），导入方按外部全局声明
  （LLVM `external global`），链接期解析。
- **`main` 前初始化**：仅常量初始化（无运行时代码），无需 ctor。

## M6.3 编译期求值（设计）

- 形式：`#[const] fn make_table() -> [int; 64]` + `const TABLE = make_table()`。
- 实现：对 `#[const]` 函数在 HIR（或 MIR）上做**解释执行**（常量栈 + 局部环境），
  限制：无堆分配、无 IO、无 panic 之外的副作用、循环/分支可用；数组用固定大小。
- 输出：`HirLiteral` 标量或常量数组（新增 `HirLiteral::Array(Vec<HirLiteral>)`），
  发射为 LLVM 常量（`[N x T]`）或全局只读数据。
- 分阶段：M6.3a 标量 const fn；M6.3b 固定大小数组/字符串表。

## 阶段与排期

| 阶段 | 内容 | 状态 |
|---|---|---|
| M6.1 | const 基础（标量 / 数组大小 / 0x） | ✅ 已实现 |
| M6.1b | `pub const` / 命名空间 const 导出导入 | 待做 |
| M6.2 | static / global（可寻址、常量初始化、跨模块） | ✅ M6.2a 已实现（标量；见下） |
| M6.3 | const fn / make_table()（解释执行 + 常量表） | 待做 |

## 风险

- 常量表达式求值要与运行时语义一致（溢出/移位/除零），避免编译期与运行期不一致；
- `static` 的所有权限制需要清晰诊断（禁止堆拥有类型）；
- const fn 解释器与泛型/接口的交互（暂不支持泛型 const fn）。
