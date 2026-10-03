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
| 条件/契约 | `cfg` `requires` `ensures` `invariant` `assume` | 条件编译；debug 运行时检查 + release `llvm.assume` | A2 | 已完成（`--release` 切换） |
| 效应 | `io` `state` `alloc` `pure` `no_error` `throws` | 注册表 + 默认最好情况推断；承诺/事实分开导出 | A3 | 部分实现（§6 已定稿） |
| 生命周期 | 类型参数 `'a`（不是标注） | 显式生命周期与 outlives 检查、字段引用 | A4 | 设计 |
| 用户宏/插件 | `#[pkg::macro(...)]` | 标注 provider 解析；宏展开（声明式或编译期执行）；插件注册属性 | A5 | 设计（§8） |

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

- `#[cfg(target = "linux")]`：编译期裁剪 item 与语句；已实现（A2b/A2f）。
- `#[requires(cond)]`：函数前置条件（已实现运行检查，A2d）。默认插入运行检查
  （失败打印位置并 abort）；`--release`（或 `AYANAMI_CHECKS=0`）时转为 `llvm.assume`。
- `#[ensures(cond)]`：后置条件，`result` 绑定返回值（已实现，A2e）。
- `#[invariant(cond)]`：循环不变式（语句级标注），debug 每轮校验、release `llvm.assume`（已实现，A2f）。
- `#[assume(cond)]`：无条件向优化器声明事实（已实现，A2c）。

契约条件必须是无副作用的 bool 表达式；debug 与 release 语义差异必须在文档与报错中明确。

### A2a 标注实参（已完成，2026-10）

`Attr.args` 从字符串升级为 `AttrArg`：

- `AttrArg::KeyValue(key, value)`：`cfg(target = "linux")`；
- `AttrArg::Expr(expr)`：任意表达式（`requires(x > 0)`、`inline(always)`），
  由手写解析器 `parse_attr_arg` 解析，formatter 往返保真；
- LIR 侧经 `lir/lower/util.rs::attrs_to_lir` 渲染为字符串，`LirAttr` 格式不变。

### A2b `#[cfg]`（已完成，2026-10）

- 求值点：HIR 降低前 `hir/cfg.rs::filter_program`，递归过滤顶层 /
  namespace / impl / interface 方法；`validate_program` 先行，未知谓词报错（ADR-2）。
- 谓词：`target = "linux"`、`arch = "x86_64"`、裸标识符
  （`unix`/`windows`/`linux`/`macos`/`x86_64`/`aarch64`）、`!` 取反；
  多个谓词/多个 `#[cfg]` 之间为「与」。
- 包导出（`package/symbols.rs`）同样跳过被裁剪项；
  `defs` 命令仍列出两侧（IDE 视角），语句级 cfg 待后续。

### A2c `#[assume]`（已完成，2026-10）

- 函数级标注：`#[assume(cond)]` 可重复，条件在形参绑定后求值；
- 校验（`hir/contracts.rs`）：恰好一个表达式实参；条件须为 bool 值或比较运算
  （HIR 中比较保持操作数类型，Bool 结果由 MIR→LIR 决定，故用 `HirNode::is_comparison` 判定）；
- 链路：`HirStmt::Assume` → `SMirAssumeStmt` → `SLirAssume`（tag 28）→
  `call void @llvm.assume(i1 ...)`，位于函数入口 alloca/参数存储之后；
- 运行时为零开销；`opt -O2` 可据此优化（如消除冗余分支）。
- `example/test_assume.aya`；反例：非 bool 条件、无实参、`key = value` 实参均报错。

### A2f 语句级标注（已完成，2026-10）

- AST 新增包装 `Stmt::Attributed { attrs, stmt, span }`；文法 `Stmt = AttrList StmtKind`；
  手写解析器对非声明语句保留前导标注（声明仍由各自解析器消费）。
- 校验（`hir/attrs.rs`）：语句级仅允许内置 `cfg`/`invariant`；`invariant` 仅限 `while`/`for`；
  库宏暂不允许出现在语句位置。
- 语句级 `#[cfg]`：`hir/cfg.rs::filter_one` 递归过滤，覆盖函数体内语句与 `Attributed` 包装。
- `#[invariant(cond)]`：每轮循环体首注入检查（`HirStmt::Contract{kind: Invariant}`）；
  `for` 的迭代变量在作用域内可引用；`AYANAMI_CHECKS=0` 时退化为 `llvm.assume`。
- formatter/泛型替换/调试打印均处理包装；`example/test_invariant.aya`。

### A2e `#[ensures]`（已完成，2026-10）

