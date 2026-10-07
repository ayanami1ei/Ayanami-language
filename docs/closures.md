# 闭包与捕获（M2）

> 状态：**C1 已实现**（2026-10-07）；C2（fn→Fn trampoline、std HOF 迁移）待做。
> 关联：GitHub #15（epic）、#13（对象表示与调用约定）、#53（Fn/FnMut/FnOnce 模型）；
> bd `Ayanami-language-thc` / `.2` / `.3`。

## 1. 目标

把 lambda 从"裸函数指针（无捕获）"升级为**可捕获环境的闭包**：

- 新增闭包类型 `Fn(T) -> U`（拥有语义），与现有 `fn(T) -> U` 裸指针共存。
- 按值捕获（Copy 复制；拥有类型移动进环境），支持**可变捕获**（FnMut 语义：lambda 内可修改捕获副本）。
- 复用接口/胖指针机制（`{data, vtable}`、vtable、虚调用、drop 槽），不新增调用 ABI 家族。
- 非捕获 lambda 保持现有 `fn` 裸指针路径（零开销、Copy、可 FFI），现有代码零改动。

## 2. 非目标（本阶段不做，C3 评估）

- FnOnce 检查（把捕获值移出闭包）；`FnMut`/`FnOnce` 作为独立约束名。
- 按引用捕获（语言当前不允许引用存字段，天然受限）。
- `T: Fn(...)` 形式的泛型约束、闭包 trait 对象互操作、async。
- 环境栈分配优化（逃逸分析）；首版统一堆分配。

## 3. 语法与类型

```ayanami
// 闭包类型（注意大写 F；fn 仍是裸函数指针）
fn apply(Fn(int) -> int f, int x) -> int { return f(x) }

// lambda（既有语法；返回类型可省略；支持尾表达式）
y = 10
f = (int x) -> int { x + y }        // 捕获 y（Copy）
g = (int x) -> int { y = y + 1; return x + y }  // 可变捕获（修改闭包自身状态）
```

- 解析：`Fn` 为普通标识符，仅在类型位置后跟 `(` 时按闭包类型解析（无需新关键字）。
- 尾表达式：lambda body 的裸尾表达式作为隐式返回值（与普通函数一致），修复当前"尾表达式按语句丢弃"的缺口。

## 4. 语义

### 4.1 捕获集合

lambda body 中**引用且在外层作用域可见**、且未在 lambda 内先声明的变量（递归含嵌套 lambda 的间接引用）。
判定在 AST 层按语句顺序做（声明追踪），捕获记录 `(名字, 类型)`。

- `x = v` 在 lambda 内且外层有 `x` → **捕获 + 可变写**（FnMut）；外层没有 `x` → lambda 局部声明。
- 拥有类型捕获 = 移动；捕获后外层再用报 use-after-move（与普通移动一致）。
- 不可捕获 `ref` / `ref mut` 类型（引用不可存字段）：报错，建议解引用/`.clone()`。
- 拥有捕获不可移出闭包（`return s`）：报错（FnOnce 语义留待 C3）。
- 嵌套 lambda：内层捕获外层 env 中的字段或外层局部；捕获分析递归。

### 4.2 类型规则

- 非捕获 lambda：类型 `fn(...)`（现状不变）。
- 捕获 lambda：类型 `Fn(...)`，**非 Copy**，离开作用域释放环境（递归 drop 捕获字段）。
- `fn` → `Fn`：参数位置允许隐式转换（trampoline，见 §6）。
- `Fn` → `fn`：不允许（可能捕获）；报错建议形参改 `Fn`。
- 捕获 lambda 传给 `fn` 形参：报错（同上）。

## 5. 表示与 ABI（复用接口机制）

- `HirType::Closure(Vec<HirType>, Box<HirType>)`；LLVM 类型 `{ ptr, ptr }` = `{ env, vtable }`，
  与 `FatPtr` 布局一致（虚调用 `extractvalue 0/1` 直接可用）。
- **env**：编译器生成结构体 `__ClosureEnv_N`，字段 = 捕获变量（按捕获顺序），注册 `struct_defs`。
- **代码函数**：`__lambda_N(env: ref mut __ClosureEnv_N, 参数...) -> 返回`，env 为第一参数（ABI = `ptr`）。
  捕获读写落在 env 字段上（字段访问/字段赋值，自动穿透 `ref mut`）。
- **vtable**：`[drop_glue_or_null, call_fn]`，与接口 vtable 相同布局（slot 0 = drop 预留槽；
  调用槽 = `1 + method_index`，method_index=0）。
- **创建**：构造 env 结构体值（捕获表达式按移动/复制）→ 复用 `SMFP`（MakeFatPtr：malloc + 拷贝 + 写 vtable）。
- **调用**：复用 `SVCall`（`method_index = 0`，receiver = 闭包值，interface = 合成符号）。
- **释放**：`emit_drop_value` 新增 `Closure` 分支：取 vtable slot 0；非空调用 drop glue（递归字段 drop + free），
  空则回退 `__ayanami_unique_free`（无拥有捕获时零额外函数）。
