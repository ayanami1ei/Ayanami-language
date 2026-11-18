# Phase 6：宏系统升级（文本 → AST/HIR）设计

> 状态：**mcl.1 / mcl.2 / mcl.3 完成**（TokenStream 编解码（含分组）+ std TokenStream/Span/Token API + token 宏（item/语句/表达式）+ opt-in hygiene + 调用点/显式 span 诊断 + `dump expanded` / `dump macros` / token 映射链 / 错误包装）（2026-11-18）。关联：bd `Ayanami-language-mcl`（epic）与
> `mcl.1`（TokenStream/Span/Diagnostic）/ `mcl.2`（hygiene 与错误定位）/ `mcl.3`（展开调试与 IDE）；
> GitHub #43 / #42 / #45 / #62。现状（A5 已落地部分）见 `docs/annotations.md` §A5。

## 1. 现状（2026-10）

| 能力 | 位置 | 形态 |
|---|---|---|
| 文本宏 `#[macro] fn f(String input, String ...args) -> String` | `compiler/macro_expand/plugin.rs`（ABI v5） | item / 语句 / 表达式级；**源码文本**进出；插件 `.so`（宏库 LIR → llc(PIC) + C shim + runtime） |
| MIR pass `#[pass]` | `pass_plugin.rs`（schema v0/v3） | MIR blob 进出（`fn(ref mut MirFunction) -> void`）；API 见 `std/mir.aya` |
| 只读检查 `#[check]` | `check_plugin.rs` | `warn`/`error` 诊断通道；只读强制（blob 不变） |
| 调用点定位 | `__line/__col/__file` 保留参数 | 调用点自动填充（含 `.lcl` `caller` 标记）；std 越界 panic 指向用户行 |

限制（升级动机）：

- **文本进出**：宏内无法结构化分析（缺 TokenStream）、转义/拼接易错、无法安全复用编译器解析能力；
- **无 Span/Diagnostic**：展开后的错误指向宏输出而非调用点；宏无法主动报位置化诊断；
- **无 hygiene**：宏内定义的名字会泄漏到调用点、也可能捕获调用点的名字（静默错绑定）；
- 插件 `.so` 较重（简单宏也走 native 编译；`/tmp/ayanami-macros` 缓存缓解）。

## 2. 目标

1. **结构化接口**：`TokenStream`（token 序列 + Span）替代字符串进出；
2. **诊断**：宏可产生带调用点定位的 error/warn（Diagnostic 通道）；
3. **hygiene（务实版）**：宏内定义的名字不泄漏、不捕获调用点名字；
4. **兼容**：文本宏保留（迁移期）；结构化宏为新 kind（ABI v6）；
5. **IDE 友好**：展开记录（span 映射）可导出（mcl.3）。

## 3. 设计

### 3.1 TokenStream 表示（ABI v6）

- 编码（小端）：`[kind:u8][file:u32][line:u32][col:u32][len:u32][bytes...]` 序列；
  **分组**：`(...)`/`[...]`/`{...}` 折叠为 GROUP token，负载 = 开符码 + 递归编码的子流；
- kind：标识符 / 字面量（int/float/char/bool/string）/ 标点（含多字符）/ 关键字 /
  **分组**（`(...)` `[...]` `{...}`，递归承载子流）；
- 输入：调用点 item / 语句 / 表达式的 token 序列（**含调用点 span**）；
- 输出：同构 token 序列（编译器重新解析为 AST 后按既有流程降级）；
- 编译器侧复用 `lexer`（token 化）与 `parser`（重解析）；宏侧 API 契约见 §4。

### 3.2 Span 与 hygiene（务实模型；mcl.2 已实现原语）

- **实现（opt-in 原语）**：插件 shim 提供 `__ayanami_macro_hygienic(name) -> name__m<id>`
  （宏 id = 插件符号短哈希，确定性）；宏内**定义名**显式经此加后缀 →
  不捕获调用点同名函数、不泄漏给调用点；调用点名字（输入 token/实参）保持原样；
- 宏库可直接 `extern "C" fn __ayanami_macro_hygienic(String) -> String;` 声明使用
  （std 侧按 §4 契约包装为 `hygienic_ident` 等 API）；编译器对 `__ayanami_macro_*`
  声明发射 `extern_weak`（宏库作为模块编入可执行文件时允许未定义——调用只存在于插件路径）；
