# HIR transform schema v1（A5e/B0）

## 目标

给第三方 `#[transform]` 一个稳定、可版本化的 HIR 传输格式，使插件可以：

1. 读取完整 HIR（函数、类型、body、效应）；
2. 返回修改后的 HIR；
3. 编译器在 MIR 前应用，并重新执行后续检查。

B0 已完成：`HIR → JSON → HIR` 的 identity roundtrip。
B1 已完成 identity 插件通路：`.lcl` `transform=`、函数上的 `#[pkg::name]` 触发、
`fn(String json) -> String` 插件调用。B2 已用低层 JSON 修改跑通首个变换
（`"value":7` → `"value":8`）；结构化 `std/hir.aya` helper 仍待补。

## 为什么用 JSON

- Ayanami `std/collections/json` 已有解析/渲染，插件侧可直接使用；
- 传输层可以复用宏插件的 `String → String` ABI，不需要新 C 结构布局；
- schema 可读、可调试、可快照；
- 后续可在 JSON 上提供 `std/hir.aya` 类型 helper，而不改变底层 ABI。

## 顶层格式

```json
{
  "schema": 1,
  "items": [...],
  "vtables": [...],
  "struct_defs": [["Point", [...]]],
  "generic_struct_params": [...],
  "imported_fns": [...],
  "statics": [...],
  "consts": [...]
}
```

- `schema` 必须等于 `HIR_SCHEMA_VERSION`（当前 1）；
- `struct_defs`/`generic_struct_params` 用数组而不是 JSON object，编码时按名字排序，
  保证 roundtrip 稳定；
- symbol 一律用字符串，解码时 `Symbol::intern`；
- `FnId`/`VarId` 用整数，B1/B2 由编译器负责校验/重编号；
- 源码位置 `Span` 保留，便于 transform 后的诊断；
- attrs 以源码文本 + span 传输，解码时重新解析（避免序列化任意 AST 表达式树）。

## 节点编码

每个 HIR 表达式带 `"kind"` 标签，例如：

```json
{"kind":"bin","op":"add","lhs":{...},"rhs":{...},"ty":{"kind":"int"}}
```

语句同理：`assign` / `field_assign` / `return` / `if` / `while` /
`contract` / `block` 等。

类型：`int` / `float` / `named` / `unique` / `ref` / `fn_ptr` / `array` 等。

效应：`effects`、`pure`、`no_error`、`throws`、`inferred` 分开保存。

## B2 低层修改示例

当前插件可以直接操作 HIR JSON 字符串。例如把字面量 7 改成 8：

```ayanami
import "text"

#[transform]
pub fn bump(String input) -> String {
    old = "\"value\":7"
    new = "\"value\":8"
    return replace(ref input, ref old, ref new)
}
```

消费方：

```ayanami
import "transform_lib"

#[lib::bump]
fn main() -> int {
    return 7
}
```

这证明插件可以修改 HIR 并影响后续 MIR/LIR。v1 不提供 typed JSON helper，
因此复杂变换仍容易写错；B2 的下一小步是 `std/hir.aya` 访问器/构造器。

## Identity 调试开关

设置：

```bash
AYANAMI_HIR_SCHEMA=1 ayanami check example/test_struct.aya
```

`lower_program` 会在 HIR 降级后执行：

```text
HIR → JSON → HIR
```

并断言再次编码结果完全一致，然后继续 MIR/LIR。它用于在 `check_all` 中证明：

- schema 覆盖现有节点；
- 解码后的 HIR 仍能通过后续检查；
- 编码顺序稳定。

## 当前覆盖

schema 覆盖：

- `HirProgram` 所有字段；
- `HirItem`：Fn / StructDef / Namespace / InterfaceDef；
- `HirFn`、`HirLocal`、`HirBlock`；
- 全部 30 种 `HirNode`；
- 全部 `HirType` / `HirLiteral`；
- `EffectDecl` / `EffectSet` / `ThrowsDecl`；
- `VtableEntry` / `ImportedFnSig` / `HirStatic` / consts；
- attrs（源码文本 + span）。

## 后续

- **B1（已完成 2026-11-14）**：`.lcl` `transform=` 表、`#[transform]` 声明、
  `fn(String json) -> String` 插件 ABI、identity 插件；
- **B2（低层 MVP 已完成）**：HIR JSON 字符串修改已跑通；`std/hir.aya` 视图/helper、
  插入函数/改 body/改调用点尚未完成，
  变换后 FnId/VarId/类型/借用/效应校验；
- **B3**：泛型、enum niche、vtable、async/sync 内建变换统一接口。

## 非目标

- 本切片不改变插件执行模型（仍按可信构建期代码处理）；
- 不做远程 registry、沙箱、WASM；
- 不承诺 schema 跨大版本兼容；变化必须递增 `HIR_SCHEMA_VERSION` 并进入插件清单。
