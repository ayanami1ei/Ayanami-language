# 泛型值表示与单态化（selfhost）

> 状态：设计（2026-10-20）。对应 bd `Ayanami-language-3tq`（及关联 `us0` Ptr[T]）。
> 本文定义 selfhost（stage-1）如何在**无 MIR/无类型标注发射器**的架构下支持
> 泛型参数 `T` 的跨函数值表示，并给出分阶段落地与验收标准。

## 1. 现状与问题

stage-1 管线是 AST → LLVM（无 HIR/MIR 类型下降）。泛型**完全擦除**：

| 场景 | 现状 | 后果 |
|---|---|---|
| `T` 槽（字段/元素/局部） | 8 字节 i64；指针类值 `ptrtoint` 存入 | 结构体/枚举值存入可读回为 i64（丢失"是指针"信息） |
| `T` 为 `String` | `store_slot_i64` 只存 `v.a`（ptr），长度丢失 | **非法 IR**：`store i64 @.str.0` |
| `T` 元素（`[T]` / `ArrayList[T]`） | 元素步长 8（`elem_llvm_m` 对泛型名归 i64） | String 元素（16 字节）放不下 |
| `T` 返回（`fn f[T](...) -> T`） | `ret_kind` 归 0（i64） | 调用点得到 i64，无法 `.field`/方法调用 |
| `Ptr[T].get() -> T` | 同上 | 无法解引用使用 |

具体失败例（diff 未对齐）：

- `test_constructors`：`ArrayList[String].push("hi")` → `store i64 @.str.0`（llc 失败）
- `test_generic_nested`：`fn inner[T](ref Box[T]) -> T` → 调用点 `inner(ref o).v` 取 0
- `test_generic_nested_arg` / `test_generic_struct_infer` / `test_usage_infer` /
  `test_collection_any` / `test_aggregate_array`：容器元素为 String 或泛型返回
- `test_ptr_tree` / `test_ptr_match` / `test_rawptr`：`Ptr[T]` 的 `.get()/.new()`
- `test_iter_infer` 等部分泛型推断例

**根因**：发射器在**每个表达式节点**上没有具体类型信息；同一函数/结构体只编译一次，
而 `T` 的布局依实例化而变。

## 2. 目标与非目标

**目标（本设计范围）**
1. 泛型容器（`ArrayList[T]`/`LinkedList[T]`/`Arena[T]` 等）能存 `String` 与结构体值；
2. 泛型函数可返回 `T` 并在调用点按**具体类型**使用（字段/方法/运算）；
3. `Ptr[T]` 可作为预编译库或内置实现；
4. 不破坏固定点（stage-2 == stage-3），不改变非泛型代码的 ABI 与输出。

**非目标**
- 完整的 trait 约束求解 / 高阶泛型 / GAT 等；
- 与 rust 侧 MIR 级单态化符号**逐字一致**（只需行为一致）；
- 改变现有 i64 槽内存模型（`%struct.X = { i64, ... }`）——单态化在其之上扩展。

## 3. 候选方案

### A. 按需单态化（推荐）
收集**显式实例化**（`ArrayList::new[String]()`、类型位置 `ArrayList[String]`、`Ptr[Tree]`），
为每个（函数/结构体, 类型实参元组）生成**独立特化**：

- 符号：`aya.<name>.__m<k>`（k = 实例编号，按出现顺序稳定）；
- 结构体：`%struct.<name>.__m<k>`，字段类型用**替换后**的类型（`T`→`String` 时字段占 2 槽）；
- 函数：参数/返回/局部按替换后的类型发射（`T`→`Box[int]` 时按结构体 ptr）；
- 调用点：按类型实参（显式或从实参推断）选择特化；无实例信息时回退擦除版本。

优点：语义正确、与 rust 侧模型一致、元素布局自然 16 字节；
缺点：需要**实例收集 + 类型替换 + 按实例重发射**（架构性，最大工作量）。

### B. 统一装箱 + 运行时类型标记
所有 `T` 值为 16 字节单元 `{值或指针, 类型标记}`，读取时按标记解包。
优点：无需单态化；缺点：**读取侧仍需具体类型**才能静态选择解包方式（方法派发可走
vtable，但 `"str" + x`、`x.field` 等静态语义无法还原）；且改变所有泛型 ABI，回归面大。
**不推荐**。

### C. 检查器标注具体类型 + 发射器按标注
检查器已有表达式类型字符串，但**不做替换**（`b.index(0)` 的类型仍是 `T`）；即使补替换，
结构体布局仍是全局唯一（`ArrayList` 只有一个），**元素大小无法按实例变化**。
**单独不可行**，但可作为 A 的辅助（为调用点推断类型实参）。

## 4. 方案 A 的分阶段落地

### 阶段 0：类型实参贯通（已完成部分）
- 解析器保留泛型调用的类型实参（`FnCall.tys`，`bits::reinterpret[T]` 已用）；
- 类型位置保留 `Generic` 节点的实参链（`ArrayList[String]`）。

### 阶段 1：实例收集（只收集，不改发射）
- 扫描 AST：函数调用/方法调用的类型实参、结构体字面量/类型标注中的 `Name[Arg...]`；
- 推断：调用点无显式实参时，用**实参的具体类型**（`String`/结构体名/枚举名/int/float）
  匹配形参中的泛型位置（首个匹配即可，歧义时回退擦除）；
- 产出 `inst(kind, name, args) -> id` 表（去重、顺序稳定）。

### 阶段 2：结构体特化
- `fill_struct` 支持**替换表**：字段类型 `T` 按实参替换后走正常字段类型判定
  （String→fk1 两槽、结构体→fk2、枚举→fk4、原语→fk0/fk5）；
