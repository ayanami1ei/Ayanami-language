# const / static / 编译期求值设计（M6）

> 状态：**M6.1 / M6.1b / M6.2a / M6.2b（pub static）/ M6.3a 已实现**；M6.2c（复合初始化）、
> M6.3b（常量表）待做。见文末排期与各节实现状态。

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
| `static` / 全局 | ✅ M6.2a（标量、常量初始化、可寻址、`static mut`） |
| 编译期函数求值 | ✅ M6.3a（`const fn` 标量：编译期解释 + 运行期普通调用） |

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
- 命名空间内 `const` 支持（M6.1b）：限定名 `ns.NAME`（可嵌套），命名空间内可裸名使用。

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

## M6.1b `pub const` 导出（已实现，2026-10）

- `PackageSymbol::Const { name, ty, value }` → `.lcl` `const="name,ty,value"` 文本行
  （值编码见 `src/package/const_codec.rs`：Int 十进制 / Float 最短往返 / Char 码点 / Bool）；
- 导入时注册到 `Ctx.consts`（裸名 + 限定名 `pkg.CONST`；本地同名优先）；
- 命名空间 const：限定名 `ns.NAME`（可嵌套 `ns.sub.NAME`），顶层导出裸名；
- 传递依赖经 `merge_symbols` 再导出；
- 回归：`example/test_const_ns.aya`、`tests/lcl_const/`（打包 lib → 导入裸名/限定名/命名空间）。

## M6.2 static / global（M6.2a 已实现，2026-10）

已实现（标量、常量初始化）：

- `static NAME [: T] = <常量表达式>`（不可变）与 `static mut NAME [: T] = ...`（可写）；
- 读取 = 全局地址 + 载入；`ref NAME` / `ref mut NAME` = 全局地址；`f(NAME)` 传 `ref`/`ref mut` 形参自动借用全局；
- `static mut` 赋值 → 穿透引用写入（`DerefAssign`）；
- 不可变 static 赋值/取 `ref mut` 报错；初始化复用 M6.1 常量求值器（含 IntN 范围检查）；
- 发射：`@name = global <ty> <init>`；引用全局的 ref 局部不参与局部 loan 跟踪（全局始终存活）；
- 回归：`example/test_static.aya` + 两个负例。

M6.2b（已实现，2026-10）：`pub static` 跨模块导出/导入：

- `.lcl` `static="name,ty,mut"`（标量类型；值不需要——定义在被导入包的 .o 中）；
- 导入注册裸名 + 限定名 `pkg.NAME`（别名指向同一发射符号）；命名空间 `ns.NAME`；
- 导入方发射 `@name = external global <ty>` 声明，不定义；release 内部化跳过 pub static；
- 回归：`tests/lcl_static/`（不可变/可变/命名空间跨模块读写）、`example/test_static.aya`。

M6.2c（数组部分已实现，2026-10）：复合初始化——数组全局：

- `static TABLE: [int; N] = [a, b, ...]`、重复字面量 `[v; N]`、`static mut` 数组读写；
- 类型位新增 `[T; N]`（N 为字面量整数）；常量求值产出 `HirLiteral::Array`；
- 发射：数据数组常量 `@__ayanami_gdata_<name> = private global [N x T] [...]`
  + 指针变量 `@<name> = global ptr @__ayanami_gdata_<name>`（语言层 `[T; N]` 为指针语义，
  索引/引用/跨模块导入复用既有路径）；release 内部化仅对非 pub；
- `.lcl` `static=` 类型编码支持数组 `[T;N]`（跨模块导出/导入）；
- 回归：`example/test_static_array.aya`、`tests/lcl_static/`（pub static 数组跨模块）。

结构体全局（已实现，2026-10）：

- `static P: Point = Point { x = 1, y = 2 }`（标量字段 + 嵌套结构体字段；字段按声明顺序规范化）；
- 发射为**内联**结构体常量 `@P = global %struct.Point { i64 1, i64 2 }`（结构体值语义，
  取址即全局地址；字段读/写/`ref`/`ref mut` 复用既有路径）；
- `.lcl` `static=` 类型编码支持命名类型（结构体布局随 struct_defs 导入）；
- 回归：`example/test_static_struct.aya`、`tests/lcl_static/`（Coord 跨模块读写）。

结构体常量中的数组字段（已实现，2026-10）：字段类型为指针语义，发射层预发射数据数组
`@__ayanami_gdata_<static>_<field路径>` 并在结构体常量中引用（`ptr @...`）；支持嵌套结构体路径。

待做（M6.2c 剩余）：`static mut` 的 unsafe 门控（M5）、数组/结构体常量用于 `const`（M6.3b）。

- **所有权**：static 值不可含堆所有权（`String` / `[T]` 动态数组）——避免无 drop 的全局；
  需要表时用固定大小 `[T; n]` 或 M6.3 生成的常量。
- **`main` 前初始化**：仅常量初始化（无运行时代码），无需 ctor。
- 示例：`static COUNTER = 0`、`static TABLE: [int; 16] = [0; 16]`、`static mut STATE = 0`。

## M6.3 编译期求值 `#[compile_time] fn`（M6.3a 已实现，2026-10）

统一 `const` 关键字，**编译器按上下文区分编译期与运行期**：

- 声明：`#[compile_time] fn name(params) -> T { ... }`（与 `const NAME = ...` 分开：值用 `const`，函数用标注）。
  被标注函数同时作为**普通运行期函数**编译（运行期调用正常发射），并登记为编译期可求值体。
- 调用点：出现在 `const` / `static` 初始化式（含嵌套调用）→ 编译期解释执行；
  出现在运行期代码 → 普通函数调用。
- 求值器（`hir/lower/body/const_fn.rs` + `const_eval.rs`）：标量（int/float/char/bool/IntN）、
  局部变量/赋值、if/elif/else、while、for 区间、break/continue、return 与块尾表达式、
  const fn 互调（递归）、`if` 表达式分支块值。
- 限制：递归深度 64、循环 100 万次（超出报错）；泛型/extern 标注报错；标注仅允许函数；
  命名空间内需限定调用 `ns::f(...)`；不支持堆/数组/字符串/方法调用（见 M6.3b）。
- 回归：`example/test_const_fn.aya`（fib 递归 + 运行期调用、for/while、尾 if、命名空间）、
  负例 `const_fn_not_const` / `const_fn_generic` / `const_fn_recursion`。

待做（M6.3b）：固定大小数组/字符串表（`const TABLE = make_table()` 产出 `[N x T]` 常量）、
`HirLiteral::Array` 与只读全局发射。

## 阶段与排期

| 阶段 | 内容 | 状态 |
|---|---|---|
| M6.1 | const 基础（标量 / 数组大小 / 0x） | ✅ 已实现 |
| M6.1b | `pub const` / 命名空间 const 导出导入 | ✅ 已实现 |
| M6.2 | static / global（可寻址、常量初始化、跨模块） | ✅ M6.2a/b 已实现；M6.2c 数组/结构体全局（含数组字段）已实现（unsafe 门控待做） |
| M6.3 | `#[compile_time] fn` / make_table()（解释执行 + 常量表） | ✅ M6.3a 已实现（标量；M6.3b 常量表待做） |

## 风险

- 常量表达式求值要与运行时语义一致（溢出/移位/除零），避免编译期与运行期不一致；
- `static` 的所有权限制需要清晰诊断（禁止堆拥有类型）；
- const fn 解释器与泛型/接口的交互（暂不支持泛型 const fn）。
