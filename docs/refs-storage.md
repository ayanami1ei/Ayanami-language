# M4：引用存储 / arena / stable object（设计）

> 状态：**设计完成（hfj.1）；阶段 1（`Ptr[T]`）已落地；hfj.4 评估完成；阶段 2 语言能力 + 示例完成（std 移植待做）**（2026-10-07 / 2026-10-18）
> 关联：bd `Ayanami-language-hfj`（epic）、GitHub #9；依赖 M3（已完成）；M5 unsafe（联动）
> 目标：自然表达自引用结构 / parent pointer / graph / 双向链表 / intrusive / arena+reference /
> AST 内部引用——对自举尤其重要。

## 1. 需求分类（不照搬 Rust）

| 类别 | 典型场景 | 所有权 / 生命周期 | 方案 |
|---|---|---|---|
| **A. 拥有指针（box）** | 树子节点、递归结构、堆上单值 | 拥有；随字段递归 drop | **阶段 1：`Ptr[T]`**（内部即 `HirType::Unique(T)`） |
| **B. 非拥有稳定句柄（arena + index）** | graph/DAG、双向链表、AST 交叉引用、intrusive | arena 拥有全部节点；句柄不拥有 | **阶段 2：`Arena[T]` + `Idx`**（std 为主，基于 `Ptr`/数组） |
| **C. 借用引用存字段** | 视图、parent 借用 | 生命周期（Rust 式 `<'a>`） | **不引入**：NLL 无跨结构生命周期标注；用 B/D 替代 |
| **D. 裸/不安全指针** | intrusive、FFI、极致控制 | unsafe 边界 | M5（`hfj.3` 联动） |

设计原则：**不为循环引用引入共享所有权（RC/GC）**；安全图结构用 arena+句柄表达，
裸指针留给 M5 的 unsafe。

## 2. 阶段 1：`Ptr[T]`（拥有指针）

### 类型与语义
- `Ptr[T]` 映射到内部 `HirType::Unique(T)`（编译器已具备：alloc、递归 drop、深 clone、
  移动语义、装箱、可存字段/数组、可传参/返回）。
- 非 Copy；移动语义；作用域结束递归释放 pointee（`emit_drop_value(Unique)`）。
- 不共享：同一 `Ptr` 只有一个所有者；`Ptr` 可安全复制仅限 `clone`（深拷贝 pointee）。
- 空值：用 `Option[Ptr[T]]`，**不提供裸 null**（安全代码无空指针）。

### 接口
```ayanami
p = Ptr::new(Point { x = 1, y = 2 })   // 堆分配，稳定地址
r = p.get()                            // ref T（借用）
rm = p.get_mut()                       // ref mut T（可变借用）
x = p.x                                // 字段自动解引用（等价 p.get().x）
n = p.method()                         // 方法自动解引用（等价 p.get().method()）
```

### 阶段 1 明确不做
- `take()`（移出 pointee 并只释放盒）：需新的 LIR 取出口；留待阶段 2/按需。
- 共享/弱引用、循环拥有（用阶段 2 或 M5）。
- `Ptr` 的 `==` 指针相等（先按 pointee 比较；指针身份比较留待 unsafe）。

## 3. 阶段 2：arena / stable 句柄

- `Arena[T]`（std）：内部 `ArrayList[Ptr[T]]` 拥有节点，地址稳定；
  `alloc(v) -> Idx`、`get(ref self, Idx) -> ref T`（越界 panic）、`len`。
- `Idx`（std）：`struct Idx { usize }`；首版不加 generation（文档明示悬垂风险）。
- 图/双向链表：节点内 `Idx` 字段 + arena 访问（parent/next/prev 均为 `Idx`）。
- AST：`Arena[Node]` + `Idx` 交叉引用（自举友好）。
- 编译器支持：仅阶段 1 的 `Ptr` 能力；arena 本身是 std。

**阶段 2 语言能力（2026-10-18 落地）**：`ref` 元素地址（`ref arr[i]` / `ref mut arr[i]`——
`place_ptr` 元素地址 → 借用检查 loan 回溯数组根变量）；`[Ptr[T]; n]` 等**复合元素类型**的
定长数组解析；示例 `example/test_arena.aya`（双向链表 + 图 DFS + `live_allocs` 归零）。
- 句柄复制：`Idx` 为结构体（非 Copy），需 `Idx::copy(i)` 显式复制；trivial 结构体自动
  Copy / `#[copy]` 列为后续项。
