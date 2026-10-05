# Typeclass / Self / 关联类型设计（#84 ①）

> 状态：设计提案（2026-10）。实现分阶段，见文末排期；当前实现状态在文末维护。

## 目标

- 真正的 `FromStr` / `Into` 惯用法：`int::parse(str)`、`T::parse(str)`、`s.into()`。
- 接口方法可返回 `Self`（实现类型）。
- 关联类型：`type Err`，泛型代码中可写 `T::Err`。
- 保持现有「结构匹配 + 单态化」模型，不引入运行时字典传递（dictionary passing）。

## 现状（2026-10）

| 能力 | 现状 |
|---|---|
| 接口声明 | `interface ToString { fn to_string(self) -> String; }`（`src/parser/parser/enum_iface.rs`） |
| 实现方式 | Go 式**结构匹配**：方法名/形参/返回类型齐全即自动实现，无显式 `impl X: Y` |
| 泛型约束 | `[T: Ord]`，约束是接口名；特化时按方法集检查（`body/generic_specialize.rs` Step 1.5） |
| 动态分派 | `FatPtr` + vtable（`body/iface_match.rs::register_generic_vtable`）；泛型走单态化 |
| `Self` 类型 | 仅 AST/格式化存在（`Type::Self_`），HIR 无解析 |
| 关联类型 | 无 |
| 关联函数 | 无（接口方法都带接收者）；`T::method` 调用语法无 |
| 显式 impl | 无 |
| 返回类型驱动推断 | 无（泛型只能从形参推断或显式 `[T]`，`parse::<T>` 的显式形式可用） |

## 设计总原则

- **单态化为准**：静态调用在特化时把 `Self`/关联类型替换为具体类型；vtable 只在 `FatPtr`
  动态分派时需要。
- **结构匹配继续有效**：现有代码不迁移；显式 `impl` 是增强项（关联类型、静态方法、消歧）。
- **不做高阶类型**：关联类型只允许具体类型或类型参数，无 `type F<T>`。

## T1：`Self` 类型

```ayanami
interface FromStr {
    fn parse(ref str) -> Self;   // 返回实现类型
}
```

语义：

- 接口方法签名中的 `Self` = 实现接口的具体类型；impl 方法中 `Self` = impl 目标类型。
- 结构匹配时 `Self` 视为「与实现方法返回类型一致」：`iface_match` 的签名比较加入
  Self 规则（返回位置 `Self` 匹配任意具体类型；形参位置 `Self` 匹配实现类型）。
- 泛型体内 `T: FromStr`：`T.parse(s)` 返回 `T`，特化后为具体类型。
- vtable：`register_generic_vtable` 生成的签名中 `Self` → 具体类型。
- HIR：新增显式变体 `HirType::SelfType`（避免与用户类型名 `Self` 冲突）；在接口注册、
  impl 方法签名、特化替换（`helpers/substitute.rs`）三处消解。

验收：

- `interface FromStr { fn parse(ref str) -> Self; }` + `impl int { fn parse(ref str) -> int }`；
- 泛型 `fn read[T: FromStr](ref str) -> T { return T.parse(str) }`（依赖 T2）；
- 现有结构匹配用例零回归。

## T2：关联函数与类型限定调用

```ayanami
interface FromStr {
    fn parse(ref str) -> Self;
}

n  = int::parse("42")    // 类型限定调用（无接收者）
n2 = parse[int]("42")    // 显式泛型实参（现有 `ns.fn[T]` 形式的泛化）
```

语义：

- 接口方法无 `self` / `ref self` 接收者即关联函数；结构匹配要求实现类型存在同名关联函数。
- 解析顺序：值方法 → 类型关联函数 → 泛型特化。`T::parse` 中 `T` 为泛型参数时**推迟到
  特化**（记录待解析调用，特化时按具体类型重放，类似现有接口方法调用的处理）。
- 语法：`Type::fn(args)` 复用 `::`；与命名空间 `ns.fn` 的歧义按「前缀是已声明类型则视为
  类型」处理，否则按命名空间；两者同名时类型优先。
- `parse[int](...)` 复用现有显式泛型实参解析（`lower_fn_call` 的 `explicit` 路径）。

验收：

- `int::parse`、`Point::parse`、`parse[int]` 三种形式；泛型体内 `T::parse`；
- 类型限定调用与命名空间调用不冲突。

## T3：显式 impl 块

