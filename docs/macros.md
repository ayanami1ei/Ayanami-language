# 自举宏引擎设计（selfhost）

> 状态（2026-10-24）：**M1–M4 已实现**（源码级解释执行，见 `selfhost/src/macro/expand.aya`）。
> 实现方式与下文的「解释执行」一致：不建 .so；注解/宏调用先展开源码再重新解析。
> M5：`#[check]` 只读插件已完成（自举用 AST 聚合视图 + PIC `.so`，见 `docs/check-plugins.md`）；
> 剩余 `#[pass]` 写回（需要 MIR 或 HIR 编辑面）。
> stage-0 参考实现：`rust/src/compiler/macro_expand/`（M3 插件 ABI v3，编译宏库为 `.so` 后 dlopen）。

## 1. 目标

让自举编译器支持 `example/macro_lib.aya` 定义的宏（11 个用例）：

| 形态 | 示例 | 语义 |
|---|---|---|
| item 宏 | `#[answer] fn placeholder()` | 宏返回源码替换整个 item |
| item 宏（库限定） | `#[macro_lib::emit_const(meaning, 42)]` | 第一个参数是 item 源码，其后是实参源码文本 |
| 语句宏 | `#[twice] y = y + 1` | 返回源码替换该语句（可多条） |
| 表达式宏 | `x = #double(3)` | 返回源码作为表达式 |
| 块实参 | `#block_len({ y = 12345 })` | 实参为块源码文本 |
| token 宏 | `#[macro(tokens)]` / `#tok_expr(...)` | 输入/输出是 token 流编码（String 承载） |
| 诊断/卫生 | `__ayanami_macro_diag` / `__ayanami_macro_hygienic` | 编译期告警、生成名加宏 id 后缀 |

## 2. 总体方案（自举：解释执行，不建 .so）

自举编译器不 dlopen，改为**编译期解释执行宏函数**：

1. **收集宏表**：扫描合并后的 AST，`#[macro]`/`#[macro(tokens)]` 标注的 FnDecl →
   `(名字, tokens?, 形参表, 函数体节点)`。
2. **展开时机**：在 `merge_source_map` 之后、`check_program` 之前，对**每个源文件**分别展开
   （保持 SrcMap 行区间稳定；新生成的代码计入该文件的行数，或接受位置漂移）。
   展开是递归的（深度上限 32）。
3. **解释器**（新增 `selfhost/src/macro/eval.aya`）：
   - 值域：`String`（源码文本）/ `int`；
   - 表达式：字面量、形参、`+` 拼接、`(expr)`、`x.len()`、`int.to_string()`、
     extern shim 调用（`__ayanami_macro_hygienic` → 生成名；`__ayanami_macro_diag` → 诊断）；
   - 语句：`x = expr`、`return expr`（后续可加 `if`/`while`）；
   - 输入约定：`input` = 被标注 item/语句的源码文本；其后依次是实参源码文本；
     表达式宏 `#f(a, b)` → `(input="", a 源码, b 源码)`；
     `__line`/`__col`/`__file` 由调用点注入。
4. **拼接**：把返回的源码文本用**现有解析器**解析成 item/语句/表达式，替换原节点；
   解析失败即宏生成非法代码（错误带宏名与调用点）。
5. **token 编码**：与 stage-0 `token_stream.rs` 对齐（长度前缀的 token 列表），
   使 `input + input` 这类拼接在 token 宏下依然合法。

## 3. 分期

- [x] **M1（item/语句/表达式 + 字符串拼接）**：test_macro、test_macro_args、test_macro_stmt、
  test_macro_fn、test_annotation_scope
- [x] **M2（块实参 + 长度/行号）**：test_macro_block、`my_line`（`.len()`/`to_string` 已支持）
- [x] **M3（token 流）**：test_macro_tokens（源文本语义对这批宏等价：仅拼接/恒等）
- [x] **M4（卫生/诊断）**：test_macro_hygiene（`__ayanami_macro_hygienic/_diag` shim）
- [x] **M5（MIR 插件）`#[check]`**：test_check 已对齐（自举原生 MirFunction 视图，见 `docs/check-plugins.md`）；
      `#[pass]` 写回仍待做

## 4. 验收

- 上述用例与 stage-0 输出/退出码一致（`./scripts/selfhost.sh diff`）；
- 固定点保持（`./scripts/selfhost.sh selfcompile`），每次改动跑 `check_all.sh`；
- 宏展开不在 `src/lir`/`src/hir` 之外引入新的 String 热路径（自举发射器无堆释放，注意内存上限）。

## 5. 已知风险

- 自举解析器/字符串处理对新增代码较敏感（历史坑 38/41/46），展开代码应尽量集中在
  `selfhost/src/macro/`，避免改动 `emit.aya`；
- 行号/SrcMap：展开后生成代码的位置诊断会偏移，诊断 golden 可能变化；
- 递归宏深度与内存上限（`selfhost.sh` 当前 4GB）。
