# 闭包与捕获（M2）

> 状态：**M2 完成**（2026-10-07）：C1 + 统一（U1）+ **FnOnce** + 健全性收尾
> （字段移出清零、extern 实参借用、`Fn` 泛型实参 mangle #151）。
> 安全代码只允许 `Fn(T) -> U` / `FnOnce(T) -> U`；裸 `fn(T) -> U` 仅保留给
> `extern "C"` 签名，unsafe 落地后再开放（M5）。
> 关联：GitHub #15（epic）、#13、#53、#151；bd `Ayanami-language-thc`（C3 后续另立）。

## 1. 统一模型

| | 安全代码 | extern "C" / 未来 unsafe |
|---|---|---|
| 类型 | `Fn(T) -> U` | `fn(T) -> U`（裸指针） |
| 能装 | 命名函数、任意 lambda | C 回调 |
| 表示 | `{ env, vtable }` 16B | 单字代码指针，Copy |
| 调用 | `code(env, args...)` | `code(args...)` |
| Copy | 静态闭包 Copy；捕获闭包移动 | Copy |

- **静态闭包**（`Closure(.., false)`）：命名函数 / 非捕获 lambda。env = 裸代码指针，
  vtable = `[noop_drop, trampoline]`（trampoline 转发 `env(args...)`）。零堆分配、Copy。
- **拥有闭包**（`Closure(.., true)`）：捕获 lambda。env = 堆环境（`__ClosureEnv_N`），
  vtable = `[drop_glue, code]`；移动语义，作用域结束递归释放。
- 静态闭包可隐式重标记为拥有型（同布局）；反向不允许。
- `ref Fn(T) -> U` 形参：借用闭包，调用点对左值自动借用，可重复传入同一闭包。
- 旧包（.lcl）源码里的 `fn(...)` 类型在导入侧统一降级为拥有型 `Fn`（向后兼容，std 无需先迁移）。

## 2. 语法与语义

```ayanami
fn apply(Fn(int) -> int f, int x) -> int { return f(x) }   // 统一形参
fn add(int a, int b) -> int { return a + b }

fn main() -> int {
    f = add                                  // 命名函数 → 静态闭包（Copy）
    y = 10
    g = (int x) -> int { return x + y }      // 捕获 y → 拥有闭包
    h = (int x) -> int { y = y + x; return y }  // 可变捕获（FnMut）
    return apply(f, 1) + apply(g, 2)         // 静态可重复传；拥有型传一次
}
```

- 捕获按值：Copy 复制 / 拥有类型移动；lambda 内可修改捕获副本（FnMut）。
- 尾表达式 lambda：`(int x) -> int { x + 1 }` 作为隐式返回值。
- 不可捕获 `ref` / `ref mut` 变量（引用不可存字段）；拥有捕获不可移出（FnOnce 留 C3）。
- `extern "C"`（含 `#[export]`）签名可写裸 `fn(...)`；调用时实参目前只支持**直接函数名**。

## 3. 非目标（C3 / M5）

- 引用捕获、环境栈分配优化（FnOnce 与 `T: Fn(...)` 泛型约束已实现）。
- unsafe 裸函数指针的完整语义（转换、回调变量、FFI 泛化）。
- 无长度元数据的裸动态 `[T]` 对拥有元素自动递归 drop；定长数组和带兄弟长度字段的集合已支持。

## 4. 实现位置

| 位置 | 内容 |
|---|---|
| `src/parser/parser/types.rs` / `decl.rs` | `Fn(...)` 解析；`fn(...)` 仅 extern/`#[export]` 签名允许（否则解析报错） |
| `src/parser/parser/stmt.rs` | `parse_lambda` 尾表达式（body 带 `Block`） |
| `src/hir/ty.rs` | `HirType::Closure(ps, ret, owns_env)`；静态 Copy / 拥有 drop |
| `src/hir/lower/helpers/closure.rs` | 捕获分析（AST 自由变量 + 声明追踪，含嵌套） |
| `src/hir/lower/helpers/closure_lower.rs` | 拥有闭包：env 结构体、代码函数、drop glue、vtable |
| `src/hir/lower/helpers/closure_static.rs` | 静态闭包：每签名 trampoline + noop drop + vtable |
| `src/hir/lower/helpers/types.rs` | `ast_type_to_hir`（FnPtr→Fn）/ `ast_type_to_hir_extern`（保留裸）/ 旧包统一 |
| `src/hir/lower/body/expr_misc.rs` / `expr_lower.rs` | lambda 降级、函数名作值 → 静态闭包 |
| `src/hir/lower/body/expr_call.rs` | `Fn` 调用（`SVCall`）、`ref Fn` 解引用调用、extern 裸实参 |
| `src/hir/lower/body/overload_resolve.rs` | 静态→拥有 / 静态→裸 FnPtr 兼容 |
| `src/parser/parser/mod.rs` + `hir/lower/body/iface_args.rs` | `T: Fn(...)` 签名约束编码、特化校验与 Fn→FnOnce 子类型 |
| `src/mir/lower/checks.rs` | 语句内按求值顺序的 use-after-move 检查（修复重复移动双释放） |
| `src/lir/ir/helpers.rs` / `emit/functions.rs` | Closure drop（vtable slot 0）；生成函数 internal 链接 |
| `src/lir/serialize/*` | `.lcl` 类型 tag 16（含 owns 标志） |

