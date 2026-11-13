# Rust 主线 ↔ 自举分支差异对齐（2026-10-25）

> 快照：自举 `65780ed`（`selfhost`），Rust 主线 `133a5e4`（`origin/rust`），
> 共同祖先 `c6fc9f9`。本文是差异/路线图，不是自动同步结果；后续以 git 为准。

## 0. 一句话结论

自举分支在 **c6fc9f9 的 Rust 源 + 154 个 example** 上达到 **154/154**、
固定点 `1cda55de`、全部 golden 绿；Rust 主线此后前进了 **44 个提交**，
新增 async / 并发 / env 能力 / niche 枚举 / 增量缓存等，并把 std 从 `0bd1bfd`
推进到 `ad1e6db`（+19 提交）、example 从 154 扩到 **178**（+24）。
**直接 merge 不会自动得到 178 对齐**：需要先补 `#[extern]` 与 std 兼容，
再按特性分批移植。

## 1. 分支与规模

| 项 | 自举 `selfhost` | Rust `origin/rust` |
|---|---|---|
| HEAD | `65780ed` | `133a5e4` |
| 相对 c6fc9f9 | **+204** 提交 | **+44** 提交 |
| Rust `src/` 改动 | **0**（干净快照） | 94 文件，+1924/−349 |
| 全仓改动 | selfhost/、scripts、docs、IR 快照、std 指针 | 156 文件，+3894/−405 |
| std 子模块 | `0bd1bfd` | `ad1e6db`（base `0ec26cc`） |
| example | **154** | **178**（+24；`test_asm.aya` 修改） |
| 版本 | 0.6.6 | 0.6.6 |
| 当前验证 | 154/154、固定点 `1cda55de`、tokens/AST/errs/check/e2e 全绿 | 自带回归 |

自举分支自 c6fc9f9 起**没有改过 Rust `src/`**，所以 `src/` 的合并本身是干净的；
真正需要处理的是 **std 指针**、**example/tests**、**docs/scripts** 的交叉改动，
以及新特性在自举实现里的缺失。

## 2. Rust 侧新增（44 提交，按特性）

| 组 | 代表提交 | 内容 |
|---|---|---|
| async A1a/A1b | `1a3e60e`→`8f6dae1` | Future 协议/单线程 executor；`#[async]` AST 展开；`await` 单/多挂起点、float/bool/String 载荷、循环/分支 await 诊断 |
| 并发 std S1–S4 | std `15d16eb`→`7c33540` | Atomic/fence、Mutex/RwLock（futex）、Thread、Channel；纯汇编 runtime |
| env E1–E4 | `393cbe9`→`dd8d255` | `env::allocator`/`env::logger` 能力键、提供者作用域、块级覆盖、bound 传播、泛型字段持有分配器 |
| A5e 注解 | `9e21323`/`69ae955`/`8d117d8` | `#[sync(obj)] { ... }`、`#[sync(m, guard=d)]`、统一注解表与同包唯一性 |
| 内存/借用 | `af097c2`/`ba60bee`/`57c4b9b` | #211 条件表达式 use-after-move、禁止从借用移出（A）+ match ergonomics、match/if 分支结果移出 |
| 布局 | `a407f95`/`dcea463` | niche `Option[Ptr/ref/fn]` 8 字节；枚举 >16B 载荷装箱 |
| 引用 R3/R4 | `d7afe75`/`663cc73` | 自定义解析器（ID+注册表）、引用字段 ID 序列化 + 对象图重建 |
| 宏 | `3fb8e68`/`bf9a2fe` | #215 仅声明 String input 的宏视为可变参数；#217 函数宏 input 传实参原文；std TokenStream API |
| 增量 #35 | `201bfa4`/`6fad201` | 整程序增量缓存（默认开）、编译器指纹、std 内容入 key |
| 缺陷修复 | `600585e`→`7c2ac77` | #200–#217（asm 效应/clobber、赋值类型、闭包装箱、裸指针写、unsafe 闭包捕获、void 泛型、ref self 泛型 enum、本地函数重名等） |
| std | `ad1e6db` | 并发、allocator、TokenStream、asm 后端、`#[extern]` 语法迁移 |

