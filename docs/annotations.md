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
| 生命周期 | `#[follow_with(...)]` | 注解式引用存活契约；NLL 来源标记与字段引用检查 | A4 | 部分实现（A4a/A4b/A4c；多来源联合与逃逸待后续） |
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
| `#[noalias]` | 指针参数（ref/[T]/fn） | `noalias` | 该指针不与其它指针别名（已实现，A1b） |
| `#[nonnull]` | 指针参数（ref/[T]/fn） | `nonnull` | 参数非空（已实现，A1b） |

**跨语言优化**：以上标注用在 `extern "C"` 声明上时，属性进入
`declare` 行，LLVM 即可跨越 FFI 调用做优化。例：

```ayanami
#[pure] #[nounwind]
extern "C" fn strlen([char] s) -> int;
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
- 形参标注（A1b）：`#[noalias]`/`#[nonnull]` 写在形参类型前（`fn f(#[nonnull] ref int x, #[noalias] [char] s)`），
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
| `#[alloc]` | 可能堆分配（数组/字符串） |
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

## 7. 生命周期（A4，注解式 `follow_with`）

不使用 Rust 风格 `'a` 类型语法；用注解声明“引用不会超过被引用对象”：

```ayanami
#[follow_with(s)]
fn pick(ref S s) -> ref S { return s }

struct Holder {
    #[follow_with(owner)]
    ref S item
}
```

- 多来源取最短；单引用参数省略规则保留。
- 完整设计、检查规则与阶段（A4a 语法/校验、A4b 借用检查接入、A4c 跨函数）
  见 `docs/lifetimes.md`。

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
- **A5b-3 宏参数约定**：宏函数形参为 `fn(String input, String a1, ..., String aN) -> String`：
  第 1 参是带宏 item 的源码文本（已去掉宏标注本身），其后依次是标注实参的源码文本；
  实参按书写形式传入（字符串字面量含引号，`k = v` 保留键值形式）。
  无实参时仍可用 `fn() -> String`（不接收 item 源码）或 `fn(String) -> String`（仅 item 源码）。
- **A5b-3 插件 ABI（v3 起结构体化，减少裸指针与参数个数）**：
  ```c
  typedef struct { char* data; long len; } AyaBuf;
  typedef struct { const AyaBuf* items; long count; } AyaBufList;
  AyaBuf __ayanami_macro_expand(AyaBuf input, AyaBufList args);
  void   __ayanami_macro_free(AyaBuf);
  ```
  shim 把输入复制到可转移所有权的缓冲区（宏内消费/释放均安全）；缓存键包含 ABI 版本。
  v2 的「裸指针 + 长度 + 数组指针」形式废弃。
- Ayanami 宏：`pub` + `#[macro]` 函数，参数/返回为 `Source`（内建文本类型；v2 暂以 `String` 承载）；
  编译器按需用现有后端编译为动态库（`driver::ir_to_library(..., "dynamic-lib")`），
  导出到 `.lcl` 的宏表（新 section）。
- 展开：对带库宏标注的 item 依次调用 → 结果重新解析 → 内置标注校验 → 拼回 AST → 正常 HIR；
  递归展开设上限并检测循环，错误带宏名 + 调用点。
- 缓存 `target/macros/<hash>.so`；信任模型同 Rust proc-macro（编译期执行，文档明示）；
  WASM 沙箱与插件清单/权限为远期（A5c）。

### 8.3 展开时机与卫生性

- 阶段：解析后、`validate_program`/`cfg` 过滤前；item 级与语句级（A5c-1）已实现，表达式级后期。
- 递归展开设上限并检测循环；错误带宏名 + 调用点。
- 卫生性：M1 需专门设计；M2 由源文本拼接决定（文档明示风险），后期可提供带 span 的 AST 序列化 ABI。

### 8.4 语句级宏（A5c-1）

- 语法：块内任意非声明语句前写 `#[pkg::macro(args...)]`（解析器已产出 `Stmt::Attributed`）。
- 语义：输入为「去掉目标宏标注后的语句源码」（保留其余标注，含 `#[cfg]`/`#[invariant]`）；
  输出源码重新解析为语句序列并递归展开后原位拼接；允许展开为多条语句。
- 遍历：函数体、`if`/`elif`/`else`、`for`/`while` 块内均展开；仅含编译器标注的
  `Attributed` 原样保留（`cfg` 过滤仍在其后执行）。
- 表达式级宏（函数宏）见 8.5；其他语言插件、WASM 沙箱、插件清单与权限为远期。

### 8.5 函数宏（A5c-2，表达式级）

- 语法：表达式位置 `#name(args)` / `#pkg::name(args)`；`fmt` 原样保留，不展开。
- 语义：在 HIR 降级时展开——解析宏所在 `.lcl` → 调用宏插件 → 返回源码按表达式回填并递归展开（上限 32）。
- 调用点信息：宏形参可用保留名 `__line`（int）/`__col`（int）/`__file`（String），
  编译器按调用点自动填充（等价 Rust `line!()`/`column!()`/`file!()`）。