- `Span::new` / `Token::span` / `TokenStream::push_*_at` 允许宏作者给新 token 显式归属位置；
  `dump macros` 同时输出 input/output token spans 与 LCS 映射供 IDE 使用。
- 不追求 Rust 级完整 hygiene；采用显式 hygiene 原语，不做隐式自动改名。

### 3.3 Diagnostic 通道（mcl.2 已实现基础）

- **实现**：shim 提供 `__ayanami_macro_diag(level, msg)`（1=warn，2=error；收集上限 32）；
  编译器在插件调用后读取（`__ayanami_macro_diag_count/level/msg`）：
  warn → stderr `warning: <msg> (in macro `<symbol>`)`；error → 编译失败并附宏名；
- 宏库可直接 `extern "C" fn __ayanami_macro_diag(int, String) -> void;` 声明使用；
- 生成代码错误包装：宏输出解析失败 → `in macro `<name>` expansion: … (at 调用点)`；
- 位置：`__ayanami_macro_diag` 产生的 warn/error 附带调用点（`at l:c`，编译器在调用时注入）；
- `diag_warn_at` / `diag_error_at(msg, Span)` 可把诊断定位到宏内显式 line/col；未指定时
  回退宏调用点。

### 3.4 与现有 A5 的关系

- 宏分类（实现取用显式标记）：`#[macro]` = 文本宏（现有）；`#[macro(tokens)]` = token 宏
  （ABI v6，输入/输出为 TokenStream 编码字节，经 `String`（AyaBuf）承载——二进制安全）；
  同一包可混合；`.lcl` 宏表写 `macro="name,tokens"`（旧格式缺省为文本宏，向后兼容）；
- `.lcl` 宏表新增 `kind=`（text/tokens）字段；旧包缺省为 text（向后兼容读取）；
- MIR pass / check 不变（A5d）；文本宏不废弃（迁移期保留，后续按需引导）。

## 4. std 侧 API 契约（供 std 子仓实现）

```ayanami
// std/macros.aya（示意；契约稳定后由 std 实现，编译器提供 shim/ABI）
pub struct Span { ... }                  // file/line/col（与诊断一致）
pub struct TokenStream { ... }           // 不透明；编译器 ABI 编解码
pub struct Token { ... }                 // kind + span + 文本

pub namespace TokenStream {
    pub fn iter(ref self) -> TokenIter     // 含分组递归展开
    pub fn to_source(ref self) -> String   // 调试/文本宏桥接
    pub fn from_source(String) -> TokenStream
    pub fn push_ident(Symbol/String)
    pub fn push_punct(char)
}

pub fn diag_error(String msg, Span span)
pub fn diag_warn(String msg, Span span)
```

## 5. 阶段与验收

| 阶段 | 内容 | 验收 |
|---|---|---|
| **mcl.1**（完成） | ABI v6/v7：TokenStream 编解码（分组 token 已支持）+ 编译器侧桥（字节通道）+ 宏分类（`#[macro(tokens)]`；item/语句/表达式三形态）+ std `Span`/`Token`/`TokenStream` 字节 API | 正例：`example/test_macro_tokens.aya`（`tok_keep` 往返 / `tok_twice` 拼接 / `#tok_expr` 表达式）+ 编译器/std 编解码单测；回归全绿 |
| **mcl.2**（完成） | 显式 hygiene 原语 + token span 构造/读取 + Diagnostic 通道（调用点及显式 span） | 正例：`test_macro_hygiene`、`macros_test::test_token_span_attribution`；负例：`macro_diag_error`、`macro_diag_span`；采用设计约定的 opt-in hygiene，不做隐式自动改名 |
| **mcl.3**（完成） | `ayanami dump expanded` + `ayanami dump macros` 结构化 JSON + `AYANAMI_MACRO_TRACE=1` | 记录覆盖 item/statement/expression、调用位置、token mode、input/output token spans，以及 LCS output→input 映射；新生成 token 以调用点定位兜底 |
| 远期 | 其它语言插件 / WASM 沙箱 / 插件清单与权限 | — |

## 6. 非目标

- Rust 级完整 hygiene、声明宏 `macro_rules!` 式语法糖；
- 插件沙箱与权限模型（远期，见 A5c-2 后续）；
- 递归上限调整（保持 32）与文本宏废弃。
