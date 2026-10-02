# Ayanami 标注系统（标注式编程）设计

> 状态：**A0 已实现**（生成解析器桥接因 Asuka 生成器当前整体不可用而待补）；
> 本文是该特性的权威设计文档，每个阶段落地后必须回来更新「实现状态」一节。

## 1. 定位与原则

「标注式编程」：用简短的标注（attribute）向编译器声明**意图与契约**，
由编译器据此完成代码生成、优化、条件判定、效应检查，
而不是让标注承载任意行为（那是宏系统，见 A5）。

设计原则（按优先级）：

1. **白名单**：只有属性注册表内的标注有编译器语义；未知标注**报错**（带位置）。
2. **类型系统优先**：能用类型表达的语义不要用标注。例如生命周期用类型参数 `'a`，
   不用 `#[lifetime(...)]`。
3. **默认信任**：`#[pure]`、`#[noalias]` 等优化承诺等同 `unsafe`——
   标注者承诺正确性，编译器不校验；误标导致 UB。
4. **跨包保真**：标注必须进入 AST→HIR→`.lcl`（`generic_sources`）与 `defs` JSON，
   formatter 往返不丢；跨包内联与跨包效应检查依赖这一点。
5. **可审计**：`fmt` 输出、`--debug` 输出、`defs` 输出都展示标注。

## 2. 语法

```ayanami
#[inline]
fn add(int a, int b) -> int { return a + b }

#[pure]
extern "C" fn sqrt(float x) -> float;

#[requires(n > 0)]
#[ensures(result >= n)]
fn grow(int n) -> int { return n + 1 }

#[throws(IO, Parse)]
fn read_config(ref String path) -> Config { ... }
```

- 形式：`#[name]` 或 `#[name(arg1, arg2)]`（参数为逗号分隔的 token）
- A0 允许位置：函数、方法、结构体/字段、枚举/变体、接口/方法、`extern "C"` 声明
- 后续允许：语句与表达式级（`#[cfg]` 等，A2）
- 词法：`#` 作为标点；`#[` 与 `]` 之间做括号配平校验

## 3. 语义分类与路线图

| 类别 | 标注 | 编译器行为 | 阶段 | 状态 |
|---|---|---|---|---|
| 基础设施 | 全部 | 解析、校验、存储、序列化、展示 | A0 | 设计 |
| 优化/代码生成 | `inline` `cold` `noreturn` `pure` `readonly` `nounwind` `willreturn` `noalias` `nonnull` | 映射 LLVM 函数/参数属性（含 extern 声明，跨语言优化） | A1 | 部分实现（函数级） |
| 条件/契约 | `cfg` `requires` `ensures` `invariant` `assume` | 条件编译；debug 运行时检查 + release `llvm.assume` | A2 | 设计 |
| 效应 | `throws` `eff` | 效应检查与传播（Java 式必须处理或上抛）、`?` 统一 | A3 | 设计 |
| 生命周期 | 类型参数 `'a`（不是标注） | 显式生命周期与 outlives 检查、字段引用 | A4 | 设计 |
| 用户宏 | `#[my_macro(...)]` | token 展开、卫生性（或平台插件） | A5 | 远期 |

## 4. 优化标注（A1）与 LLVM 映射

| 标注 | 位置 | LLVM | 说明 |
|---|---|---|---|
| `#[inline]` | 函数 | `inlinehint` | 提示内联（已实现） |
| `#[inline(always)]` | 函数 | `alwaysinline` | 强制内联（已实现；`inline` 关键字仍为 `alwaysinline`） |
| `#[cold]` | 函数 | `cold` | 冷路径，影响分支布局 |
| `#[noreturn]` | 函数 | `noreturn` | 不返回 |
| `#[pure]` | 函数 | `memory(none)`（旧：`readnone`） | 无副作用、不读内存；可跨调用 CSE/下沉 |
| `#[readonly]` | 函数 | `memory(read)`（旧：`readonly`） | 只读内存 |
| `#[nounwind]` | 函数 | `nounwind` | 不抛异常（FFI 常见） |
| `#[willreturn]` | 函数 | `willreturn` | 必然返回，可提升循环 |
| `#[noalias]` | 指针参数（ref/unique/[T]/fn） | `noalias` | 该指针不与其它指针别名（已实现，A1b） |
| `#[nonnull]` | 指针参数（ref/unique/[T]/fn） | `nonnull` | 参数非空（已实现，A1b） |

