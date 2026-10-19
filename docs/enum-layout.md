# Phase 7 剩余：枚举布局优化（niche / boxed variant / indirect payload）设计

> 状态：**N1 已实现**（2026-10-19）；N2/N3 未实现（2026-10-18 设计）。关联：bd `Ayanami-language-2ru.2`（task）/
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

### 3.2 boxed variant / indirect payload（N2/N3）

- **boxed variant**：变体载荷超过阈值（或变体标注 `#[boxed]`）→ 载荷结构改为
  `Ptr[E_Vi]`（构造时分配、匹配时解引用、drop 释放 ✓）；
- **indirect payload**：统一按尺寸阈值装箱（与 boxed 同机制，触发条件不同）；
- 影响面与 3.1 类似但更局部（载荷字段类型替换 ✓——字段访问模型保留 ✓，只是多一层指针 ✓）。

## 4. 阶段与验收

| 阶段 | 内容 | 验收 |
|---|---|---|
| **N1**（完成 ✓） | niche：`Option[Ptr/ref/fn]` 单指针表示 | 正例 `example/test_niche_option.aya`：`size_of[Option[Ptr[T]]]() == 8`（ref 同）、构造/匹配/字段访问/递归 drop 无泄漏（`None`/`Some` 全路径，live_allocs == 0）；回归全绿；C ABI 边界见 §3.1-6 |
| **N2** | boxed variant（阈值或标注） | 正例：大载荷枚举尺寸下降；匹配/drop 正确；负例：阈值边界 |
| **N3** | indirect payload 策略收敛 + 文档 | 布局稳定性说明（不承诺跨版本布局 ✓） |

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
- [ ] N2（boxed variant）、N3（indirect payload 策略收敛 + 文档）
