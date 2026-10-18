# Phase 6：宏系统升级（文本 → AST/HIR）设计

> 状态：**设计（未实现）**（2026-10-18）。关联：bd `Ayanami-language-mcl`（epic）与
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
- kind：标识符 / 字面量（int/float/char/bool/string）/ 标点（含多字符）/ 关键字 /
  **分组**（`(...)` `[...]` `{...}`，递归承载子流）；
- 输入：调用点 item / 语句 / 表达式的 token 序列（**含调用点 span**）；
- 输出：同构 token 序列（编译器重新解析为 AST 后按既有流程降级）；
- 编译器侧复用 `lexer`（token 化）与 `parser`（重解析）；宏侧 API 契约见 §4。

### 3.2 Span 与 hygiene（务实模型）

- 宏输出的 token 带两类 span 来源：
  - **调用点 span**：透传的输入 token（含实参）保持原 span；
  - **宏定义 span**：宏内新写的 token 标记宏定义位置；
- 名字解析规则：宏内**新定义**的名字按「宏定义归属」加后缀（`name#<macro-id>`，仅对定义
  生效），→ 不捕获调用点名字、不泄漏给调用点；
- 调用点名字（输入 token）保持原 span → 宏不 shadow 用户名字；
- 不追求 Rust 级完整 hygiene；边界（如跨宏递归的定义共享）在文档明示，不做隐式魔法。

### 3.3 Diagnostic 通道

- 宏侧接口（契约）：`diag_error(String msg, Span span)` / `diag_warn(...)`；
- 编译器把 Diagnostic 映射到源码位置：span 为空 → 指向调用点；否则指向 span 所指位置
  （若指向宏内部 → 标注「在宏展开中」并附调用点）；
- 与 `#[check]` 现有 warn/error 通道统一（同一渲染路径，含位置化格式）。

### 3.4 与现有 A5 的关系

- 宏分类：`#[macro]` 按签名自动分类——`String -> String`（文本，现有）/ `TokenStream ->
  TokenStream`（结构化，v6）；同一包可混合；
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
| **mcl.1** | ABI v6：TokenStream 编解码 + 编译器侧桥 + 宏分类（text/tokens） | 正例：结构化宏改写 item（如按 token 改名/加字段）；负例：非法编码位置化报错；回归全绿（文本宏不回退） |
| **mcl.2** | hygiene 务实模型（span 归属 + 定义后缀）+ Diagnostic 通道 | 正例：宏内定义名不捕获调用点、错误指向调用点；负例：冲突/重复定义诊断 |
| **mcl.3** | 展开记录导出（`--dump-macros` 或 IDE 视图） | 展开前后 span 映射可查（含递归展开链） |
| 远期 | 其它语言插件 / WASM 沙箱 / 插件清单与权限 | — |

## 6. 非目标

- Rust 级完整 hygiene、声明宏 `macro_rules!` 式语法糖；
- 插件沙箱与权限模型（远期，见 A5c-2 后续）；
- 递归上限调整（保持 32）与文本宏废弃。