- 插件 ABI v5：`__ayanami_macro_expand(input, args, line, col, file)`。
- 首个消费者：`std/panic.aya` 的 `#panic("msg")` → `panic_at(line, col, file, msg)`
  → runtime `__ayanami_panic_at` 打印 `runtime error: ... --> file:line:col` + 源码片段，退出码 101。
- 标准库越界检查：`String.index` / `ArrayList.index/set/pop` / `LinkedList.index` 调用
  `panic_bounds_at`；**函数级 track_caller** 已实现：函数末尾声明保留参数
  `int __line, int __col, String __file`，调用点自动填充（.lcl `fn=` flags 带 `caller`），
  因此 std 越界 panic 会指向用户调用行。

### 8.5 注解分类与 MIR 优化插件（A5d）

注解是大类，不用单一 `#[attr]` 表达；按类别给不同的声明、接口与阶段：

| 类别 | 标记 | 接口 | 阶段 |
|---|---|---|---|
| 编译器原语 | 保留名（`cfg`/`inline`/效应/契约/`follow_with`） | 编译器内部 | 各 pass |
| 宏注解 | `#[macro]` | 源码 → 源码 | 解析后展开 |
| 优化注解 | `#[pass]` | MIR 函数 → MIR 函数 | MIR 降级后 |
| 检查注解 | `#[check]` | MIR 函数 → 诊断（只读） | MIR 降级后 |

- **产物与跨平台**：注解库打包为 `.lcl`（LIR + `macro=`/`pass=` 表）；宿主插件由使用方编译器
  从 LIR 现场构建（llc PIC + shim + 依赖链接）并缓存；**不随包分发 `.so`**，天然跨平台。
- **MIR 公开（schema v0）**：`std/mir.aya` 定义扁平 preorder 视图
  （`MirFunction`：`pure`/`no_error`/`node_count` + `kinds`/`is_call`/`callee_pure`/
  `is_alloc`/`is_asm`/`is_store`/`child_counts` 并行 `[int]` 数组）；
  schema 带版本号，不匹配拒绝加载。
- **桥接（生成式 C，ABI v3 扩展）**：编译器把 `MirFunction` 序列化为 blob →
  shim 解码为与 `std/mir.aya` 布局一致的 C 结构体 → 调用用户 pass →
  编码回 blob → 编译器反序列化。聚合传值 ABI 不可移植，故 v0 约定为
  **`fn(ref mut MirFunction) -> void`**（就地修改，指针传参）。
- **执行后校验**：schema 版本/长度校验；只读数组（结构/分析字段）不得变化；
  body 改写只能通过编辑数组表达；失败报「注解名 + 调用点 + 原因」。
- **body 改写（schema v1 编辑面）**：`edit_kind[i]` 支持
  1/2/3/4=字面量（int/float 位模式/bool/char；比较节点按 MIR 语义放行 bool）、
  **5=克隆替换**（`edit_i64` 为源表达式节点下标，类型必须一致）、
  **6=删除语句**（替换为空块）；编译器按 preorder 下标应用编辑并跳过被替换子树，
  **应用后由管线重跑借用检查**。
- **结构化元数据**：视图含 `stmt_kinds`（语句种类）与 `var_ids`（局部变量，+1 编码），
  支持数据流类 pass；`subtree_sizes`/`replace_with`/`delete_stmt` 等 stdlib 辅助。
- **pass 不动点**：同一函数上的多个 pass 按源码顺序反复执行，直到 MIR blob 稳定
  （上限 8 轮；不收敛报 `did not converge`），使单轮实现的 pass 可组合。
- **能力边界**：允许修改效应声明；**禁止修改签名**；v1 仅函数内、仅整型表达式替换。
- **只读检查（`#[check]`）**：签名 `fn(ref MirFunction) -> void`（不可变借用）；
  通过 stdlib `warn(msg)`/`error(msg)` 上报诊断（`__ayanami_diag_emit` 通道，
  runtime.c 提供弱符号空实现、插件 shim 强定义）；error 级失败编译；
  插件返回的 blob 必须与输入完全一致（改 MIR 直接报错）。
- **解析**：与宏一致——`import` 自动作用域（该包导出的注解全部进入裸名表）、
  `pkg::name` 消歧、重名报错；声明侧校验 `fn(MirFunction) -> MirFunction`。
- **信任**：原生插件 = 编译期执行代码，信任级同宏；版本校验与沙箱后置。

## 9. 实现接口（跨包与工具）

| 层 | 载体 |
|---|---|
| AST | `Attr { name, args, span }`，挂在声明的 `attrs` 字段 |
| HIR | `HirFn.attrs`、`HirStructField.attrs` 等，供各 pass 读取 |
| 包 | `.lcl` 的 `generic_sources`（formatter 输出包含标注）+ `defs` JSON；A5b 已实现宏表（`macro=`）与依赖表（`[deps]`） |
| 工具 | `defs` JSON 带属性字段；LSP/VSCode 插件、未来的代码设计平台消费 |
| 宏 | 已实现：`AttrPath` 解析 → 宏表（`.lcl`）→ 插件 `.so`（`/tmp/ayanami-macros`，ABI v2） |