## 5. 验收与用例

- 正例：`example/test_closure_capture.aya`（捕获/可变/拥有 + live_allocs=0）、
  `example/test_closure_hof.aya`（Fn 形参/泛型推断/返回闭包/结构体字段）。
- 负例：`tests/compile_fail/{capture_ref, closure_to_fn, move_capture_out, fn_type_rejected}.aya`。
- `./scripts/check_all.sh` 全绿；debug 运行 live_allocs = 0（本地拥有类型无泄漏）。

## 6. 已知限制

- extern C 回调实参只支持直接函数名（裸指针变量留待 unsafe）；extern 形参之外的 `fn` 类型报错。
- 拥有闭包按值传给 `Fn` 形参会移动（重复使用需 `ref Fn` 形参）；静态闭包不受影响。
- 拥有捕获不可移出闭包（FnOnce 未检查）；`ref` 变量不可捕获。
- **FnOnce**：体内移出捕获（`x = a` / `return a` / 按值传参 / 显式 `move`）→ 闭包类型
  `FnOnce(...)`；调用消费闭包（`use of moved value` 防止二次调用），环境由调用方临时变量在
  作用域结束释放；移出字段在源结构/环境中清零（`SLirFieldTake`），drop 不再双重释放。
  `Fn` 值可隐式传给 `FnOnce` 形参；`FnOnce` 不能传给 `Fn`；`ref FnOnce` 不可调用。
- 定长数组按静态长度、带 `len`/`cap` 兄弟字段的动态集合按计数递归释放拥有元素；捕获闭包存入
  `ArrayList`/`HashMap` 后 env 只 drop 一次（`tests/runtime_safety/closure_in_collection.aya`）。
  无长度元数据的裸 `[T]` 仍只能释放缓冲区，无法枚举元素。
- `extern "C"` 回调实参目前只支持直接函数名；静态闭包变量/裸指针变量留待 unsafe。
- `extern "C"` 声明（无体）的实参按**借用**传递（不移动/清零）——C 侧不消费所有权。
- 捕获 std 导入类型的拥有字段（如 `String`）依赖导入 struct_defs 的所有权信息；
  导入路径已补回 `[T]` → `unique [T]` 所有权（`collect_import.rs`）。
- 调用实参的通用移动跟踪：拥有堆数据的命名类型（`String`/含堆结构体/枚举）按值传参
  标记移动（`expr_call.rs` + `type_needs_drop`）；`extern "C"` 按借用、POD 结构体按复制。
  `checks.rs` 顺序检查覆盖直线重复传参；循环回边的重复移动由运行时源变量清零兜底
  （`SLirLocalTake`，避免双重释放）。
- 借用临时量（`f(ref "x")` / 方法接收者字面量 / 调用结果按引用传参）提升到临时局部变量，
  语句结束后 drop（赋值/表达式/返回/字段·下标·解引用赋值/`if` 首条件）。
  残余：`while`/`elif` 条件内的借用临时量仍每次求值泄漏一次分配（bd `Ayanami-language-*`）。
- match / if 表达式非 Copy 结果：结果变量仅在分支内条件赋值（`is_result`，不参与覆盖/块末
  drop），链后显式移出到最终临时量，避免条件初始化被 drop 或重复释放。

## 7. C3a：`T: Fn(...)` 泛型闭包约束

泛型参数的约束表允许闭包签名，而不是仅允许接口名：

```ayanami
fn apply[T: Fn(int) -> int](T f, int x) -> int {
    return f(x)
}
```

- 约束文本保留 `Fn`/`FnOnce` 的完整参数与返回类型；泛型体中的 `f(args)` 可直接调用。
- 特化时把推断出的具体闭包类型与约束逐项匹配；`owns_env` 不参与签名匹配。
- `Fn` 可满足 `FnOnce`，`FnOnce` 不可满足 `Fn`。
- 非闭包类型、参数数量/类型或返回类型不匹配时，给出“不满足闭包约束”的 HIR 诊断。
- 该能力作用于泛型参数；普通 `Fn(...)` 形参语义不变。
