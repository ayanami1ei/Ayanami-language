# 统一引用关系模型（Extensible Reference Model）设计提案

> 状态：**设计提案；R1 / R2 / R3 已落地**（2026-10-18）。
> 关联：bd `Ayanami-language-tex`；原始规划记录：`docs/future-directions.md` §一。
> 联动：M4 引用存储（`Ptr[T]` 阶段 1 ✓、arena 语言能力 + 示例 ✓、std 移植 `dka`）、
> M5 unsafe（`*T` ✓ P1–P3）、`#[copy]` 句柄（✓）、环境注入（解析器/注册表可经 `env::` 提供，见 `docs/env-injection.md`）。

## 1. 目标与原则（核心决策）

- **纯接口、无标注**：解引用 / 取引用 / 解析全部由协议**接口方法**表达；
  编译器按**方法形态**识别协议（与迭代器协议同风格：任意含 `next() -> Option[T]` 的类型 ✓），
  并据此做别名分析、解析去重与内联优化——不需要名字表、不需要注解；
- 传统指针（`ref` / `Ptr[T]` / `*T`）作为协议的**零开销实现**：内联后与裸访问等价；
- 统一的是**引用关系的语义**，不强制统一存储表示或访问成本；不引入 RC/GC；
- 三分边界：静态可证明约束 / 运行时检查 / 用户契约（可用 `#[requires]`/`#[ensures]` 可选表达）。

## 2. 协议（方法形态，编译器识别）

### 2.1 自含型（引用即值，如 `Ptr[T]` / `ref T` / `*T`）

```
任意类型 C，若含：
  fn get(ref self) -> ref T                  // 解引用（不可变）
  fn get_mut(ref mut self) -> ref mut T      // 可变解引用（可选）
则 C 实现“引用协议”，目标类型 = T。
```

- `Ptr[T].get/get_mut` ✓ 已是此形态（M4 阶段 1）；
- `ref T` 的原生解引用 ✓；`*T` 裸指针（unsafe 边界内）✓；
- 字段/方法自动解引用（`p.x` ≡ `p.get().x` ✓）已是该协议的编译期行为。

### 2.2 句柄型（容器 + 句柄，如 arena + `Idx`）

```
任意类型 C（容器/解析器），若含：
  fn resolve(ref self, H h) -> ref T               // 句柄 → 目标
  fn resolve_mut(ref mut self, H h) -> ref mut T   // 可选
则 (C, H) 实现“解析协议”，目标类型 = T，句柄类型 = H。
```

- arena 示例的 `Arena[T].get(Idx) -> ref T` ✓ 已是此形态（M4 阶段 2）；
- 句柄建议 `#[copy]`（✓ 已落地：轻量值语义、自由复制）；
- 失效检查（generation 等）属**运行时检查**（§4），协议本身不强制。

### 2.3 取引用（反向，可选）

```
fn to_ref(ref self) -> H        // 目标 → 表示
```

- 反向映射可能不唯一或失败：存在时编译器可用于「对象 → 句柄」的自动转换
  （序列化引用字段等，R4）；不唯一性由用户契约约束（§4）。

## 3. 编译器识别的语义（按协议优化）

| 语义 | 依据 | 用途 |
|---|---|---|
| 目标类型 / 身份 | 协议方法签名中的 `T` | 类型检查；字段/方法自动解引用（已有 ✓） |
| 有效性域 | 容器 `ref self` 的借用 | 解析结果的引用绑定到容器 loan（NLL ✓）；容器失效 → 引用失效 |
| 别名 / 独占 | `get` vs `get_mut`（`ref` / `ref mut`） | 现有借用检查规则直接适用 ✓ |
| 解析去重（CSE） | 同容器 + 同句柄的重复解析 | 循环内 `a.get(i)` 去重；纯解析可提升 |
| 内联 / 零开销 | 协议方法可内联 | `Ptr` / `ref` / `*T` 内联为裸访问（R1 验证） |
| 序列化引用 | `to_ref` + 目标身份 | 引用字段 → 表示（R4 对象图） |

**IR 约定**：协议调用在 HIR/MIR 保留“解析”标记（不降级为普通调用）——供别名分析与后续优化；
LIR 内联后消除（零开销路径不残留标记）。

## 4. 静态 / 运行时 / 契约

- **静态可证明**：自含型与 arena 型解析（借用域编译期绑定 ✓）；`#[copy]` 句柄值语义 ✓；
- **运行时检查**：句柄失效（generation）、注册表查找失败（返回 `Option` 或 panic，由用户选）；
- **用户契约**：`to_ref` 唯一性、解析稳定性——可选 `#[requires]`/`#[ensures]` 表达（不强制 ✓）。

## 5. 阶段与验收