## 10. 决策记录（ADR）

| 编号 | 决策 | 理由 |
|---|---|---|
| ADR-1 | 语法用 `#[...]` | 与 Rust 一致，`inline` 等关键字可平滑迁移 |
| ADR-2 | 未知标注报错 | 保证「只有注册表内的标注有语义」，避免魔法蔓延 |
| ADR-3 | 优化承诺默认信任 | 与 Rust unsafe 一致；debug 校验留作可选后置 |
| ADR-4 | 生命周期用 `#[follow_with(...)]` 注解，不用 `'a` 类型参数 | `'a` 难读；注解声明“引用不超过被引用对象”，函数/字段照常写 |
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
| ADR-16 | 注解按类别分层（原语/宏/pass/check），不用单一 `#[attr]` 标记 | 各类接口、阶段、信任级不同，单一标记无法表达 |
| ADR-17 | MIR 插件可改效应、禁改签名，执行后重跑结构/借用/效应检查 | 保留优化自由度，同时守住内存安全与 ABI 稳定 |

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
- [x] A5b-2 M3 插件 ABI（Ayanami 优先）：宏库 LIR → llc(PIC) + C shim + runtime → `.so`；编译期 dlopen 调用；
      item 级展开、输出重新解析、深度上限 32、`/tmp/ayanami-macros` 缓存
- [x] A5b-3 宏实参：`#[pkg::macro(args...)]` → 宏形参 `(String input, String ...args) -> String`，
      实参按源码文本传入；插件 ABI v2（args 数组）+ shim 输入拷贝 + 缓存键含 ABI 版本；
      `.lcl` `[deps]` 依赖表 + 宏插件递归链接依赖对象
- [x] A5c-1 语句级宏：块内 `#[pkg::macro(args...)]` 语句 → 源码进出、可展开多条、递归展开；
      函数体与 `if`/`for`/`while` 块遍历；`#[cfg]`/`#[invariant]` 等编译器标注保留
- [ ] A5d-1 注解分类：`#[pass]` 声明 + `.lcl` `pass=` 表 + import 自动作用域/全限定解析 + 签名校验
- [x] A5d-2 MIR 优化插件：`std/mir.aya` 扁平视图 + 生成式 C 桥接（schema v0）+ 执行后校验 +
      demo `#[auto_pure]`（把可证明无副作用的函数提升为现有 `#[pure]` 承诺）；
      v0 约定 `fn(ref mut MirFunction) -> void`、body 只读（改写报错）、可改效应
- [x] A5d-3a body 改写编辑面：`edit_kind/edit_i64` 表达式整型常量替换 + preorder 下标应用 +
      应用后重跑借用检查；demo `#[const_fold]`（`1 + 2 + 3` → `6`，`2*3 + 4*5` → `26`）
- [x] A5d-3b `#[check]` 只读检查注解：`fn(ref MirFunction) -> void` + `warn`/`error` 诊断通道 +
      只读强制（blob 必须不变）；demo `check_lib`/`test_check`
- [x] A5d-3c-1 字面量编辑扩展到 float/bool/char + pass 不动点（源码顺序、8 轮上限、
      不收敛报错）；demo 常量折叠改为单轮 + 比较折叠（`br i1 1`/`br i1 0`）
- [x] A5d-3c-2 通用编辑面：克隆替换（同类型节点复用）+ 语句删除 + `stmt_kinds`/`var_ids` 元数据；
      demo `const_prop`（常量传播 + 折叠 + 不动点 → `ret i64 12`）、未用赋值删除
- [x] 顺带修复解析器：if/while 条件禁用结构体字面量（`if i > x { ... }` 曾被解析为 `x { x = i }`）
- [ ] A5d-4 更完整 MIR schema（类型/调用/字段完整往返）与跨函数 pass（远期）
- [x] A5c-2 函数宏（表达式级）：`#name(args)` + 保留参数 `__line/__col/__file` + ABI v5 +
      `#panic`（运行时 panic + 源码定位 + 退出码 101）+ 标准库越界检查
- [x] A5c-2 函数级 track_caller：保留参数 `__line/__col/__file` 调用点自动填充（含 .lcl `caller` 标记），
      std 越界/空表 panic 指向用户调用行
- [ ] A5c-2 后续：其它语言插件 / WASM 沙箱 / 插件清单与权限（远期）

### 已知问题

- **Asuka 生成解析器当前对所有真实程序解析失败（`no alt`）**，
  编译器实际依赖手写回退解析器；因此属性在生成解析器桥接（`gen_bridge`）
  中暂为透传空列表（代码中有 TODO）。修复生成解析器后需补桥接。
- 属性参数当前仅支持标识符/整数/字符串字面量；表达式参数随 A2 扩展。
- `.lcl` 包格式在 A1 变更后不向后兼容，需随编译器一起重新生成（std 已重建）。
- A2 剩余：`requires`/`ensures`/`assume`/`invariant` 与 debug/release 模式切换。