## 3. std / example / test 差异

- **std**：`0bd1bfd..ad1e6db` 共 19 提交、61 文件、+6347/−462；新增
  `sync/{atomic,mutex,rwlock,thread,channel}`、`alloc` 接口、`meta/macros`
  TokenStream、asm runtime；`core/{string,panic}.aya` 等改为 `#[extern] fn ...;`。
- **example**：+24，分类：
  - async：`test_async_await`/`test_async_manual`/`test_async_sugar`
  - env：`test_env_{allocator,block,chain,custom,generic,pool}`
  - sync：`test_sync_block`
  - 布局：`test_niche_option`/`test_enum_boxed`
  - 引用：`test_ref_{deref,graph,registry,self_enum}`
  - 其他：`test_asm_effects`/`test_closure_unsafe`/`test_fn_shadow`/
    `test_match_{dot_variant,ref_binding}`/`test_rawptr_effects`/
    `test_return_in_while`/`test_void_generic`
  - 既有 `test_asm.aya` 修改。
- **tests**：`runtime_safety` 新增分支结果移出/ref graph/registry 等；
  `compile_fail` 新增 use-after-move 条件；`positive_exit.txt` +25 行。

## 4. 自举侧新增（Rust 分支没有）

- `selfhost/`：Ayanami 写的完整编译器（HIR→LIR，无 MIR），154/154。
- `scripts/selfhost*.sh`、`selfhost-annotate.py`：自举构建/差分/golden/标注工具。
- `docs/selfhost.md`、`generic-values.md`、`macros.md`、`reference-model.md`、
  `check-plugins.md`：自举里程碑、坑、泛型/引用/宏/插件实现。
- `selfhost/tests/**`：base、parser golden、typecheck、e2e。
- `tests/ir_snapshots/test_bounds.*`：自举侧更新过的快照（合并时保留/重生成）。
- std 指针 `0bd1bfd`（带 first/last/unwrap `#[alloc]` 修复）。

## 5. 对齐阻塞（已验证/可判定）

1. **`#[extern]` 语法**：**已修复（2026-10-25）**。std `core/string.aya`/`panic.aya`、
   `meta/macros.aya`/`mir.aya` 已迁移为 `#[extern] fn ...;`；自举 parser 支持关键字属性
   `#[extern]`（无体声明），emitter 用 bit0 保留 pure/noreturn 位。
2. **新 example 语法**：`#[async]`/`await` 已实现（2026-11-13，自举 **eager v1**：
   `__f_Fut` + `__async_run_f` + poll impl，`await g(args)` 展开为 `__async_run_g(args)`；
   stage-0 的 async 警告已复刻）。`#[sync(...)]` 已实现（2026-11-12）。
   **env 已实现（2026-10-25）**：`#[env::<cap>(provider)]` 函数/块作用域、
   `env::<cap>` 访问器、隐藏 env 形参 + 调用点注入、泛型 bound 推断。
3. **布局 N1/N2/N3**：`size_of` 已按 Rust 布局（niche 8B / >16B 装箱 / 顺序载荷）计算，
   `test_enum_boxed`/`test_niche_option` 均转正（Ptr/ref 载荷按 Ptr 值实例化与 match 绑定）；
   自举实际枚举表示仍为 tag+载荷槽（仅 `size_of` 报告 Rust 布局）。
4. **内存/借用模型**：A（禁止从借用移出）+ match ergonomics、#211 条件移动检查、
   分支结果移出；自举 checker/borrow 是另一套实现，接受/拒绝集合不同。
5. **TokenStream 宏**：std `meta/macros` 用 ABI v6 TokenStream；自举宏引擎是
   **源码级解释执行**（`docs/macros.md`），没有字节通道/宏插件 ABI。
6. **增量 #35**：Rust 主线整程序缓存；自举没有（不影响输出对齐，但影响工具链）。
7. **MIR 缺失**：`#[pass]` 写回、async 状态机（HIR 层）、逐节点 MIR 视图；
   自举的 `#[check]` 是函数级聚合只读视图（`docs/check-plugins.md`）。