- std 移植：`Arena[T]`/`Idx` 进 std 子仓（可参照示例；`ArrayList` 需补
  `get(ref self, usize) -> ref T` 式借用访问）。

## 4. 自引用 struct 评估（hfj.4）

问题：parent pointer / 双向链表是否进入安全核心？

| 场景 | 表达方式 | 判定 |
|---|---|---|
| 单向链 / 树 | `Option[Ptr[T]] next`（每个 next 拥有下一个节点） | **支持**（阶段 1，已验证） |
| parent 指针 | `Ptr[T] parent` 拥有父指针 | **不支持**：子拥有父 ∧ 父拥有子 = 双所有权，构造即环，drop 双释放/泄漏 |
| parent 借用 | `ref T parent` 存字段 | **不支持**：借用存字段需跨结构生命周期标注，NLL 生命周期不跨结构 |
| 双向链表 | `Ptr[T] prev` + `Ptr[T] next` 互指 | **不支持**：同 parent 指针，环无法安全释放 |
| 图 / DAG / 交叉引用 | arena + `Idx`（阶段 2，std） | **支持路径**：句柄不拥有，arena 统一释放 |
| intrusive / 极致控制 | `*T` 裸指针（M5 unsafe） | **显式不安全**：生命周期与别名自证 |

结论：
- **不引入安全自引用字段**（无 pin 标注；安全自引用需要稳定地址 + 不移动）。
- 安全图结构统一用 **arena + `Idx`**（阶段 2，std 实现）；`Ptr[T]` 负责拥有式树/链。
- parent / 双向链 / 环需求走 arena+`Idx`（安全）或 M5 裸指针（显式 unsafe）。

`Ptr` 结构的 match 支持（2026-10-17 修复，含回归）：
- `Some(p)` 绑定 `Ptr[T]` 载荷（此前绑定类型被剥成 pointee，IR 类型错配）；
- `match p { ... }`（Ptr 目标自动解引用、不复制指针）；
- `ref a.next`（a: `Ptr[T]`）字段地址（此前漏解引用）；`Struct { p }` 解构移出 Ptr 字段；
- 回归：example/test_ptr_match、tests/runtime_safety/{ptr_chain_match,ptr_match_scrutinee,struct_destructure_owned}。

## 5. 阶段与验收

| 阶段 | 内容 | 验收 |
|---|---|---|
| 1 | `Ptr[T]`：类型/`Ptr::new`/`get`/`get_mut`/字段与方法自动解引用/存字段与数组/传返/深 drop/clone | example：二叉搜索树（`Ptr` 子节点）+ 递归 drop 无泄漏；`Ptr[String]` 深 clone；负例（空/移动后使用） |
| 2 | `Arena[T]` + `Idx`（std；语言能力与示例已完成，std 移植待做） | example/test_arena.aya：双向链表 + 图 DFS，无泄漏（**已完成**） |
| 3 | 联动 M5：裸指针 / intrusive / unsafe 边界 | 见 M5 epic |

## 6. 实现状态

- [x] 阶段 1 `Ptr[T]`：类型映射 `Ptr[T]`↔`Unique(T)`、`Ptr::new`、`get`/`get_mut`（含 `ref Ptr[T]` 接收者）、
  字段/方法自动解引用、字段/数组存储、递归 drop（按类型合成 `__drop_<n>`，弱链接）、
  `ref Ptr[T]` 借用读取；回归 example/test_ptr_tree + tests/runtime_safety/ptr_tree_drop
- [x] hfj.4 评估：parent pointer / 双向链表**不进入安全核心**（§4）
- [x] `Ptr` 与 match 互操作：载荷绑定 / Ptr 目标自动解引用 / `ref` 字段地址（§4 末，2026-10-17）
- [x] 阶段 2 语言能力 + 示例（2026-10-18）：`ref` 索引元素地址（含 `ref mut`）→ 借用检查 loan 回溯
  （`deref_root_var` 索引分支 + origin 取根变量）；`[T; n]` 复合元素类型解析（`[Ptr[T]; n]`）；
  示例 `example/test_arena.aya`（双向链表 + 图 DFS + 无泄漏）；回归 `tests/runtime_safety/arena_linked_list.aya`；
  负例 `ref_local_return` / `ref_mut_through_immutable`
- [ ] 阶段 2 std 移植：`Arena[T]` + `Idx` 进 std 子仓（含 `ArrayList` 借用访问）
- [ ] 阶段 3（M5）
