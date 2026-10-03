# 泛型结构体/枚举单态化（设计）

> 状态：**设计完成，未实现**（2026-10）。关联：`docs/annotations.md` §6.4（U2）、`docs/knowledge-graph.md`。
> 背景：`?` 已实现真错误传播（A3d），但对 `Result[T,E]` 等泛型枚举会报
> “payload monomorphization is not implemented yet”。

## 1. 现状与问题

- 泛型**函数**已单态化（`hir/lower/body/part_06.rs`）。
- 泛型**结构体/枚举**只有「基名」定义：
  - `struct_defs["Result"] = [ _tag: Int, _data_Ok: Named("Result_Ok"), _data_Err: Named("Result_Err") ]`
  - `struct_defs["Result_Ok"] = [ _0: Named(T) ]`（泛型参数未替换）
- 具体的实例化名 `Named("Result<int,int>")` 只在类型名上出现，`llvm_type` 走基名回退得到 `%struct.Result`；
  变体结构体字段仍是 `Named(T)` → `ptr`。
- 后果：
  1. 泛型枚举**构造**会把具体载荷（如 i64）存进 `ptr` 字段 → llc 类型错误；
  2. 泛型枚举**载荷访问**（含 `?` 取 `_data_Ok._0`）类型不一致；
  3. 具体枚举（用户自定 `enum Res { Ok(int), Err(int) }`）不受影响。

## 2. 目标

- 出现 `Named("X<args...>")` 时，按需生成**实例化结构体定义**：
  - `struct_defs["X<args>"]`：字段类型用 `{泛型参数 → args}` 替换；
  - 字段中指向变体结构体的 `Named("X_Variant")` 改写为 `Named("X_Variant<args>")`；
  - 变体结构体同样实例化（其泛型参数即枚举的参数），递归处理嵌套泛型。
- 构造与访问使用实例化名，从而让 `?`/字段访问/模式匹配在 `Result[T,E]` 上工作。
- `.lcl`：实例化条目随 `struct_defs` 序列化，跨包类型一致。

## 3. 方案

1. **实例化请求登记**
   - `Ctx` 增加 `instantiated: HashMap<Symbol, ()>`（或直接在 `struct_defs` 查键存在性）。
   - 触发点（按可达性，先做覆盖签名与构造的保守集合）：
     a. `lower_fn`：参数/返回类型；
     b. `find_field_index/type` 的回退分支（改为惰性实例化：`find_field_*` 改为 `&mut self` 或提供
        预热函数 `ensure_instantiated(&mut self, ty)` 在访问前调用）；
     c. `expr_enum::lower_enum_construct`：构造点。
2. **实例化算法**（`Ctx::instantiate_named(ty) -> Result<()>`）
   - 解析 `"Base<T1,T2>"`（`<`/`[` 两种写法兼容）；
   - `subst = { gp_name → sig_str_to_hir(arg) }`（复用 `build_generic_subst`）；
   - 对 `struct_defs[Base]` 每个字段：`substitute_hir_type`，并把 `Named("Base_Variant")`
     改写为 `Named("Base_Variant<args>")`（解析变体名 = 基名 + `_` + 变体）；
   - 对每个变体结构体：以同一 `subst` 实例化其字段，插入 `struct_defs["Base_Variant<args>"]`；
   - 递归实例化字段/参数/返回中出现的其它 `Named("...<...>")`；
   - 幂等：以完整名为键，已存在则跳过。
3. **构造与访问的实例化名选择**
   - 构造：`lower_enum_construct` 目前用 AST 写的枚举名（常为基名）。两种途径：
     a. **期望类型上下文**：`lower_return`、函数实参（`wrap_arg_for_param` 已接收参数类型）、
        带类型标注的赋值：在已知期望为 `Named("Base<args>")` 时，把构造出的 `SStruct`
        重写为实例化名（新增 `SStruct::instantiated(&self, name, ty)` 递归重建）；
     b. 无上下文：保留基名并给出类型标注建议（保守报错），避免静默错型。
   - 访问：字段查找已做替换（`variant_payload_type`），实例化定义存在后类型自洽。
4. **发射/序列化**：`emit_struct_defs`、`lir/serialize` 已按 `struct_defs` 键遍历，无需改动；
   `llvm_type` 的基名回退保留（对未实例化的基名仍保守为 `ptr`）。

## 4. 验收标准

- 新增 `example/test_try_generic.aya`（或并入 `test_try.aya`）：
  ```ayanami
  import "../std/std.aya"        // Result[T,E]
  fn inner(int x) -> Result[int, int] { if x < 0 { return Result::Err(x) } return Result::Ok(x + 1) }
  fn outer(int x) -> Result[int, int] { v = inner(x)?; return Result::Ok(v * 10) }
  fn main() -> int { ... 用 match/字段观测 Ok 与 Err 传播 ... }   // exit 0
  ```
- 泛型结构体字段访问（如 `Pair[int]`）类型正确、无 llc 错误。
- 全量 example check/run 回归、std 重建、`cargo test`/`check_all` 通过。
- `?` 的示例不再报 “payload monomorphization”。

## 5. 影响面与风险

- 涉及：`hir/lower/ctx.rs`（实例化）、`hir/lower/helpers/types.rs`（替换）、
  `hir/lower/body/expr_enum.rs`（构造）、`hir/lower/body/part_07.rs`/`part_06.rs`（触发点）、
  `hir/node.rs`（`SStruct` 重建辅助）。
- 风险：期望类型缺失时的实例化选择（保守报错）；同一基名不同实例化的 LLVM 类型隔离
  （以完整名为键即可）；泛型方法自引用（`Result[T,E]` 泛型 impl）需先实例化再看方法体。