- 类型名 `%struct.<name>.__m<k>`；`find_field_idx`/`fslot_at` 按特化索引；
- 字面量/字段访问/字段赋值的类型名统一经实例映射。

### 阶段 3：函数特化
- `add_fn_sig`/`emit_fn_named` 接受替换表；参数/返回种类按替换后的类型；
- 泛型参数 `T` 的**局部/字段读取**按替换后的类型发射（结构体 ptr / String 双槽 / i64）；
- 调用点按实例选择符号；未特化的调用保持擦除版本（兼容）。

### 阶段 4：内置与库
- `Ptr[T]`：`new`（分配单元存值）、`get/get_mut`（按替换后的 T 装载）、`as_ptr`；
- 泛型容器的 `push/index/pop` 等方法经特化获得正确元素步长。

### 阶段 2/3 实施细则（2026-10-23 定稿，供下轮实施）

1. **特化 worklist**：`spec_nodes/prefixes/params/args/names`；调用点登记（去重按 `(node, arg_ty)`），
   主发射循环后按序发射，循环条件用动态 `spec_count`（处理传递闭包）。
2. **实例符号**：`fn_sym_ord(full, ord)` + `.__i<k>`（k = worklist 下标）；定义端与调用端同式，
   定义端以 `cur_inst_ord` 参与。
3. **替换上下文**：`cur_inst_param`（类型参数名）/`cur_inst_arg_ty`（实参类型节点）/
   `cur_inst_name`（实例名）/`cur_inst_src`（源结构体下标）。`param_kind`/`ret_kind`/`ret_info`
   遇到参数名时改用实参类型。
4. **接收者实例**：局部 etype = 实例下标（构造调用返回值 → `inst_struct_name(src, arg)` 查实例名）；
   `struct_of_local`/`obj.b` 得到实例名；方法查找剥离 `__m<k>` 得源类型，`find_method_id(src, name)`。
5. **结构体字面量**：特化上下文中 `texts == 源结构体名` 时改用实例（字段表/槽位）。
6. **`self` 的 etype**：impl prologue 在特化上下文里记录实例下标而非源下标。
7. **验收顺序**：test_constructors（ArrayList/LinkedList 存 String）→ test_usage_infer /
   test_collection_any / test_aggregate_array → 泛型返回（test_generic_nested 等）。
8. **每步门槛**：`selfcompile` + 全量 diff + `check_all`；任一失败即回退该步。

## 5. ABI 与命名约定

| 项 | 约定 |
|---|---|
| 特化符号 | `aya.<full>.__m<k>`；结构体 `%struct.<name>.__m<k>`；k 为实例表下标 |
| 实例键 | `(name, arg1, arg2, ...)`，arg 用基础类型名（`String`/`int`/`Box`/`Box[int]` 展开） |
| 回退 | 无法确定实例时用现有擦除符号（行为保持现状，不引入新错误） |
| 固定点 | 编译器自身不使用泛型容器存 String；特化只影响被实例化的用户代码 |

## 6. 验收标准

1. 下列 example 的 diff 转为 SAME（stage-0 vs stage-1）：
   `test_constructors`、`test_generic_nested`、`test_generic_nested_arg`、
   `test_generic_struct_infer`、`test_generic_struct_lit`、`test_nested_generic_type`、
   `test_usage_infer`、`test_collection_any`、`test_aggregate_array`、
   `test_iter_infer`、`test_ptr_tree`、`test_ptr_match`、`test_rawptr`（分批验收）；
2. **固定点保持**（`./scripts/selfhost.sh selfcompile`，stage-2 == stage-3）；
3. `check_all` 全过（含零告警、文件行数、符号地图、IR 快照、回归）；
4. 非泛型代码的 IR 不变化（快照对比只允许实例化相关 diff）。

## 7. 风险与缓解

| 风险 | 缓解 |
|---|---|
| 实例爆炸 / 递归实例化 | 实例表上限 + 递归深度上限；超限回退擦除 |
| 推断歧义（`f[T](T, T)` 两实参不同型） | 仅首个匹配；歧义回退 |
| 发射器状态（表容量、符号冲突） | 实例符号独立命名空间；容量检查沿用现有上限 |
| 固定点回归 | 每阶段独立提交并跑 selfcompile；失败即回退该阶段 |
| 与 rust 侧诊断差异 | 特化不改变 check 诊断（只影响发射），diff 只应减少 |

## 8. 实现状态

- [x] 阶段 0：类型实参保留（`FnCall.tys`）
- [x] 阶段 1：实例收集（`collect_instances`：声明类节点的 `tys` 类型标注 + `FnCall.tys` 显式泛型调用；
  泛型参数实参跳过；`inst_keys` 去重表。注：表达式节点的 `tys` 存操作符/模式标签，不可扫描）
- [~] 阶段 2：结构体特化——**实例布局建表已实现**（
  `fill_struct_inst`：字段类型按类型参数→实参替换，数组元素同理；从显式泛型调用的返回类型
  派生结构体实例，如 `ArrayList[String]`；实例名 `Name__m<k>`）。发射侧接入（局部实例类型、
  字段访问、方法特化）待做
- [~] 阶段 3：函数特化——**显式泛型调用已接入**（
  `spec_register` worklist（按 (FnDecl, 实参类型) 去重）+ `emit_specializations` 动态发射；
  实例符号 `aya.<full>.__i<k>`；`param_kind`/`ret_kind`/`ret_info` 按替换上下文取实参类型；
  调用点返回类型/实参 ABI 同步替换。校验：`id[T]` + `id[int](7)` 生成 `aya.id.__i0` 并与 stage-0 一致）。
  余下：方法特化（接收者实例）、结构体字面量实例化、`[T; n]` 元素类型
- [ ] 阶段 4：Ptr[T] 与容器方法