8. **并发/asm/allocator std**：分配器接口、asm（early-clobber / ref 操作数取址）
   与 `std/runtime.c` 的 C 后端 futex 均已可用；`sync_block` 的 `#[sync]` 已实现。
   thread/channel 的 std 汇编 runtime 仍未接入自举链接。

## 6. 对齐路线

### A. 全量同步（独立分支，推荐用于追新）

`selfhost-rust-sync` ← `selfhost`，merge `origin/rust`（std 指针取 `ad1e6db`）：

1. **Phase 0**：`#[extern]` 属性（parse + 链接）+ 新 std 可编译；自举 CLI 能构建。
2. **Phase 1**：原 154 example 回归（新 stage-0/布局/借用变化）；更新 IR 快照。
3. **Phase 2**：env 能力、A5e `#[sync]` 块均已实现；niche/装箱枚举待做。
4. **Phase 3**：async A1a/A1b 已实现（自举为 eager v1；完整 Pending/状态机挂起待后续）。
5. **Phase 4**：并发/asm/TokenStream/allocator std；目标 178/178。

每一步都以 `selfcompile + diff + check_all` 为门槛，保持固定点。

### B. 选择性修复同步（保持 154 基线）

只 cherry-pick 与旧 std/旧特性兼容的修复（如 #212 ref self 泛型 enum、#217/#215
宏、部分内存修复），保持 std `0bd1bfd`、example 154；逐步验证，避免引入新语法。
适合“只修 bug、不追新特性”。

### C. 冻结

保持 c6fc9f9/0bd1bfd 与 154/154；Rust 主线继续前进，本文记录差异。
适合把自举作为稳定参考实现。

**建议**：先做 **B**（低风险修复，守住 154）；同时把 **A** 作为独立分支路线，
从 Phase 0（`#[extern]` + std）开始。不要在主 `selfhost` 分支直接 merge 全套新特性。

## 7. 当前自举基线（勿回退）

- example diff：**154/154**
- 固定点：stage-2 == stage-3，md5 `1cda55de`
- golden：tokens 154/154、AST 5/5、errs 8/8、check 29/29、e2e 19/19
- 自举源 effect 告警：0；`check_all` 全过

## 8. 同步实验分支结果（2026-11-13）

分支 `selfhost-rust-sync`（worktree `selfhost-sync`）：

