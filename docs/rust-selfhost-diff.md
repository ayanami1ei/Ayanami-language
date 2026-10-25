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
2. **新 example 语法**：`#[async]`/`await`（lexer 无 `Await` token）、
   `#[sync(...)] { ... }`；自举 parser/checker 未实现。
   **env 已实现（2026-10-25）**：`#[env::<cap>(provider)]` 函数/块作用域、
   `env::<cap>` 访问器、隐藏 env 形参 + 调用点注入、泛型 bound 推断。
3. **布局 N1/N2/N3**：niche 枚举 8 字节、>16B 载荷装箱；自举 emitter 仍是
   tag+载荷槽。会影响 `size_of`/`align_of`/`static_assert` 与 ABI 相关用例。
4. **内存/借用模型**：A（禁止从借用移出）+ match ergonomics、#211 条件移动检查、
   分支结果移出；自举 checker/borrow 是另一套实现，接受/拒绝集合不同。
5. **TokenStream 宏**：std `meta/macros` 用 ABI v6 TokenStream；自举宏引擎是
   **源码级解释执行**（`docs/macros.md`），没有字节通道/宏插件 ABI。
6. **增量 #35**：Rust 主线整程序缓存；自举没有（不影响输出对齐，但影响工具链）。
7. **MIR 缺失**：`#[pass]` 写回、async 状态机（HIR 层）、逐节点 MIR 视图；
   自举的 `#[check]` 是函数级聚合只读视图（`docs/check-plugins.md`）。
8. **并发/asm/allocator std**：分配器接口与 `src/runtime.c → std/runtime.c` 已可用
   （`test_env_*` 走通）；futex/thread/channel 的 std 汇编 runtime 尚未接入自举链接，
   `sync_block` 的 `#[sync]` 去糖也未实现。

## 6. 对齐路线

### A. 全量同步（独立分支，推荐用于追新）

`selfhost-rust-sync` ← `selfhost`，merge `origin/rust`（std 指针取 `ad1e6db`）：

1. **Phase 0**：`#[extern]` 属性（parse + 链接）+ 新 std 可编译；自举 CLI 能构建。
2. **Phase 1**：原 154 example 回归（新 stage-0/布局/借用变化）；更新 IR 快照。
3. **Phase 2**：env 能力（已实现）；A5e `#[sync]`、niche/装箱枚举待做。
4. **Phase 3**：async A1a/A1b（HIR 状态机）。
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

## 8. 同步实验分支结果（2026-10-25）

分支 `selfhost-rust-sync`（worktree `selfhost-sync`）：

- merge `origin/rust` `133a5e4`；merge 曾丢 5 个 async 源文件，已用 `9b09394` 恢复
- std `ad1e6db`、example **178**
- **旧 154/154 保持；新快照 170/178（95.5%）**
- 自举 golden：test、tokens **178/178**、AST 5/5、errs 8/8、check 29/29、e2e 19/19
- 固定点：stage-2 == stage-3，md5 `0b55d4041401e6abf39d63910dc5792e`
- 已完成修复：`#[extern]`（关键字属性 + 无体声明 + emitter bit0）、asm `clobbers(rax)`/无输出 asm、
  点限定变体模式、换行缩进规则、`ref`/`ref mut` 的 `*r` 读写、裸指针形参 pointee 标记、
  **env E1/E2**（函数/块 provider 作用域、`env::<cap>`、隐藏 env 形参、调用点注入、
  bound 驱动的泛型推断）、**裸块语句 + Attributed checker/emitter**、
  **语句 match 的 Block 臂体发射**（此前被跳过；同时修复 struct 解构 `Ptr[T]` 绑定的
  解引用标记，`test_ptr_match` 转正）
- 因此转正：env×5（allocator/block/chain/custom/generic）、async_manual（块臂体发射）、
  ptr_match（struct 绑定修复）
- 剩余 8：async_await、async_sugar、closure_unsafe、enum_boxed、niche_option、
  ref_graph、ref_registry、sync_block
- 下一步：sync_block（`#[sync]` 去糖 + std futex/asm runtime 接入）、async A1a/A1b、
  niche/boxed 布局、R3/R4 引用图、closure_unsafe
- 复现：`AYANAMI_NO_INCREMENTAL=1 ./scripts/selfhost.sh diff`
  （`example/math_lib.lcl` 是本地未跟踪 fixture；缺失时 test_import 的 stage-0 会 exit 1）
- 主 `selfhost` 分支仍为 154/154，本实验分支未合并