- `result` 绑定返回值：为函数分配隐藏局部 `result`（可变），在 ensures 条件求值前绑定；
- 注入点：递归重写所有 `return v` 为 `result = v; 检查…; return result`（含 if/elif/else/while/嵌套块）；
- 兜底路径：无显式 return（或并非所有路径返回）时追加 `result = 默认值; 检查…; return result`，
  默认值仅支持基元类型（int/float/bool/char），其余报错要求显式 return；
- void 返回类型 + `#[ensures]` 报错；`AYANAMI_CHECKS=0` 时检查退化为 `llvm.assume`；
- 契约链路统一为 `HirStmt::Contract { kind, cond, line, col }`
  （`ContractKind::{Require, Ensure, Invariant}`）→ `SMirContractStmt` → `SLirContractCheck`（tag 29，kind 字节）；
- `example/test_ensures.aya`；违反时 runtime 输出 `ensures failed at L:C` 后 abort。

### A2d `#[requires]`（已完成运行检查，2026-10）

- 校验与 assume 同规（恰好一个表达式实参；bool 或比较运算）；
- 链路：`HirStmt::Require` → `SMirRequireStmt` → `SLirRequireCheck`（tag 29）→
  `br i1 %c, label %contract_ok_N, label %contract_fail_N`；失败分支调用
  `__ayanami_require_fail(line, col)`（runtime.c，noreturn，打印 `requires failed at L:C` 后 abort）；
- `--release` / `AYANAMI_CHECKS=0`：退化为 `llvm.assume`（发布语义；环境变量优先）；
- `example/test_requires.aya`（含多条件）；违反契约时退出码为 SIGABRT。
- `ensures`/`invariant` 待后续；当前无独立 debug/release 模式，以 `AYANAMI_CHECKS` 区分。

## 6. 效应系统（A3）

目标：以注解为契约、推断为事实、**默认最好情况**的可扩展效应框架。

### 6.1 效应与具体注解（不要泛化 `eff`）

效应是「名字」。编译器维护可注册的 `EffectRegistry`：内置 `io`/`state`/`alloc`
（含推断规则与 LLVM 记忆属性规则），A5 插件可注册新效应；**不设编译期白名单**。

| 标注 | 含义 |
|---|---|
| `#[io]` | 可能读/写外部世界（终端/文件/系统调用） |
| `#[state]` | 可能修改可观察状态（全局/堆/`ref mut`；函数内局部变量暂不算） |
| `#[alloc]` | 可能堆分配（`unique`/数组/字符串） |
| `#[pure]` | 承诺无任何效应（封闭承诺）→ `memory(none)` |
| `#[no_error]` | 承诺永不失败（封闭承诺）→ `nounwind` |
| `#[throws]` / `#[throws(_)]` | 可能失败，错误类型未知（占位；调用方用 `?`，无需命名 E） |
| `#[throws(E1,E2)]` | 具体错误集合（多/少声明都允许） |
| `#[throws()]` | 今天无具体错误 + **开放槽位**（未来可能失败，符号不变） |

已移除 `#[eff(...)]` 泛化写法。`#[throws()]`/`#[throws]`/`#[throws(E)]` 都**不产生**
`nounwind`；只有 `#[no_error]`（或推断证明本次无失败）才可以。

### 6.2 默认与推断（事实）

- **什么都不写 = 从「最好情况」（无效应、无错误）起步**。
- 编译器扫函数体（调用链、IO/alloc/state 操作、`Err` 构造）得到**实际集合（推断事实）**。
- **有效效应 = 声明 ∪ 推断**。声明用于：补 FFI 等推断看不见的效应、提前/多声明、给出承诺。
- `extern` 无声明时保守：不产生优化属性，但不引入 `unknown` 效应类别。
- `state` 范围：全局/堆/`ref mut` 的可观察修改；需要时再纳入局部变量。

### 6.3 承诺、冲突与告警

- `#[pure]`/`#[no_error]` 是封闭承诺；与推断矛盾时报到具体调用点
  （默认 warning，`--verify-effects` 升级为 error）。
- 推断了效应而接口未声明：对 **pub/导出接口与 extern** 给出行级建议
  （“这里应该加 `#[io]`”）；内部函数只进摘要，不打扰。
- 泛型标注作用于定义，实例继承（A3a-1 已实现）。

### 6.4 导出与优化（U1–U4）

- `.lcl` / `defs` 导出**分开存**：声明的效应、推断的事实、承诺位、`throws` 状态；
  工具可区分「契约」与「今天的事实」。
- 同编译单元：有效集合无错误来源 → `nounwind`；无 io/state/alloc → `memory(none)`。
- 跨包：消费者对导入函数只信任**承诺位**生成属性（静态链接可用推断事实，动态边界保守）。
- 承诺可跨动态边界；推断事实仅限同构建。
- **U2（`?` 错误传播）已实现（A3d，具体枚举）**：
  `expr?` 生成 `tmp = expr; if tmp._tag == 0 { ok = tmp._data_Ok._0 } else { return tmp }`，
  要求表达式类型与函数返回类型一致；Err 提前返回整个枚举值。
  泛型枚举已做**最小单态化**（`docs/generic-monomorphization.md`）：
  `Result[int,int]` 上 `?` 可用并以 `test_try.aya` 验收；调用实参/带标注赋值等期望类型上下文待补。

