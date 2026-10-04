# Ayanami 自举编译器（Selfhosting）设计文档

> 状态：**M0 搭建中**。本文档是自举工程的权威设计与路线图，先于代码。
> 分支：`selfhost`（从 `rust` 拉出；`rust` 仍是对端持续开发的主线，需要经常合并）。

## 1. 目标

用 **Ayanami 语言**重写 Ayanami 编译器（现有实现为 `src/**/*.rs`，约 23k 行 Rust），
使其最终能编译自身：

- **stage-0**：现有 Rust 编译器（模板与行为基准）。
- **stage-1**：Ayanami 写的编译器（本工程产物），由 stage-0 编译。
- **stage-2**：用 stage-1 编译 stage-1 自身，验证 `stage1 == stage2`（固定点）。

验收的核心口径：对 `example/`、`tests/` 下的用例，stage-1 与 stage-0 的
**诊断/退出码/运行输出**一致（诊断允许文案差异，位置与级别须一致）。

## 2. 硬约束（源语言现状）

stage-1 只能用当前 Ayanami 已有的能力（见 `book/` 与 `README.md`）。已知缺口与对策：

| 缺口 | 对策 |
|---|---|
| 无文件 IO（std io 只有控制台） | `selfhost/src/base/file.aya` 用 `extern "C"` 包 `fopen/fread/fwrite/fclose` |
| 无 argv / 环境变量 | `/proc/self/cmdline`（NUL 分隔）读取 argv；`extern "C" getenv` 读环境变量 |
| 无 HashMap/HashSet | 自建 `IntMap[V]`（u32 键开放寻址）与字符串驻留表；`base/map.aya` |
| 无闭包（lambda 不捕获） | 只用函数指针；上下文显式传参或放进 `ref mut` 的结构体方法 |
| 引用不可存字段/数组 | **索引式 IR**（`Vec<Node>` + `NodeId(int)`），与 Rust 版 `FnId/VarId/Symbol` 设计一致 |
| 无 trait object | HIR/MIR 节点用 **enum + 变体方法派发**（`enum HirNode { ... }`）替代 `Box<dyn>` |
| 无位运算/移位 | 用算术与枚举替代（词法/哈希需要时用乘模实现） |
| 无 const/全局变量 | 常量放进 `Ctx` 字段或函数返回；配置经 `Config` 结构体传递 |
| 模式匹配只有枚举绑定 | 字符串/整数分派用 `if/elif` 链；枚举用 `match` |
| `for` 只能区间 | 遍历集合用 `while` + 索引；`ArrayList.iter(fn)` 仅用于无捕获回调 |
| 无 `String` 格式化 | `base/sb.aya`（StringBuf：按块拼接，避免 O(n²)） |
| 无 `std::process::Command` | `extern "C" system()` 调用 `llc`/`gcc`；路径来自环境变量 |

## 3. 目录结构

```
selfhost/
  README.md            # 快速上手（构建/测试命令）
  src/
    main.aya           # CLI：check/build/run/package 分派（读 /proc/self/cmdline）
    base/              # 编译器内部基础库（先于 std 缺失部分）
      file.aya         # 文件读写（extern C）
      sb.aya           # StringBuf
      map.aya          # IntMap / StrMap（开放寻址）
      intern.aya       # Symbol 驻留（u32）
      log.aya          # 诊断输出（rustc 风格）
      sys.aya          # argv/env/system
    lexer/             # token.aya / lexer.aya
    ast/               # 节点 enum、打印
    parser/            # 手写递归下降（对齐 rust 版回退解析器）
    hir/               # 降级、类型解析、接口、单态化
    mir/               # 降级、内存策略、借用检查
    lir/               # 三地址码 + LLVM IR 文本发射
    driver/            # llc/gcc 调用、.lcl 包（后期）
  tests/               # 每个模块的 .aya 自测（stage-0 运行，返回非 0 即失败）
  scripts/             # 本目录内辅助脚本（构建/差分测试）
```

## 4. 里程碑与验收