- merge `origin/rust` `133a5e4`；merge 曾丢 5 个 async 源文件，已用 `9b09394` 恢复
- std `ad1e6db`、example **178**
- **旧 154/154 保持；新快照 177/178（99.4%）**
- 自举 golden：test、tokens **178/178**、AST 5/5、errs 8/8、check 29/29、e2e 19/19
- 固定点：stage-2 == stage-3，md5 `2a313cf0010f296303f5bafc191fdcfb`
- 已完成修复：`#[extern]`（关键字属性 + 无体声明 + emitter bit0）、asm `clobbers(rax)`/无输出 asm、
  点限定变体模式、换行缩进规则、`ref`/`ref mut` 的 `*r` 读写、裸指针形参 pointee 标记、
  **env E1/E2**（函数/块 provider 作用域、`env::<cap>`、隐藏 env 形参、调用点注入、
  bound 驱动的泛型推断）、**裸块语句 + Attributed checker/emitter**、
  **语句 match 的 Block 臂体发射**（此前被跳过；同时修复 struct 解构 `Ptr[T]` 绑定的
  解引用标记，`test_ptr_match` 转正）、**A5e `#[sync]` 块**（lock → guard → 块体 → unlock；
  return/break/continue 逆序解锁、guard 用 payload ref 本地；配套 asm `out(early reg)`、
  asm ref 操作数取址、`ref <field> as *T` 取址、间接调用 ref 实参取址、
  match 合并块 terminated 修正）、**async A1b eager v1**（每个 `#[async]` fn 生成
  `__f_Fut{params,_done}` + 构造器 + `__async_run_f`（原体；`await g(args)` → `__async_run_g(args)`）
  + `impl { poll }` 返回 `Poll::Ready(run(...))`；async 位 524288；生成 poll/run 的 effect 警告抑制；
  stage-0 的 async 警告（接口/首个 async fn 的 alloc，生成 span）已复刻）、
  **泛型枚举返回实例追踪**（`Poll[String]` 的 String 载荷 match）、**`fx_mask` 保留 async 位**、
  **closure_unsafe**（unsafe 块内裸指针闭包捕获：raw pointer 按值捕获并保留 pointee 标记；
  `Ptr.new(g)`/`q.get()` 的闭包 ftype 跨 Ptr 传播，`r()` 正确走间接调用）、
  **枚举布局 size_of**（N1 niche 8B / N2 >16B 装箱 / 顺序载荷：Option[Ptr/ref] 8、
  Option[int] 16、E 24、OptBig 8；`test_enum_boxed` 转正）与 **declare_local form 位掩码**
  （同名 match 绑定跨 form 复用存储，修复重复 alloca）、
  **多类型实参调用**（`HashMap::new[K, V]`：解析器把第 2/3 个类型实参存入 FnCall.cc/dd，
  发射器按 4 实参特化）、**嵌套泛型字段替换**（`Registry[T] { Arena[T] arena }`：
  实例字段按父实参替换并预建嵌套实例；字段表改为局部暂存后连续追加，避免递归建实例时
  覆盖父实例布局）、**String ref self**（`impl String` 的 self 一律按 `{ptr,len}` 引用传递，
  长度不再丢失）、**String 字典序比较**（`< > <= >=` 走 memcmp + 长度）、
  **数组/结构体指针与 null 比较**（比较前 ptrtoint）
- 因此转正：env×5（allocator/block/chain/custom/generic）、async_manual、
  ptr_match、sync_block、async_sugar、async_await、closure_unsafe、enum_boxed、
  niche_option
- R3/R4 功能已落地：`test_ref_registry` **SAME**（编译/运行/effect 警告顺序/退出码全部一致）；
  `test_ref_graph` 的图构建、序列化、往返重建与 effect 警告均与 stage-0 一致，但最终
  `__ayanami_live_allocs()` 因自举发射器不发堆释放而为 1（exit 10）。diff 仍按“完整输出一致”
  计为 DIFF，故唯一剩余项是 ref_graph 的 live 检查。
- 泛型 effect 警告按“被调特化先于调用特化”的后序回放，并按类型语义归并同类型的不同 AST
  节点（保留 Arena[Node]/Arena[GNode] 等不同实例的重复警告），与 stage-0 的单态化诊断顺序一致。
- niche_option 已对齐：size_of 8/8/16；`Ptr[Struct]` 载荷按 Ptr 值实例化
  （`val_marker` 保留 Ptr 标记、ctor 实例复用显式 `Option[Ptr[T]]`），
  `match Some(x)` 绑定 Ptr 值，`_data_V._i` 支持负标记载荷（Ptr/ref/struct/enum），
  `ref` 载荷不拥有对象
- 已知限制：自举 async 为 **eager 展开**（await 目标立即就绪；完整 Pending/状态机挂起待后续）；
  async 警告 parity 采用生成 span 的 v1 复刻；堆释放/drop 尚未发射（引用图用例的 live 检查失败）
- 环境备注：`scripts/selfhost.sh` 开发构建优先 `/usr/bin/llc`（安装包 bundled LLVM 可能与
  系统 libLLVM 版本不匹配，M68k 符号缺失时 llc 启动失败）
- 当前定点：stage-2 == stage-3（md5 `2a313cf0010f296303f5bafc191fdcfb`）
- 下一步：堆释放/drop（ref_graph 的 live 检查）；其余 177 例已与 stage-0 逐字节一致
- 复现：`AYANAMI_NO_INCREMENTAL=1 ./scripts/selfhost.sh diff`
  （`example/math_lib.lcl` 是本地未跟踪 fixture；缺失时 test_import 的 stage-0 会 exit 1）
- 主 `selfhost` 分支仍为 154/154，本实验分支未合并