| 阶段 | 内容 | 验收 |
|---|---|---|
| **R1**（完成 ✓） | 协议形态统一：`Ptr[T]` / `ref` / `*T` 归入自含型；任意 `get`/`get_mut` 类型自动解引用（字段读/写/方法） | 零开销验证：release 产物中协议 getter 调用 = 0（objdump）；负例：借用冲突 / 不可变借出可变 |
| **R2**（完成 ✓） | 句柄型语义：arena + `Idx` 有效性（越界句柄 / 容量越界 → panic 101）；解析成本验证（去重由 release opt 承担） | 双向链表 / 图用例（已有 ✓ + 边界检查）；release objdump：`Arena_get` 调用 = 0；越界用例 `arena_oob`（101）；std 移植 `dka` |
| **R3**（完成 ✓） | 用户自定义解析器（ID + 注册表）：`HashMap[int, Idx]` + arena 两级解析；检查式 API（`contains` + `resolve`） | 正例：`example/test_ref_registry.aya`（注册/解析/移除失效）；负例：`ref_registry_missing.aya`（缺失 ID → 101） |
| **R4** | 序列化 / 对象图（引用字段 → 表示；对象图重建） | 往返用例；循环 / 多对一 / 失效 ID 边界（区分“序列化引用”与“序列化完整对象图”） |

## 6. 非目标

- 注解式引用声明（本设计**不引入标注** ✓ 纯接口）；
- 共享所有权（RC/GC）/ 弱引用；
- 远程引用等（远期；协议可扩展 ✓）；
- 统一布局 / 统一成本（明确不承诺）。

## 7. 实现约定（要点）

1. 协议识别按**方法形态**（同迭代器 ✓）：编译器无需名字表；std 提供规范接口供用户实现；
2. 协议调用保留“解析”语义至 MIR（别名 / CSE / 去重），LIR 内联消除（零开销）；
3. 句柄值语义优先 `#[copy]`；失效检查显式（运行时或契约）；
4. 与 M4/M5 既有实现不冲突：`Ptr` / `*T` / arena 是协议的首批实现 ✓；
5. 测试口径同仓库（正/负例 + `runtime_safety` + IR 快照）。

## 8. 实现状态

- [x] **R1（2026-10-18）**：协议形态统一 + 自动解引用 + 零开销验证
      - 识别：`get`/`get_mut`（单参数 `ref self`，返回 `ref T`）方法形态；覆盖非泛型与泛型 impl
        （`method_fn_ids` + `generic_fns`）；目标类型按接收者实例实参替换（`ref_protocol.rs`）
      - 自动解引用：字段读（`expr_access.rs`）、字段写（`stmt_lower.rs`，经 `get_mut`）、
        方法调用（`expr_method.rs` 回退）——协议类型与 `Ptr`/`ref`/`*T` 同权
      - 借用规则：合成调用与显式调用同规则（不可变引用不能借出可变；loan 冲突照常报错）
      - 连带修复：结构体字面量对**复合字段**（`Ptr[T]` 等）的泛型实参推断（改用 P4c `unify`；
        此前裸基名导致字段访问无细化点）
      - 零开销验证：`example/test_ref_protocol.aya` release 产物中协议 getter 调用 = 0（objdump）
      - 回归：`test_ref_protocol`（正例）+ `ref_protocol_borrow_conflict` /
        `ref_protocol_immutable_write`（负例）
      - 已知边界（沿既有模型）：rvalue 引用（如 `p.get_mut().x = 2` 的调用结果）不登记 loan
        ——`Ptr` 原生路径同形亦然（借用检查为变量级）；如需收紧另立设计。
- [x] **R2（2026-10-18）**：句柄型（arena + `Idx`）有效性语义 + 解析成本验证
      - 示例与运行时用例补**有效性检查**：越界句柄 / 容量越界 → `panic_at`（101，与数组越界一致）；
        回归 `tests/runtime_safety/arena_oob.aya`（101）、`arena_linked_list`（含检查，release 亦通过）
      - 解析去重（CSE）：release 下协议解析完全内联（objdump：`Arena_get` 调用 = 0），
        冗余解析由 `opt -O3` 消除——**不需要编译器专用 CSE pass**（设计修订；此前验收项据此更新）
      - 句柄型 `resolve` 形态识别：留待 R3/R4 的真实消费者（无消费者的识别不写死代码）
- [x] **R3（2026-10-18）**：用户自定义解析器（ID + 注册表）
      - 示例 `example/test_ref_registry.aya`：`HashMap[int, Idx]` + arena 两级解析；检查式 API
        （`contains` + `resolve`，缺失 → panic 101）；移除后 `contains=false`（arena 节点保留——批量释放语义）
      - 运行时用例 `tests/runtime_safety/ref_registry_missing.aya`（101 + "unknown id"）
      - 连带修复（借用检查）：调用返回借用的实参扫描支持**字段接收者**
        （`self.arena.get(i)` → `deref_root_var` 回溯根变量 + origin 口径）
      - 已知边界：`Option[ref T]` 可构造/传递，但**返回值内的引用不登记 loan**（聚合载体未跟踪）
        ——解析器建议用检查式（panic）路径；如需跟踪另立设计
- [ ] R4（序列化 / 对象图）
