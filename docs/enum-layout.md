# Phase 7 剩余：枚举布局优化（niche / boxed variant / indirect payload）设计

> 状态：**设计（未实现）**（2026-10-18）。关联：bd `Ayanami-language-2ru.2`（task）/
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

### 3.1 niche（推荐先行；N1 限定形态）

- **识别（N1）**：恰好 2 个变体、其中一个**无字段**、另一个**恰有 1 个字段且该字段为
  “可 niche 类型”**（`Ptr[T]` / `ref T` / `*T` / `FnPtr` / `Closure(静态)`）；
- **表示**：该枚举 = 单字段值（指针）；空变体 = `null`；非空变体 = 指针值本身；
- **影响面**（必须逐项处理）：
  1. 布局：`hir/layout.rs::type_layout`（niche 枚举 = 字段尺寸 ✓）；`lir/emit` 的
     `%struct.E` 不再发射（或发射为单字段 ✓）；
  2. 构造：`E::Some(p)` → 直接存 p；`E::None` → null；
  3. 匹配：`_tag` 读取 → `is null` 判断；`_data_V._i` 字段地址 → **返回枚举值本身的地址**
     （指针槽 ✓）——需要“字段访问特殊化”；
  4. drop：只 drop 指针槽（pointee 归 `Ptr` 自己 ✓）；
  5. `.lcl`/跨模块：同一编译器一致 ✓；**C ABI 边界回退**：`#[extern]`/`#[export]` 签名中的
     此类枚举**不使用 niche**（保持 tag 表示，保证与 C 的布局预期一致——或文档化为不保证 ✓）；
  6. 泛型：`Option[T]` 的单态化按具体 T 判定（`Option[Ptr[X]]` niche、`Option[int]` 不 ✓）。
- **实现位置（建议）**：`hir/lower/collect_enum.rs` 增加 `niche` 判定与标记
  （enum 元数据：`NicheKind`）；`match_pattern.rs`/`place.rs`/`lir` 按标记走特殊化路径。

### 3.2 boxed variant / indirect payload（N2/N3）

- **boxed variant**：变体载荷超过阈值（或变体标注 `#[boxed]`）→ 载荷结构改为
  `Ptr[E_Vi]`（构造时分配、匹配时解引用、drop 释放 ✓）；
- **indirect payload**：统一按尺寸阈值装箱（与 boxed 同机制，触发条件不同）；
- 影响面与 3.1 类似但更局部（载荷字段类型替换 ✓——字段访问模型保留 ✓，只是多一层指针 ✓）。

## 4. 阶段与验收

| 阶段 | 内容 | 验收 |
|---|---|---|
| **N1** | niche：`Option[Ptr/ref/*T/fn]` 单指针表示 | 正例：`size_of[Option[Ptr[T]]]() == 8`、构造/匹配/字段访问/递归 drop 无泄漏（含 `None`/`Some` 全路径）；回归全绿；C ABI 边界保持旧表示 |
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
