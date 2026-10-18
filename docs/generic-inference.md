# Phase 4：泛型推断统一化（设计）

> 状态：**设计完成；P4a/P4b/P4d 已实施，P4c/P4e 待做**（2026-10-17 / 2026-10-18）
> 关联：bd `Ayanami-language-upt`（epic）、GitHub #48（epic）/ #49（设计问题）；`upt.1` 本文档。
> 依赖说明：bd 将 Phase 4 挂在 M4 之后（软依赖）；P4a/P4b 与 M4 无耦合，可先行。

## 1. 目标

解决「调用点无法推导泛型实参」类问题，把推断从**各语法位置各自一套规则**收敛为统一链路：

```
constraint generation → unification → type inference → generic instantiation
```

并明确 **coercion（隐式转换）与 inference 的关系**：参数/返回值/接口约束/嵌套泛型/字面量/空指针等位置先按期望类型产生约束，再在约束解上做转换；与编译期约束求解（const fn / 类型级常量）方向合并。

## 2. 现状（探测矩阵，2026-10-17）

已可用（回归覆盖）：

| 场景 | 示例 | 状态 |
|---|---|---|
| 引用/值参数推导 | `fn get[T](ref Box[T])` ← `get(ref b)` | ✓ |
| 标量参数 | `fn wrap[T](T x)` ← `wrap(5)` | ✓ |
| 多参同型 | `fn pair[T](T, T)` ← `pair(1, 2)` | ✓ |
| 数组元素 | `fn sum[T]([T])` ← `sum([1,2,3])` | ✓ |
| 接口约束 | `fn same[T: Eq2](ref T, ref T)` | ✓ |
| 方法接收者 | `impl[T] Box[T] { fn get(ref self) -> T }` ← `b.get()` | ✓ |
| 限定名显式实参 | `ArrayList::new[int]()` | ✓ |
| 嵌套**实参** | `Box[Box[int]]` → 形参 `Box[T]`（`first(b).v`） | ✓ |
| 结构体字面量实参推断 | `Box { v = 42 }` → `Box<int>`；`Pair { first = 1, second = "s" }` → `Pair<int,String>` | ✓ |
| 冲突诊断（P4d） | `pair(1, "s")` → `generic parameter \`T\` ... inferred as \`int\` ... but \`String\` here` | ✓ |

缺口：

| # | 场景 | 现象 |
|---|---|---|
| a | 单标识符显式实参 `make[int]()` | **解析失败** `expected expression`（显式泛型调用仅支持含 `.`/`::` 的限定名） |
| b | 泛型体内嵌套字段**经局部变量**：`x = b.v; x.v`（`b: ref Box[Box[T]]`） | 报 `field access v on generic parameter T` —— `x` 的类型被收窄为 `T` 而非 `Box[T]` |
| c | 嵌套**调用点**特化：`inner(ref Box[Box[int]])`（形参 `ref Box[T]`） | llc 类型错误：`%t7 defined with type %struct.Box<int> but expected i64`（替换/装箱不一致） |
| d | 规则分散 | 实参→形参 `infer_generic_from_param`、后续用法 `usage_infer`、调用点 `specialize_generic_call`、约束检查 `generic_types`、重载解析各自实现；**冲突诊断已落地（P4d）**，统一约束表示待 P4c |

代码入口：`helpers/convert.rs::infer_generic_from_param`、`body/generic_specialize.rs`、`body/generic_types.rs`、`body/generic_check.rs`、`body/usage_infer.rs`、`body/overload_resolve.rs`、`helpers/types.rs::sig_str_to_hir`/`split_generic_args`。

## 3. 设计

### 3.1 统一推断上下文

```
InferCtx {
    subst: HashMap<Symbol, HirType>,      // 泛型参数 → 已解类型
    obligations: Vec<Obligation>,         // 待解约束（接口 bound / 数值约束 / 生命周期省略）
    conflicts: Vec<(Symbol, HirType, HirType, Span)>,  // 冲突记录（诊断用）
}
```

- `unify(expected: &HirType, actual: &HirType, span) -> Result<()>`：**递归**结构化匹配：
  - `Named("Box<A>")` vs `Named("Box<B>")` → 拆 `split_generic_args` 递归 unify（现有 `sig_str_to_hir` 已能解析角括号泛型名）；
  - `Unique/Ref/RawPtr/FatPtr/Closure/FnPtr/ArraySized` → 递归内层；
  - `Named(参数名)` 且未解 → 绑定 `subst`；已解 → 递归 unify 已解值与实际；
  - 不一致 → 记录冲突（位置化诊断）。