**跨语言优化**：以上标注用在 `extern "C"` 声明上时，属性进入
`declare` 行，LLVM 即可跨越 FFI 调用做优化。例：

```ayanami
#[pure] #[nounwind]
extern "C" fn strlen(unique [char] s) -> int;
// 发射：declare i64 @strlen(ptr) memory(none) nounwind
// 多次 strlen 调用可被合并/提升（调用者承诺正确性）
```

实现要点（A1a 已完成）：

- `LirProgram.extern_decls`（`lir/ir/nodes_d.rs`）携带真实签名（参数/返回 `HirType`）
  与 `LirAttr`，由 `lir/lower/mod.rs` 从源码 `extern "C"` 声明与未定义的包导入函数构建；
- 发射层按 `declare <ret> @name(<params>)<attrs>` 输出（`lir/emit/mod.rs`）；
- 属性后缀统一由 `lir/emit/functions.rs::llvm_attr_suffix` 生成，定义与声明共用；
- 字符串字面量的堆副本追加 NUL，保证可直接传给 C 字符串 API；
- 包导入函数暂不带标注（`ImportedFnSig.attrs` 为空）；
- 形参标注（A1b）：`#[noalias]`/`#[nonnull]` 写在形参类型前（`fn f(#[nonnull] ref int x, #[noalias] unique [char] s)`），
  经 `AST.param_attrs → HirFn/MirFn/LirFn.param_attrs → ExternDecl.param_attrs` 全链路到达
  `define`/`declare` 的参数列表（如 `declare i32 @memcmp(ptr nonnull noalias, ptr nonnull noalias, i64)`）；
  仅允许指针类型，接口方法与 lambda 的形参暂不支持（解析期报错）。

优化效果实证（`opt -O2` 下，调用非内置 C 函数 `mystery` 两次）：

| 声明 | 优化后 `call @mystery` 次数 |
|---|---|
| `#[pure] #[nounwind] #[willreturn]` | 1（GVN 跨 FFI CSE） |
| 无标注 | 2 |

驱动已接入 `opt -O2`（A1c）：有 `opt` 时先中端优化再交给 `llc`，
上述跨 FFI CSE 默认生效；`opt` 缺失或运行失败时自动回退未优化 IR，
`AYANAMI_OPT=0` 可显式关闭。产物保留 `*.opt.ll` 便于查看优化结果。

## 5. 条件与契约（A2）

- `#[cfg(target = "linux")]`：编译期裁剪 item/语句；
  条件表达式限定字面量与编译期常量，禁止副作用。
- `#[requires(cond)]`：函数前置条件。debug 构建插入运行检查（失败 abort 并报位置）；
  release 构建转为 `llvm.assume`（可被优化器利用）。
- `#[ensures(cond)]`：后置条件，`result` 绑定返回值。
- `#[invariant(cond)]`：循环不变式（语句级标注），debug 每轮校验、release `llvm.assume`。
- `#[assume(cond)]`：无条件向优化器声明事实。

契约条件必须是无副作用的 bool 表达式；debug 与 release 语义差异必须在文档与报错中明确。

## 6. 效应系统（A3）

目标：README 所述代数效应的第一步——**Java 式可检查副作用**。

- 声明：`#[throws(E1, E2)]`（异常效应）与后续的 `#[eff(state, io, ...)]`。
- 传播：对声明了效应的函数调用，调用点必须：
  1. 用 `?` 上抛（调用者因此获得该效应），或
  2. 显式处理（`try/handle` 语法在 A3 后期提供）。
- 检查：编译器自底向上推断每个函数的效应集合；`#[throws]` 声明与实际集合求差集，
  缺失报错，多余也报错（保持声明可信）。
- 与 `Result` 的关系：`?` 现在等价于 `try_unwrap`。A3 统一为
  「`?` = 把未处理效应上抛」；`Result` 退化为一种内置效应。
- 默认推断以减少标注噪音：无 `#[throws]` 声明的函数若产生效应，编译期报错并给出建议标注。

## 7. 生命周期（A4，类型参数而非标注）

