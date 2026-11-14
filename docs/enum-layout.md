# Phase 7 剩余：枚举布局优化（niche / boxed variant / indirect payload）设计

> 状态：**N1/N2/N3 已实现**（2026-10-20；N2/N3 采用统一 16 字节阈值装箱策略）。关联：bd `Ayanami-language-2ru.2`（task）/
> `2ru`（Phase 7 epic）；GitHub #40（Phase 7）/ #61（niche / boxed variant / indirect payload）。
> 相关：`docs/optimization.md`（M-opt 语义优化）、`docs/knowledge-graph.md` §2（枚举布局现状）。

## 1. 现状（2026-10）

枚举表示为**标签 + 每变体载荷结构**（`collect_enum.rs`）：

```
struct_defs[E]      = [_tag: int, _data_V1: Named(E_V1), _data_V2: Named(E_V2), ...]
struct_defs[E_Vi]   = 变体字段（tuple → _0/_1/...；named → 原字段名；空 → 无字段）
```

- `%struct.Option = { i64, %struct.Option_Some, %struct.Option_None }`；
  `Option[int]` = 16 字节 / align 8；`Option[Ptr[T]]` = 16 字节（tag 8 + 指针 8）——**指针的空值
  位型（null）完全浪费**；
- 构造 / 匹配 / 字段访问（`_tag`、`_data_V._i`）都走**统一的结构体字段模型**
  （`match_pattern.rs` 的 `variant_tag` / 字段下标、`place.rs` 的字段地址）；
- drop 按字段递归（tag + 各载荷结构）。

关键约束：**任何改变表示的优化都必须保持（或特殊化）上述字段访问模型**——这是本设计
最大的影响面来源。

## 2. 目标与收益

| 目标 | 示例 | 收益 |
|---|---|---|
| **niche**：用载荷类型中的无效位型承载判别 | `Option[Ptr[T]]` / `Option[ref T]` / `Option[*T]` / `Option[fn(..)]` | 16 → **8 字节**（-50%），缓存友好 |
| **boxed variant**：大载荷变体改为间接存储 | `enum E { Small(int), Big([int; 4096]) }` | 枚举尺寸 → 最大“直接”变体 |
| **indirect payload**：超阈值载荷统一装箱 | 同上 | 同上（策略不同：按阈值 vs 按变体标注） |

三者都**不改变语言语义**（构造/匹配/字段访问的源码形态不变）。

## 3. 方案评估

### 3.1 niche（N1 已实现）

**表示（实现口径）**：niche 枚举 = **去掉 `_tag` 字段**的单指针槽：

```
struct_defs[E] = [_data_Some: Named(E_Some), _data_None: Named(E_None)]
```

- 布局 = 载荷变体结构体（8 字节）+ 空变体结构体（0 字节）→ **8 字节**（16 → 8，-50%）；
- 判别由载荷槽空值承载：Some = 指针值本身；None = 载荷槽零值（`zeroinitializer` = null）；
- `_data_*` 字段序即 tag 序——形态自描述、跨模块一致，无索引重映射（`_tag` 槽消失，
  其余字段位置与 tag 形态相同）；
- `_tag` 读取特殊化：`(reinterpret<ptr→int>(v._data_P._0) == 0) as int`（载荷 tag 为 0 时；
  否则 `!= 0`）——沿用“比较保留操作数类型 + `SBoolToNum`”的既有转换口径；
- drop：既有“按字段递归 + 非活跃变体零初始化 + null 释放安全”机制直接适用（无需改动）。

**可 niche 载荷**：`Ptr[T]`（`Unique(具名/胖指针/数组/8 字节基元)`）、`ref T`、`fn(..)`。
**不含 `*T`**——null 是合法裸指针值，`Some(null)` 会塌缩为 `None`（与 Rust 口径一致；
本文档早前版本列为可 niche，实现时修正）。

**判定时机**：声明（`collect_enum` / `lower_items`）与泛型实例化（`instantiate_named` /
构造侧单态化）——只看类型结构 + 具体载荷类型（确定性）；`Option[T]` 按单态化后的 T 判定
（`Option[Ptr[X]]` niche、`Option[int]` 不）。

**影响面（实现位置）**：
1. 布局 / 发射：单指针槽的 `struct_defs` → `type_layout` / `emit_struct_defs` / 结构体字面量 /
   字段访问**自动正确**（无需特殊化）；
2. 构造：`expr_enum`（跳过 `_tag` 写入；非活跃变体保持零值）+ 适配路径
   （`instantiate_enum_value` 与结构体字面量字段丢弃基枚举 `_tag`）；
3. 匹配：`match_pattern`（tag 条件 → 空值判定；载荷绑定下标按实际字段表）；
4. `_tag` 读取：`expr_access`（含 `?` 算子路径）→ `ctx_niche::build_niche_tag_expr`；
5. 枚举判定/派发：`is_enum_type`、`variant_tag`、变体方法派发（`expr_method`）、
   穷尽性计数（`enum_variant_count`）niche 感知；
6. **C ABI 边界**：extern 签名中的 niche 枚举按新表示（8 字节）——与 C 的 tag 布局不一致；
   跨 FFI 的布局契约属非目标（§5），需 `#[repr]`-类机制另行设计（文档早前“边界回退旧表示”
   验收项据此修订）。

### 3.2 boxed variant / indirect payload（N2/N3 已实现）

- **boxed variant / indirect payload**：变体载荷结构 **> 16 字节** → 枚举字段类型改为
  `Unique(E_Vi)`（堆装箱；构造时分配、匹配经指针字段访问、drop 释放）；
