# 闭包与捕获（M2）

> 状态：**C1 + 统一（U1）已实现**（2026-10-07）。安全代码只允许 `Fn(T) -> U`；
> 裸 `fn(T) -> U` 仅保留给 `extern "C"` 签名，unsafe 落地后再开放（M5）。
> 关联：GitHub #15（epic）、#13、#53；bd `Ayanami-language-thc` / `.2` / `.3`。

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

- FnOnce 检查、`T: Fn(...)` 约束、引用捕获、环境栈分配优化。
- unsafe 裸函数指针的完整语义（转换、回调变量、FFI 泛化）。

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
- 捕获 std 导入类型的拥有字段（如 `String`）依赖导入 struct_defs 的所有权信息；
  当前导入路径有已知缺口（bd「导入结构体字段丢失所有权」），此时捕获值沿用该行为不释放。
- 调用实参的通用移动跟踪仍有历史缺口（如 `String` 实参重复传递不报错），闭包路径已由
  `wrap_arg_for_param` 显式移动 + `checks.rs` 顺序检查覆盖。
