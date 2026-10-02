# Ayanami 标注系统（标注式编程）设计

> 状态：设计阶段。A0（基础设施）尚未实现；本文是该特性的权威设计文档，
> 每个阶段落地后必须回来更新「实现状态」一节。

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
| 优化/代码生成 | `inline` `cold` `noreturn` `pure` `readonly` `nounwind` `willreturn` `noalias` `nonnull` | 映射 LLVM 函数/参数属性（含 extern 声明，跨语言优化） | A1 | 设计 |
| 条件/契约 | `cfg` `requires` `ensures` `invariant` `assume` | 条件编译；debug 运行时检查 + release `llvm.assume` | A2 | 设计 |
| 效应 | `throws` `eff` | 效应检查与传播（Java 式必须处理或上抛）、`?` 统一 | A3 | 设计 |
| 生命周期 | 类型参数 `'a`（不是标注） | 显式生命周期与 outlives 检查、字段引用 | A4 | 设计 |
| 用户宏 | `#[my_macro(...)]` | token 展开、卫生性（或平台插件） | A5 | 远期 |

## 4. 优化标注（A1）与 LLVM 映射

| 标注 | 位置 | LLVM | 说明 |
|---|---|---|---|
| `#[inline]` | 函数 | `inlinehint` | 桥接现有 `inline` 关键字 |
| `#[inline(always)]` | 函数 | `alwaysinline` | 强制内联 |
| `#[cold]` | 函数 | `cold` | 冷路径，影响分支布局 |
| `#[noreturn]` | 函数 | `noreturn` | 不返回 |
| `#[pure]` | 函数 | `memory(none)`（旧：`readnone`） | 无副作用、不读内存；可跨调用 CSE/下沉 |
| `#[readonly]` | 函数 | `memory(read)`（旧：`readonly`） | 只读内存 |
| `#[nounwind]` | 函数 | `nounwind` | 不抛异常（FFI 常见） |
| `#[willreturn]` | 函数 | `willreturn` | 必然返回，可提升循环 |
| `#[noalias]` | 指针参数 | `noalias` | 该指针不与其它指针别名 |
| `#[nonnull]` | 指针参数 | `nonnull` | 参数非空 |

**跨语言优化**：以上标注用在 `extern "C"` 声明上时，属性进入
`declare` 行，LLVM 即可跨越 FFI 调用做优化。例：

```ayanami
#[pure] #[nounwind]
extern "C" fn strlen(unique [char] s) -> int;
// 多次 strlen 调用可被合并/提升（调用者承诺正确性）
```

前置条件：extern 声明目前发射为无签名的 `declare i64 @name()`，
A1 必须先按 HIR 签名发射 `declare <ret> @name(<params>)`。

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

## 10. A0 验收标准

- [ ] grammar/手写解析器均支持 `#[...]`，生成解析器同步
- [ ] 属性在 AST/HIR/`fmt`/`defs` 全链路保真（fmt 往返一致）
- [ ] 白名单校验：未知属性报错并带行列号
- [ ] `#[inline]` 桥接现有 `is_inline`（生成 IR 可见 inline 提示）
- [ ] 其余 A1 属性接受并存储（暂不映射 LLVM）
- [ ] 全量 example `check` 0 失败；`check_all.sh` 全绿