- **N3 策略收敛**：N2 与 N3 采用**同一机制**、唯一触发条件 = 16 字节阈值（不做 `#[boxed]`
  逐变体标注；布局不承诺跨版本稳定，见 §5 非目标）；
- 实现要点：`apply_boxing_rewrite`（声明/实例化两处，先于 niche 判定）；
  构造字面量对 boxed 槽用 `ToUnique` 装箱；非活跃 boxed 槽为零值（null，drop 安全）；
- **保守边界**：**声明期泛型枚举不装箱**（载荷为未实例化类型，布局无意义；具体实例化时再装箱）；
  载荷引用“未实例化的泛型基类型”（推断期临时实例化，如 `Opt<Opt>`）时同样跳过——
  其布局与真实实例化不一致，装箱会破坏 ABI 一致性（此前 `Opt<Opt>` 与 `Opt<Opt<int>>`
  同为 24 字节的巧合依赖）；
- 与 N1 组合：boxed 槽本身即指针 → 2 变体（1 空）枚举仍可 niche
  （`enum OptBig { Big(Big), None }` → 8 字节）；`_tag` 表达式直接读槽（不再取 `_0`）；
- 匹配绑定：move 路径的 `lower_field_addr_of` 对指针型字段返回字段值（指针）而非字段地址。

## 4. 阶段与验收

| 阶段 | 内容 | 验收 |
|---|---|---|
| **N1**（完成 ✓） | niche：`Option[Ptr/ref/fn]` 单指针表示 | 正例 `example/test_niche_option.aya`：`size_of[Option[Ptr[T]]]() == 8`（ref 同）、构造/匹配/字段访问/递归 drop 无泄漏（`None`/`Some` 全路径，live_allocs == 0）；回归全绿；C ABI 边界见 §3.1-6 |
| **N2**（完成 ✓） | boxed variant（16 字节阈值） | 正例 `example/test_enum_boxed.aya`：`E` 40 → 24 字节、`OptBig`（N1 组合）8 字节；构造/匹配/字段访问/递归 drop 无泄漏；阈值边界（≤16 不装箱） |
| **N3**（完成 ✓） | indirect payload 策略收敛 + 文档 | 统一 16 字节阈值（与 N2 同机制）；布局稳定性说明见 §5（不承诺跨版本布局） |

## 5. 非目标

- Rust 级“全面 niche filling”（多 niche 组合 / 嵌套 niche 搜索）——仅做**可静态判定的简单形态**；
- 布局稳定性承诺（枚举/结构体布局可随版本变化，跨 FFI 边界需显式 `#[repr]`-类机制，另行设计）；
- 与 boxed/indirect 之外的通用“小字符串优化”等无关优化。

## 6. 风险与缓解

- **字段访问特殊化遗漏** → 以 N1 的验收用例覆盖 `_tag`/`_data_V._i` 的所有访问路径（构造、
  匹配、字段地址、嵌套 `ref`）；
- **跨模块/ABI 不一致** → niche 判定只看类型结构（同一编译器确定性 ✓）+ C ABI 边界回退；
- **回归面大** → 分阶段（N1 限定形态先行），每阶段跑全量回归 + IR 快照 + std 套件。

## 7. 实现状态

- [x] **N1（2026-10-19）**：niche 枚举（`Option[Ptr[T]]` / `Option[ref T]` / `Option[fn]` → 8 字节）
      - 判定与改写：`src/hir/niche.rs`（纯形态判定）+ `src/hir/lower/ctx_niche.rs`（Ctx 侧改写与
        `_tag` 特殊化）；
      - 接线：`collect_enum` / `lower_items`（声明）、`ctx_mono`（实例化 + 适配）、`expr_enum`（构造）、
        `expr_access` / `expr_ops1`（`_tag`）、`match_pattern` / `match_lower` / `expr_method`（匹配与派发）；
      - 连带修复（潜伏不一致）：
        1. `sig_str_to_hir` 现解析 `Ptr<T>` → `Unique`（此前泛型实参回解析退化为 `Named("Ptr<..>")`：
           尺寸按 16 计、丢所有权语义）；
        2. `type_to_string_generic` 的 `ref` 拼写补空格（`refNode` → `ref Node`，与解析器/显示一致）；
        3. `size_of` / `align_of` / `offset_of` 求值前先实例化类型（此前落回基枚举的泛型布局，
           `Option[Ptr[X]]` 报 24）；
      - 验收：`example/test_niche_option.aya`（8/8/16 尺寸 + 构造/匹配/字段访问 + 泄漏 0）；
        `test_ptr_tree` / `test_arena` / `test_ref_*` 等既有指针用例回归通过；std 套件重建后全绿。
- [x] **N2/N3（2026-10-20）**：boxed variant / indirect payload（统一 16 字节阈值）
      - 声明/实例化时 `apply_boxing_rewrite`：载荷结构 > 16 字节 → 枚举字段 `Unique(E_V)`；
        构造 `ToUnique` 装箱、非活跃槽 null、匹配经指针字段访问、drop 自动释放
      - 与 N1 组合：boxed 槽即指针 → niche 仍生效（`OptBig` 8 字节）；`_tag` 表达式直读槽
      - 匹配 move 路径修复：`lower_field_addr_of` 对指针型字段返回字段值（而非地址）
      - 验收：`example/test_enum_boxed.aya`（尺寸 24 / 8、全路径、泄漏 0）