### 6.5 里程碑

- A3a 注解/推断/定位（已完成；本轮改为具体效应 + 注册表）
- A3b 承诺/推断 → 自动 LLVM 属性（本轮修正：`#[no_error]`→nounwind，`#[pure]`/推断→memory(none)）
- A3c `.lcl` 摘要（本轮扩展为 声明的 + 推断的 + 承诺）
- A3d `?`/`Result` 错误传播与空效应消除（待设计）
- A3e `try/handle`（后期）

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

## 8. 用户宏与插件系统（A5，设计）

目标：`#[...]` 不再只是编译器白名单——用户可以像 Rust 属性宏一样自己写标注；
「语法 + 来源库」区分编译器内置优化、用户宏与编译器插件。

### 8.1 命名与解析（provider）

- 标注名升级为路径：`AttrPath = Ident ("::" Ident)*`（`#[inline]`、`#[mylib::getter]`）。
- 解析域：
  1. **编译器内置**：裸名（`inline`/`cfg`/`requires`/`assume`/...），`core::` 前缀等价；
     内置名保留，库不可覆盖；未知裸名报错并提示是否缺少 `import`。
  2. **库宏**：`<pkg>::<macro>`，必须由已 `import` 的包导出；
     导入后允许裸名引用，重名时报错并要求写全路径。
  3. **插件属性**（远期）：插件注册的编译器扩展属性，独立命名空间（如 `plugin::attr`）。
- 宏只能展开出源码；若结果含内置标注，再由既有白名单与各 pass 处理。
  「编译器优化」与「宏」由此分层：前者是保留内置，后者是纯源码变换。
- **A5a 已实现**：`Attr.qualifier` + `AttrPath`（`pkg::macro`）解析与 formatter 往返；
  `core::name` 为内置别名；`hir/attrs.rs::Imports` 汇总 import 并做 provider 解析；
  `import "pkg" { a, b }` 短名列表（依赖改写保留列表）；库宏当前解析成功但报
  “lands in A5b”。

### 8.2 宏写法（执行模型：M3，Ayanami 先行）

已拍板：**M3 外部插件 ABI**；首个实现语言为 Ayanami，其他语言按同一 ABI 后续接入。

- 插件 ABI（草案）：插件导出
  `extern "C" fn __ayanami_macro_expand(input_ptr: *const u8, input_len: usize, out_len: *mut usize) -> *mut u8`，
  源文本进 / 源文本出（长度前缀）；编译器 `dlopen` 调用（`libloading` 或裸 `dlopen`）。
- Ayanami 宏：`pub` + `#[macro]` 函数，参数/返回为 `Source`（内建文本类型）；
  编译器按需用现有后端编译为动态库（`driver::ir_to_library(..., "dynamic-lib")`），
  导出到 `.lcl` 的宏表（新 section）。
- 展开：对带库宏标注的 item 依次调用 → 结果重新解析 → 内置标注校验 → 拼回 AST → 正常 HIR；
  递归展开设上限并检测循环，错误带宏名 + 调用点。
- 缓存 `target/macros/<hash>.so`；信任模型同 Rust proc-macro（编译期执行，文档明示）；
  WASM 沙箱与插件清单/权限为远期（A5c）。

### 8.3 展开时机与卫生性

- 阶段：解析后、`validate_program`/`cfg` 过滤前；item 级优先，语句/表达式级后期。
- 递归展开设上限并检测循环；错误带宏名 + 调用点。
- 卫生性：M1 需专门设计；M2 由源文本拼接决定（文档明示风险），后期可提供带 span 的 AST 序列化 ABI。

## 9. 实现接口（跨包与工具）

| 层 | 载体 |
|---|---|
| AST | `Attr { name, args, span }`，挂在声明的 `attrs` 字段 |
| HIR | `HirFn.attrs`、`HirStructField.attrs` 等，供各 pass 读取 |
| 包 | `.lcl` 的 `generic_sources`（formatter 输出包含标注）+ `defs` JSON；A5 计划新增宏表 section |
| 工具 | `defs` JSON 带属性字段；LSP/VSCode 插件、未来的代码设计平台消费 |
| 宏 | 计划：`AttrPath` 解析 → 宏表（`.lcl`）→ `target/macros/*.so`（A5） |

## 10. 决策记录（ADR）