- `obligation`：接口 bound `T: I` 在 `T` 解出后校验方法集（复用 `generic_types.rs` 现有检查）；数值/字面量约束（如 `[T: Ord]`）。
- `resolve()`：全部参数已解且无冲突 → 得到 `Vec<HirType>` 供 `instantiate`；否则报错（含未解参数与冲突点）。

### 3.2 各位置只做「约束收集」

| 位置 | 约束 |
|---|---|
| 调用实参 | `unify(形参类型, 实参类型)`（含 `ref`/`Ptr` 包装穿透） |
| 返回类型（赋值/返回上下文） | `unify(期望类型, 表达式类型)`（用于字面量/空指针/枚举变体实例化） |
| 结构体/枚举字面量字段 | 字段期望类型 vs 值类型 |
| 方法接收者 | `unify(impl 接收者类型, 接收者类型)` |
| 数组元素 | 元素期望类型 vs 各元素 |
| 接口 bound | `obligation` |

coercion 顺序：**先 unify 产生约束并解出泛型实参**，再按期望类型 `coerce_expr`（字面量适配、`null`、接口装箱等），避免「先转后推」导致的信息丢失。

### 3.3 嵌套泛型

- 泛型名统一 `Base<A,B>` 角括号表示；unify 时按 `split_generic_args` 拆解递归（不展开为字符串匹配）。
- `Box[Box[T]]` 的字段类型替换必须**整体替换**为 `Box<T>`（现 bug b：被收窄为 `T`）。
- 特化（instantiation）时按解出的实参逐层替换（现 bug c：单态化装箱不一致）。

### 3.4 显式实参

- 解析：单标识符 `f[T](args)` 与限定名 `Ns::f[T](args)` 统一走显式通道；与索引 `a[i](x)` 的歧义用「括号内容可解析为类型列表且后随 `(`」+ 回退（`pos` 保存/恢复）解决。

## 4. 阶段与验收

| 阶段 | 内容 | 验收 |
|---|---|---|
| **P4a** | 解析 `f[T](x)`（单标识符显式实参） | 正例：`make[int]()`、`pair[int](1,2)`；负例：`a[i](x)`（索引调用）不受影响；回归全绿 |
| **P4b** | 嵌套泛型 unify/替换：修 b（局部变量收窄）、c（特化装箱） | `x = b.v; x.v`（`Box[Box[T]]`）通过；`inner(ref Box[Box[int]])` 生成正确 IR 并运行；负例：嵌套不匹配报错位置化 |
| P4c | 收敛 `infer_generic_from_param`/`usage_infer`/`specialize_*` 到 `unify`，删除重复规则 | 行为不回退（全量回归 + IR 快照），代码入口减少 |
| **P4d** | 诊断：未解参数/冲突位置化（期望 vs 实际） | 负例覆盖；错误信息含位置与类型对 |
| P4e | 与编译期约束求解（const fn/类型级）接口预留 | 设计说明 + 最小接入点 |

## 5. 实现状态

- [x] **P4a** 解析单标识符显式实参 `f[T](args)`（歧义消解：类型列表须「像类型」——关键字类型/引用/指针/数组/嵌套泛型/大写开头；`a[i](x)` 保持索引调用）
- [x] **P4b** 嵌套泛型：
      - `infer_generic_from_param` 对嵌套模式（`Box[Box[T]]` ← `Box<Box<int>>`）**递归推导**；
      - `GType::Struct(base, args)` 携带泛型实参，字段/方法返回按接收者实参替换（`x = b.v; x.v` 不再收窄为 `T`）；
      - 附带：**返回类型一致性检查**（`return Box[int]` 当 `int` 返回不再静默生成坏 IR；`Fn`→`FnOnce`/数组尺寸擦除/接口装箱/`null` 字面量/未标注 lambda 延后校验均放行）
- [x] **P4d** 位置化诊断：推断冲突（`inferred as A from earlier argument(s) but B here`，数值互转/所有权包装放行）；未解参数保留调用点位置
- [x] 结构体字面量实参推断（`Box { v = 42 }` → `Box<int>`，与枚举构造 `Option::Some(1)` 一致；`Ptr[T]`/`Box[T]` 等复合字段类型留给使用点细化）
- [x] 特化失败回滚：修复“僵尸签名”（失败的特化仍注册，重试去重命中 → 调用引用无函数体、真实错误被吞）
- [ ] P4c 统一到 `unify`（收敛重复规则）
- [ ] P4e 编译期约束求解接口