- **drop glue**：捕获含拥有字段时生成普通 Ayanami 函数
  `fn __closure_drop_N(unique __ClosureEnv_N env) -> void {}`，利用既有"拥有参数作用域结束递归 drop"机制；
  vtable slot 0 指向它。
- release 模式的 `unique_alloc/free → malloc/free` 字符串替换同样覆盖闭包路径（堆提升）。

## 6. `fn` → `Fn` 转换（trampoline）

参数位置出现 `Fn(...)` 而实参是 `fn(...)`（命名函数 / 非捕获 lambda / fn 变量）时：

- 生成/缓存 `__fnptr_tramp_<sig>(env: ref mut __FnTramp_<sig>, 参数...) -> 返回 { return env.f(args...) }`；
- `__FnTramp_<sig>` = `{ f: fn(...) }`，创建时 boxed 进 env（无拥有字段，slot 0 空 → unique_free）；
- 命名函数、fn 变量统一走此路径；后续可优化为直接 thunk（无分配）。

## 7. 阶段

| 阶段 | 内容 | 状态 |
|---|---|---|
| C1 | 类型/语法/表示/创建/调用/捕获（含可变）/drop/尾表达式 | **已实现** |
| C2 | fn→Fn trampoline、std HOF 迁移（std 子仓，用户） | 待做 |
| C3 | FnOnce/引用捕获/`T: Fn` 约束/环境栈分配 | 评估 |

### C1 已实现

- `Fn(T1, T2) -> U` 类型（parser `Fn(` 特判、HirType::Closure、显示/序列化/LLVM `{ptr,ptr}`）。
- lambda 捕获分析（AST 自由变量 + 声明追踪，含嵌套 lambda）、按值捕获（Copy 复制/拥有类型移动）。
- 可变捕获（FnMut：lambda 内对捕获赋值写 env 字段）。
- 环境结构体 `__ClosureEnv_N`、代码函数 `__lambda_N(env, ...)`、drop glue `__closure_drop_N(unique Env)`、
  每 lambda vtable `[drop, call]`（复用接口胖指针/虚调用机制）。
- 闭包调用（`f(x)` / `(expr)(x)`）、作参数/返回值/结构体字段、泛型推断（`Fn(T)->U` 形参 + lambda 实参）。
- 非捕获 lambda 仍为 `fn` 裸指针；闭包函数 `internal` 链接（跨模块不冲突）；`.lcl` 签名 `Fn(...)->R` 往返。
- 尾表达式 lambda（`(int x) -> int { x + 1 }`）。
- 正例：`example/test_closure_capture.aya`、`test_closure_hof.aya`；
  负例：`tests/compile_fail/{capture_ref,closure_to_fn,move_capture_out}.aya`。

### C1 已知限制

- `fn` 值（命名函数/非捕获 lambda/fn 变量）暂不能传给 `Fn` 形参（错误信息明确；C2 用 trampoline 支持）。
- 捕获 lambda 不能传给 `fn` 形参（可能捕获，不隐式转换）。
- 拥有捕获不能移出闭包（`move s` / 返回捕获）——FnOnce 检查留待 C3。
- `ref`/`ref mut` 变量不可捕获（引用不可存字段）。
- 捕获 std 导入类型的拥有字段（如 `String`）依赖导入 struct_defs 的所有权信息；
  当前导入路径有已知缺口（bd「导入结构体字段丢失所有权」），此时捕获值沿用该行为不释放。

## 8. 实现位置

| 位置 | 内容 |
|---|---|
| `src/parser/ast/ty.rs` | `Type::Closure` |
| `src/parser/parser/types.rs` | `Fn(...)` 解析 |
| `src/parser/parser/stmt.rs` | `parse_lambda` 尾表达式（body 改 `Block`） |
| `src/hir/ty.rs` | `HirType::Closure`（非 Copy、needs_drop） |
| `src/hir/lower/helpers/types.rs` | `ast_type_to_hir` / display |
| `src/hir/lower/helpers/closure.rs`（新） | 捕获分析、env/vtable 构建、drop glue、trampoline |
| `src/hir/lower/body/expr_misc.rs` | `lower_lambda` 重写 |
| `src/hir/lower/body/expr_lower.rs` / 赋值路径 | 捕获变量读写（env 字段） |
| `src/hir/lower/body/expr_call.rs` | `Fn` 值调用 → `SVCall` |
| `src/hir/lower/to_mir/*` | 复用 `SMirMakeFatPtr` / `SMirVirtualCall` |
| `src/lir/emit/types.rs` / `ir/helpers.rs` | `{ptr,ptr}` 类型、Closure drop 分支 |
| `src/lir/serialize/*` | `.lcl` 类型序列化新 tag |
| `example/test_closure_*.aya`、`tests/compile_fail/*` | 验收与负例 |

## 9. 验收标准

- 正例：Copy 捕获、拥有类型捕获（String drop 无泄漏）、可变捕获（计数器）、闭包作参数/返回值、
  命名函数 → `Fn` 形参、尾表达式 lambda、嵌套 lambda（若 C1 落地）。
- 负例：捕获 `ref` 变量、`Fn` → `fn` 形参、拥有捕获移出闭包、捕获 lambda → `fn` 形参。
- `./scripts/check_all.sh` 全绿；debug 运行 live_allocs = 0（无泄漏）。
