# 自举通用堆释放 / Drop 设计

> 状态：D0（设计）+ D1（首片，受控开关）  
> 目标分支：`selfhost` / `selfhost-drops`  
> 相关：`docs/rust-selfhost-diff.md`、`std/runtime.c`、`selfhost/src/lir/emit.aya`

## 1. 目标与边界

自举发射器目前只对 `fs.read_lines` 的读缓冲做显式释放；其余 `@malloc`
分配的 `String`、`ArrayList`、枚举/结构体载荷依赖进程退出回收。本设计定义
自举的通用 drop/作用域释放子系统，分阶段补齐，同时保持：

- stage-0 vs selfhost 的 178/178 输出、退出码、effect 警告逐字节一致；
- stage-2 == stage-3 自举定点；
- 与 stage-0 自动 drop 共存时无双重释放（运行时计数释放已幂等）。

非目标（D1）：完整的 Rust 式 drop glue、Drop trait、析构顺序/panic 安全；
这些留给后续阶段。

## 2. 所有权语义（与语言一致）

- 非 Copy 值默认移动；`ref`/`ref mut` 是借用，不转移所有权。
- `#[copy]` 结构体按值复制；`String`/`[T]`/`ArrayList`/枚举载荷拥有堆缓冲。
- 函数按值 `String` 形参拥有实参；返回 `String` 时所有权转移给调用者。
- 借用检查保证引用不逃逸；drop 只能在最后一个所有者处发生。

自举当前没有 MIR，发射器直接从 AST 生成 LLVM IR，因此 drop 分析必须
在发射期完成，并且要比 stage-0 的 MIR drop 插入更保守。

## 3. 运行时基础

- `std/runtime.c` 已有计数分配登记表：`__ayanami_unique_alloc` 登记指针，
  `__ayanami_unique_free` 对已释放指针幂等。
- 新增 `__ayanami_release_owned(void *ptr)`：D1 只释放已登记（计数）缓冲；
  计数指针走 `counted_take` + 递减 + free，未登记指针不 free（可能是字面量/
  全局/借用视图）。只应在确定是该缓冲最后所有者时调用。
- 字符串字面量/全局常量不在登记表内；D1 的保守分析不得释放其指针。

## 4. 阶段

### D0 设计（本文件）

定义所有权、插入点、运行时原语、验收标准。

### D1 首片：未使用按值 String 形参（受控）

- 解析期：函数解析完成后，在函数 AST 节点范围内做文本级形参使用检测；
  仅当形参名除 Param 节点外从未出现，标记 `Param.bb = 1`（未使用）。
  文本级近似只会有误报（把字段名/字符串字面量当作使用），不会漏报，
  因此对释放是保守的。
- 发射期：对 `param_kind == 1`（按值 String）、非 `self`、非隐藏 env 形参
  且标记未使用的形参，在函数所有返回路径（显式 return、尾表达式返回、
  默认返回）前发射 `__ayanami_release_owned(ptr)` 并置空槽位。
- D1 只处理**未使用**形参：不读取、不返回、不存储，所有权明确，D1 安全。
- 开关：`AYANAMI_SELFHOST_DROP=1` 时发射；默认关闭，先保住 178/178。
  后续 D2 通过 tracked allocator 和“拥有型局部”分析后默认开启。

### D2 拥有型局部与字段

- 跟踪 `String` 局部/结构体字段/枚举载荷的拥有来源，插入作用域/分支 drop。
- 处理 move、部分 move、重新赋值、循环内覆盖；借用中的值不释放。
- 字符串字面量/全局/借用视图标记为非拥有，不释放。
- 需要把自举发射器的 `@malloc` 统一到一个可登记/可检查的分配入口，避免
  释放全局或借用指针。

### D3 完整 drop glue

- 结构体/枚举/数组递归 drop；返回/break/continue/panic 路径；
- 与 stage-0 自动 drop 的幂等协作；性能与代码尺寸评估。

## 5. 验收

D1：

- 默认（开关关闭）：`./scripts/selfhost.sh diff` 178/178，selfcompile 定点，
  tokens/AST/errs/check/e2e 全绿，`check_all` 全过。
- 开关开启：受控探针中未使用按值 String 形参的发射 IR 含
  `@__ayanami_release_owned`；程序运行 exit 0；无双重释放。
- 不改变 stage-0 行为；运行时 `release_owned` 仅对计数指针递减 `live_allocs`，未登记指针不改变计数。

## 6. 实现状态

- [x] D0 设计
- [x] D1 解析期未使用形参标记
- [x] D1 发射期 return/tail/default drop（`AYANAMI_SELFHOST_DROP=1`）
- [x] D1 运行时 `__ayanami_release_owned`
- [ ] D2 拥有型局部/字段（后续）
- [ ] D3 完整 drop glue（后续）