```ayanami
impl FromStr for Point {
    fn parse(ref str) -> Self { ... }   // Self = Point
}
```

语义：

- `impl Trait for Type` 注册到接口实现表，优先于结构匹配；一个类型可对不同接口有多个 impl。
- 显式 impl 中的方法同时是该类型的普通方法（与 `impl Point { ... }` 合并语义）。
- 无 `for` 的 `impl Point { ... }` 保持固有方法块语义不变。
- 冲突检测：同一 `(Type, Trait, 方法名)` 重复报错；显式 impl 与结构匹配同时满足时以显式为准
  （用于消歧/关联类型）。
- 结构匹配仍可自动满足「无关联类型、无静态方法」的接口；显式 impl 用于需要 `type` 绑定的场合。

验收：

- `p.parse(s)` / `Point::parse(s)` / 泛型 `[T: FromStr]` 静态分派；
- 重复 impl 报错；`impl Trait for Type` 与结构匹配共存。

## T4：关联类型

```ayanami
interface FromStr {
    type Err;
    fn parse(ref str) -> Result[Self, Self::Err];
}

impl FromStr for Point {
    type Err = ParseError;
    fn parse(ref str) -> Result[Point, ParseError] { ... }
}
```

语义：

- impl 中 `type Err = ParseError;` 绑定；泛型代码中投影 `T::Err`（约束 `T: FromStr` 时）。
- 特化时把 `T::Err` 替换为绑定的具体类型；`Result[Self, Self::Err]` 中的 `Self` 同 T1 消解。
- 对象安全规则（动态分派）：`Self` 只能出现在方法签名；关联类型必须已绑定且无泛型参数，
  否则给出「接口不对象安全」诊断。
- HIR：`HirType::Assoc { base, name }`；`InterfaceReg` 增加 `assoc_types: Vec<Symbol>`；
  实现表 `type_assoc: HashMap<(Symbol, Symbol), HirType>`；解析复用 `::` 类型限定语法。
- 错误：关联类型未绑定、约束缺失、impl 缺失 → 明确诊断。

验收：

- `FromStr` + `Result` 组合；`fn parse_all[T: FromStr](...)` 内使用 `T::Err`；
- vtable 场景编译通过或给出对象安全诊断。

## T5：返回类型驱动推断（可选）

- `int::parse(s)` 已由类型限定确定返回类型（T2）；`parse[int](s)` 显式实参已可用。
- `let x: T = parse(s)` 需要双向类型检查（期望类型向下传递）。建议**暂缓**：先文档化
  「显式类型限定 / 显式泛型实参」，收集实际痛点后再定。
- `Into`：`s.into()` 依赖返回类型推断，同样先走 `s.into[float]()` 显式形式。

## 阶段与排期

| 阶段 | 内容 | 依赖 | 规模 |
|---|---|---|---|
| T1 | `Self` 类型 | — | 小 |
| T2 | 关联函数 + `T::method` | T1 | 中 |
| T3 | 显式 impl | T2 | 中 |
| T4 | 关联类型 | T3 | 中 |
| T5 | 返回类型驱动推断 | T4 | 大（可选） |

## 影响文件

- parser：`types.rs`（`Self` 已有）、`enum_iface.rs`（接口方法/`type` 成员）、表达式 `::`
  调用、`impls.rs`（`impl for`）。
- HIR：`item.rs`（`HirInterfaceMethod`、关联类型）、`lower/mod.rs`（`InterfaceReg`）、
  `body/iface_match.rs`（Self/静态方法匹配）、`body/generic_specialize.rs`（约束与投影替换）、
  `helpers/substitute.rs`（`Self`/`Assoc` 替换）。
- MIR/LIR：静态调用节点已存在，无需新节点；vtable 条目扩展关联类型绑定。
- 文档/测试：README 接口章节、`example/test_typeclass*.aya`。

## 风险

- `Self`/关联类型与结构匹配的交互复杂：以显式 impl 为关联类型的唯一入口，结构匹配仅支持
  `Self` 子集，避免组合爆炸。
- `T::method` 与命名空间 `ns.fn` 的语法歧义：按「已声明类型优先」解析并给诊断。
- 泛型体内静态调用推迟解析：需要待解析节点 + 特化重放，注意与现有 `pending_stmts` 机制配合。

## 实现状态

- 设计：本文件（2026-10，#84 ① 提案）。
- 实现：未开始；T1–T4 已建 bd 任务排期。
