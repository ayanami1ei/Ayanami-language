# M4：引用存储 / arena / stable object（设计）

> 状态：**设计完成（hfj.1）；阶段 1（`Ptr[T]`）已落地**（2026-10-07）
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

## 4. 自引用 struct 评估（hfj.4）

- 安全自引用需要**稳定地址 + 不移动**（pin）；Ayanami 无 pin 标注。
- 结论：**不引入安全自引用字段**；用 arena+`Idx`（安全）或 M5 unsafe 裸指针（显式）。
- `Ptr[T]` 的 pointee 地址稳定（堆），因此 `struct Node { Ptr[Node] next }` 可表达
  **单向链/树**（每个 next 拥有下一个节点）；环需阶段 2/M5。

## 5. 阶段与验收

| 阶段 | 内容 | 验收 |
|---|---|---|
| 1 | `Ptr[T]`：类型/`Ptr::new`/`get`/`get_mut`/字段与方法自动解引用/存字段与数组/传返/深 drop/clone | example：二叉搜索树（`Ptr` 子节点）+ 递归 drop 无泄漏；`Ptr[String]` 深 clone；负例（空/移动后使用） |
| 2 | `Arena[T]` + `Idx`（std） | example：双向链表 / 图遍历，无泄漏 |
| 3 | 联动 M5：裸指针 / intrusive / unsafe 边界 | 见 M5 epic |

## 6. 实现状态

- [x] 阶段 1 `Ptr[T]`：类型映射 `Ptr[T]`↔`Unique(T)`、`Ptr::new`、`get`/`get_mut`（含 `ref Ptr[T]` 接收者）、
  字段/方法自动解引用、字段/数组存储、递归 drop（按类型合成 `__drop_<n>`，弱链接）、
  `ref Ptr[T]` 借用读取；回归 example/test_ptr_tree + tests/runtime_safety/ptr_tree_drop
- [ ] 阶段 2 `Arena[T]` + `Idx`（std 子仓）
- [ ] 阶段 3（M5）