| 里程碑 | 内容 | 验收 |
|---|---|---|
| **M0** | 分支/知识库/同步脚本/骨架 | `selfhost/src/main.aya` 被 stage-0 编译并打印用法 |
| **M1** | base 库（file/sb/map/intern/sys/log） | `tests/base_test.aya` 全过：读写临时文件、Map 增删查、驻留 |
| **M2** | Lexer（完整 token 集） | 对 `example/*.aya` 的 token 流与 rust 版逐一比对（golden） |
| **M3** | Parser + AST 子集 | 能解析 `tests/parse/*.aya`；错误定位与 rust 版一致 |
| **M4** | HIR 子集 | 名称解析/类型检查覆盖子集用例 |
| **M5** | LIR + LLVM 文本 + driver | **端到端**：stage-1 编译 hello.aya 并运行，输出正确 |
| **M6** | 语言特性扩展 | struct/方法/枚举/match/泛型/接口/借用/标注逐步对齐 |
| **M7** | 自举 | `stage1` 编译自身；`stage1 == stage2`；`example/` 全过 |
| **M8** | 工具链/包 | `.lcl` 打包、`fmt`、CLI 对齐、发布产物 |

## 5. 测试策略

1. **单元自测**：`selfhost/tests/*_test.aya` 用 `assert 风格`（返回非 0 报错），由 stage-0 运行；
   脚本 `scripts/selfhost-test.sh` 编译并执行，失败即退出。
2. **差分测试**：M5 起对 `example/` 与 `tests/positive` 同跑 stage-0/stage-1，比较 stdout+退出码；
   脚本 `scripts/selfhost-diff.sh`。
3. **IR 快照**：M5 起对同一 .aya 比较两版的 LLVM IR 语义等价（先按文本 diff 报告，人工确认）。
4. **回归门禁**：在 `selfhost` 分支上不破坏 Rust 主线检查（`./scripts/check_all.sh` 仍须过），
   自举代码不进 Rust 编译单元。

## 6. 本地 AI 协作流程

- **知识库**：`tools/kb/` 把 `book/` 切成块并建索引（Ollama `nomic-embed-text`，不可用时退回 TF-IDF）。
  生成任何 Ayanami 代码前，先用 `kb.py search "<主题>"` 取相关章节，作为 `ask.sh --context` 的一部分。
- **强制规则**：所有交给本地模型的 Ayanami 代码任务，提示词必须包含
  「只允许使用 book 中记录、且经 stage-0 验证过的语法/API」与检索到的章节片段。
- **分工**：主代理负责接口/结构/验收用例/评审；本地模型按单文件规格生成实现（`ask.sh`）。
  单次委托一个文件、附完整接口与边界条件；两轮失败则由主代理直接完成并记录例外。
- **最小可验证**：每个模块先写接口与测试（stage-0 可运行的 .aya 测试），再让本地模型填实现。

## 7. 同步与合并（对端持续开发 rust）

`scripts/selfhost-sync.sh`：

1. `git fetch origin`，把 `origin/rust` 合并进 `selfhost`（`--ff-only` 失败则普通 merge）；
2. `git submodule update --remote book asuka`（教程频繁更新）；
3. 若 rust 有更新：`cargo build --release`，stage-0 使用本仓 `target/release/ayanami`；
4. 重建知识库索引。

约定：`selfhost/**`、`tools/kb/**` 为自举分支专属，不改动 `src/**`（Rust 模板）；
如确需 stage-0 改进，先在 `rust` 侧提（对端在维护），再合并过来。

## 8. 风险

- **语言缺口堆积**：M1 的 base 库是自举地基，接口一旦被大量代码依赖，返工代价高 → 先写测试再实现。
- **无 HashMap/文件 IO 的性能与正确性**：开放寻址 Map 需覆盖 tombstone/再哈希；用测试兜底。
- **Rust→Ayanami 语义差异**：迭代器链、trait 对象、闭包、`match` 守卫改写量大，属机械但量大 → 差分测试。
- **对端 rust 漂移**：频繁合并；冲突集中在 `docs/`、脚本与测试目录。
- **本地模型上限**：单文件规格 + 一shot 生成；复杂类型系统模块（借用检查/单态化）由主代理设计并复核。