```ayanami
fn pick['a](ref['a] S s) -> ref['a] S { return s }

struct Holder['a] {
    ref['a] S item
}
```

- 显式生命周期参数用于：函数返回引用、结构体字段存引用。
- 现有「单引用参数省略」规则保留为缺省；多参数需要显式 `'a`。
- 借用检查器扩展：loan 与 lifetime 的 outlives 约束、字段投影的 loan 传播。
- 完成 A4 后，引用可合法存入字段，「与 Rust 的最大差距」补齐。

## 8. 实现接口（跨包与工具）

| 层 | 载体 |
|---|---|
| AST | `Attr { name, args, span }`，挂在声明的 `attrs` 字段 |
| HIR | `HirFn.attrs`、`HirStructField.attrs` 等，供各 pass 读取 |
| 包 | `.lcl` 的 `generic_sources`（formatter 输出包含标注）+ `defs` JSON |
| 工具 | `defs` JSON 带属性字段；LSP/VSCode 插件、未来的代码设计平台消费 |

## 9. 决策记录（ADR）

| 编号 | 决策 | 理由 |
|---|---|---|
| ADR-1 | 语法用 `#[...]` | 与 Rust 一致，`inline` 等关键字可平滑迁移 |
| ADR-2 | 未知标注报错 | 保证「只有注册表内的标注有语义」，避免魔法蔓延 |
| ADR-3 | 优化承诺默认信任 | 与 Rust unsafe 一致；debug 校验留作可选后置 |
| ADR-4 | 生命周期用类型参数 `'a` | 类型系统优先于标注；可读性与工具支持更好 |
| ADR-5 | 先做 A0 基础设施 | 全链路保真后再挂语义，避免返工 |

## 10. 实现状态

### A0（已完成，2026-10）

- [x] grammar 支持 `#[...]`（`AttrList/Attr/AttrArgs`），`#` 加入标点
- [x] 手写解析器：函数/方法/结构体/枚举/接口/impl 前导属性（属性名可含关键字）
- [x] AST `Attr { name, args, span }` 全链路；`fmt` 往返保真
- [x] HIR 校验白名单；未知属性报错带行列号（ADR-2）
- [x] `#[inline]` 桥接 `is_inline`，IR 输出 `alwaysinline`
- [x] `defs` JSON 带 `attrs`
- [ ] `requires/ensures/...` 参数表达式（A2 扩展文法）
- [ ] Asuka 生成解析器桥接属性（见下方已知问题）

### A1（部分完成，2026-10）

- [x] extern 声明按 HIR 签名发射 `declare <ret> @name(<params>)`
- [x] `MirFn`/`LirFn`/`LirProgram` 全链路保留标注（`LirAttr`、`ExternDecl`；`.lcl` 格式随之更新）
- [x] 函数级 LLVM 映射：`cold` `noreturn` `pure`(`memory(none)`) `readonly`(`memory(read)`)
      `nounwind` `willreturn` `inline`(`inlinehint`) `inline(always)`(`alwaysinline`)
- [x] 字符串字面量堆副本 NUL 结尾（FFI 互操作）
- [x] `example/test_ffi_attrs.aya`（`strlen`/`abort`/`cold`/`inline`/`willreturn`）
- [x] 参数级 `noalias`/`nonnull`（`FnParam = Attr* Type Ident`；仅指针类型）
- [ ] 包导入函数的标注传递（`ImportedFnSig.attrs` 目前为空；形参/函数级均不随 .lcl 导出）
- [x] 驱动接入 `opt -O2`（缺失/失败回退未优化；`AYANAMI_OPT=0` 关闭）
- [ ] 接口方法与 lambda 的形参标注（当前解析期拒绝）
- [ ] `inline` 关键字与 `#[inline]` 语义统一

### 已知问题

- **Asuka 生成解析器当前对所有真实程序解析失败（`no alt`）**，
  编译器实际依赖手写回退解析器；因此属性在生成解析器桥接（`gen_bridge`）
  中暂为透传空列表（代码中有 TODO）。修复生成解析器后需补桥接。
- 属性参数当前仅支持标识符/整数/字符串字面量；表达式参数随 A2 扩展。
- `.lcl` 包格式在 A1 变更后不向后兼容，需随编译器一起重新生成（std 已重建）。