| 编号 | 决策 | 理由 |
|---|---|---|
| ADR-1 | 语法用 `#[...]` | 与 Rust 一致，`inline` 等关键字可平滑迁移 |
| ADR-2 | 未知标注报错 | 保证「只有注册表内的标注有语义」，避免魔法蔓延 |
| ADR-3 | 优化承诺默认信任 | 与 Rust unsafe 一致；debug 校验留作可选后置 |
| ADR-4 | 生命周期用类型参数 `'a` | 类型系统优先于标注；可读性与工具支持更好 |
| ADR-5 | 先做 A0 基础设施 | 全链路保真后再挂语义，避免返工 |
| ADR-6 | 内置标注保留裸名，库宏用 `pkg::macro`；import 后可裸名，重名报错 | 兼容现有代码，命名冲突可诊断 |
| ADR-7 | 宏只做源码展开，不能直接产生编译器级优化属性 | 保持「优化承诺」可信；展开物再走白名单校验 |
| ADR-8 | 宏执行模型采用 M3 插件 ABI，Ayanami 先行 | 统一 ABI 便于多语言；先自举可复用现有后端 |
| ADR-9 | 效应注解权威，允许多/少声明；硬性出入只定位不拒绝 | FFI 不可见效应与未来扩展都需要宽容 |
| ADR-10 | 不引入 `unknown` 效应 | 无注解即无保证，优化保守即可 |
| ADR-11 | 泛型效应标注作用于定义，实例继承 | 与以注解为核心的语言一致 |
| ADR-12 | 效应不设白名单，走可注册的 `EffectRegistry` | 具体问题具体定义；A5 插件可扩展 |
| ADR-13 | 默认最好情况：默认空集 + 推断补全；注解用于 FFI/承诺/预留 | 少量样板，忘写也能推断，宽容 |
| ADR-14 | `#[no_error]` 才是 `nounwind` 承诺；`#[throws()]` 是开放槽位 | 接口未来加错误不改符号，优化只信承诺 |
| ADR-15 | `state` 只算可观察修改（全局/堆/ref mut），局部变量暂不算 | 先保守可用，需要再加 |

## 11. 实现状态

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

### A2（已完成，2026-10）

- [x] A2a 标注实参结构化（`AttrArg`：`key = value` / 表达式），formatter 保真
- [x] A2b `#[cfg(...)]` 编译期 item 裁剪（宿主 target/arch/裸名/取反；包导出一致）
- [x] A2c `#[assume(cond)]` → `llvm.assume`
- [x] A2d `#[requires]` 运行检查
- [x] A2e `#[ensures(result)]` 后置条件（契约链路统一为 ContractKind）
- [x] A2f `#[invariant]` 循环不变式 + 语句级 `#[cfg]`
- [x] 显式 debug/release 模式：`--release`（运行检查转 assume；`AYANAMI_CHECKS` 环境变量优先）

已知边界：`cfg` 谓词仅支持 target/arch 键值与裸标识符/`!`；
`ensures` 兜底默认值仅基元类型；语句级标注暂不支持库宏。

### A3（设计已定稿，进行中）

- [x] A3a-1 `#[throws]`/`#[eff]` 语法、校验、存储；泛型实例继承（顺带修复 A1 标注在特化实例丢失）
- [x] A3a-2 推断提醒与出入定位（io/throws；AST 调用点定位；`--verify-effects`）
- [x] A3b 空集注解 → 自动 LLVM 属性（U1：`#[throws()]`→nounwind、`#[eff()]`→memory(none)，含 extern 与 .lcl）
- [x] A3c `.lcl` 效应摘要（U3：包导出/导入自动属性）
- [ ] A3d `?` 空效应消除（U2）
- [ ] A3d `try/handle`（后期）

### A5（设计，待拍板）

- [x] A5a 标注名路径化（`pkg::macro`）+ provider 解析 + `core::` 别名 + import 短名列表
- [x] A5b-1 `#[macro]` 声明 + `.lcl` 宏表（`macro="name"`）+ `pkg::macro`/import 短名解析与存在性诊断
- [ ] A5b-2 M3 插件 ABI（Ayanami 宏编译为 `.so`、dlopen 调用）+ item 级展开与重新校验
- [ ] A5c 其他语言插件 / WASM 沙箱 / 语句表达式宏 / 插件清单与权限（远期）

### 已知问题

- **Asuka 生成解析器当前对所有真实程序解析失败（`no alt`）**，
  编译器实际依赖手写回退解析器；因此属性在生成解析器桥接（`gen_bridge`）
  中暂为透传空列表（代码中有 TODO）。修复生成解析器后需补桥接。
- 属性参数当前仅支持标识符/整数/字符串字面量；表达式参数随 A2 扩展。
- `.lcl` 包格式在 A1 变更后不向后兼容，需随编译器一起重新生成（std 已重建）。
- A2 剩余：`requires`/`ensures`/`assume`/`invariant` 与 debug/release 模式切换。
