# Ayanami 编译器 — AI 协作指南

Ayanami 是一门自带 LLVM 后端的编译型语言（单二进制分发，无需系统预装 LLVM）。
编译器用 Rust 2024 编写，入口管线：源码 → Lexer → Parser → HIR → MIR → LIR → LLVM IR → 可执行文件。

**先读本文件再动手。** 找代码用 `SYMBOLS.md` + `rg`，不要全仓库盲读。

## 常用命令（均已实测）

| 目的 | 命令 |
|---|---|
| 编译检查 | `cargo check --all-targets` |
| 单元测试 | `cargo test`（仅 3 个单测，主要靠 example 端到端验证） |
| 前端检查 .aya | `cargo run -q -- check example/test_struct.aya`（输出 `check passed`） |
| 构建并运行 .aya | `cargo run -q -- run example/test_struct.aya`（产物在 `build/`） |
| 打包 .lcl | `cargo run -q -- package example/test_struct.aya` |
| 重新生成符号地图 | `./scripts/gen_symbols.sh` |
| 校验符号地图是否过期 | `./scripts/gen_symbols.sh --check` |
| 零告警校验 | `./scripts/check_warnings.sh` |
| 构建发布产物（tar + vsix） | `./scripts/package_release.sh` |
| 重新生成解析器 | `./gen_parser.sh`（会先编译 `asuka/` 子仓，再拆分生成物） |
| 校验文件行数 | `./scripts/check_file_sizes.sh`（默认上限 300 行） |
| 刷新模块依赖图 | `python3 scripts/gen_module_graph.py` |
| 安装版 CLI | `cd install && ./ayanami run ../example/test_struct.aya` |
| 初始化子仓 | `git submodule update --init --recursive`（`std/`、`asuka/`、`book/`） |

- `install/` 是发布包：`ayanami`、bundled `llc`、`libLLVM.so.21.1`、`runtime.c`、预编译 `std/`。
- **零告警要求**：`cargo check --all-targets` 不得输出任何 warning（生成代码 `src/generated/`
  已在模块级关闭；`asuka/` 子仓同样保持零告警）。校验：`./scripts/check_warnings.sh`，
  已并入 `./scripts/check_all.sh`。
- 编译 `.aya` 需要 `llc` 与 `gcc`：`cargo run` 用系统 `llc`（已装 `/usr/bin/llc`），安装版用同目录 bundled `llc`；`opt` 可选，用于 `-O2` 中端优化（`AYANAMI_OPT=0` 关闭）；bundled llc 场景只用同目录 `opt`，避免与系统 opt 版本不一致。

## 编译管线与入口

| 阶段 | 位置 | 关键入口 |
|---|---|---|
| CLI | `src/main.rs` | `cmd_check` / `cmd_build` / `cmd_run` 等 |
| 词法 | `src/lexer/` | `Lexer`（`lexer.rs`） |
| 语法 | `src/parser/` | `parse_source`（`mod.rs`）；Asuka 桥接 `gen_bridge.rs`；文法 `ayanami.grammar` |
| HIR 降级 | `src/hir/lower/mod.rs` | `lower_program` |
| HIR→MIR | `src/hir/lower/to_mir.rs` | `HirNode::lower_to_mir`（对全部 HIR 节点实现） |
| MIR | `src/mir/lower.rs` | `lower_program`；内存策略 `mir/mem/`（`MemStrategy`）；借用检查 `mir/borrow.rs` |
| LIR | `src/lir/lower.rs` | `lower_program`；LLVM 发射 `src/lir/emit.rs` `emit_program` |
| 驱动 | `src/driver/` | `ir_to_object`、`objects_to_exe` |
| 管线编排 | `src/compiler/mod.rs` | `CompilerPipeline`、`compile_source`、`check_source` |
| 导入解析 | `src/compiler/import.rs` | `resolve_import_path`、`find_std_dir` |
| 包格式 | `src/package/` | `.lcl` 序列化/反序列化 |
| 格式化 | `src/formatter.rs` | |
| 字符串驻留 | `src/intern/` | `Symbol(u32)` |

## 关键陷阱

1. **`src/generated/ayanami_parser.rs` 是生成产物，禁止手改。** 改语法必须改根目录 `ayanami.grammar`，然后跑 `./gen_parser.sh`。
2. 解析有两条路径：Asuka 生成解析器 + 手写回退（`parser.rs`，桥接 `gen_bridge.rs`）。改文法时注意回退路径仍然可用。
3. `asuka/` 与 `std/`（[Ayanami-std](https://github.com/ayanami1ei/Ayanami-std)）都是 **git 子仓**：在子仓内修改后要先在子仓提交并推送，主仓再提交新的子仓指针。`gen_parser.sh` 从 asuka 子仓构建；`std/` 源码在子仓 `src/`，主仓只更新指针。
4. `SYMBOLS.md`、`src/generated/`、`target/`、`build/` 都是产物或生成物，不要整读；`.lcl` 是二进制包，不要读。
5. `example/*.aya`（26 个）是端到端测试的主要手段；新特性至少配一个 example 用例。
6. `std/` 交付预编译 `.lcl`，源码在子仓 `std/src/**/*.aya`；根目录 `std/*.aya` 是指向 `src/` 的符号链接。构建/安装 `.lcl` 用子仓 `scripts/build.sh`（开发态安装到 `target/debug/std`）。

## 定位代码（省 token 的关键）

1. 先查符号地图：`rg "关键词" SYMBOLS.md`
   - 每行格式为 `路径:行号: 签名`，一次检索即可定位；不要整读 `SYMBOLS.md`（约 100KB）。
   - 覆盖 Rust 源码 + `std/`、`example/` 的 `.aya`，不含 `src/generated/`。
2. 再精读目标文件的相邻代码：`rg -n -C 5 "符号名" <文件>`。
3. 改完代码若增删了定义，跑 `./scripts/gen_symbols.sh` 更新索引，保持 `--check` 通过。
4. 大范围探索先派子代理，只把结论带回主会话。

## 代码约定

- 错误处理统一用 `src/error.rs` 的 `Error`（`thiserror` 派生，按阶段分变体）与 `crate::error::Result<T>`；**新代码禁止 `Result<_, String>`**。生成解析器（`src/generated/`）内部仍是 `String`，在边界转换。
- 借用检查为 NLL 风格（`src/mir/borrow/`：CFG + liveness 驱动 loan 活性）：引用可存局部变量、最后一次使用后失效；可返回（单引用参数生命周期省略）；不可存入字段/数组。
- IR 节点是 struct + trait object（`HirNode` / `MirNode`），字面量/节点转换走 `lower_to_mir`。
- 新增语言特性时同步整条链路：`ayanami.grammar` → AST → HIR → MIR → LIR → `emit` → example 用例。
- 注释中英混排，跟随所在文件的既有风格。
- 批量代码生成交给本地模型（见全局 `~/.config/opencode/AGENTS.md` 的 local-coder 政策）。

## 文件组织

- **单个 `.rs` 文件不超过 300 行**（目标 150~250），按职责拆到子目录 + `mod.rs`；校验：`./scripts/check_file_sizes.sh`。
- 类型定义留在 `mod.rs`（子模块才能访问私有字段）；`impl` 块可放子模块，跨模块调用的方法标 `pub(crate)` / `pub(super)`。
- 目录入口用 `mod` + `pub use` 保持对外路径不变。
- `src/generated/ayanami_parser/` 由 `./gen_parser.sh` 生成（asuka 输出 + `scripts/split_generated_parser.py` 拆分），禁止手改；只改根目录 `ayanami.grammar` 后重新生成。
- **改动代码后必须更新符号地图**：跑 `./scripts/gen_symbols.sh`（会刷新 `SYMBOLS.md` 的文件与行号），提交前 `./scripts/gen_symbols.sh --check` 必须通过。
- `std/`、`asuka/`、`book/` 是 git 子仓（见「关键陷阱」）；子仓内改动先在子仓提交推送，再更新主仓指针。

## 提交前检查

每完成一段必须通过检查并推送（`git push origin rust`）：

```bash
./scripts/check_all.sh        # 版本号 + 文件行数 + 符号地图 + 零告警 + 语言回归 + IR 快照
./scripts/regression.sh       # 单独跑正/负回归（check_all 已包含；AYANAMI_SKIP_REGRESSION=1 可跳过）
cargo check --all-targets
cargo test
```

## 分支约定

- `rust`：**集成分支**，功能分支合入这里（跟踪 `origin/rust`）。
- **每个新功能/子系统开一个新分支**：从 `rust` 拉出主题分支（如 `std-constructors`、`span-diagnostics`），
  在分支上完成、验证并推送；完成后合回 `rust`（优先 `--ff-only`）并删除该分支。
  `rust` 上只保留合并提交与紧急小修，不在 `rust` 上直接开发新功能。
- `release`：**发布分支**，从发布点拉出，只做发布修复与版本合并；发布用标签 `vX.Y.Z` 标记。
- `origin/master`、`origin/runtime`、`rust-old` 是旧实现/历史分支，不要在其上开发。
- 工作节奏：**每完成一个可验证的阶段就提交并推送**（`git push origin rust`）。
- `std/` 子仓改动直接在 [Ayanami-std](https://github.com/ayanami1ei/Ayanami-std) 的 `main` 上小步提交推送；主仓只更新子模块指针，不为 std 单独开主仓分支。

## 版本号同步（必须一起改）

权威版本号是 `Cargo.toml` 的 `version`，不得低于最新发布标签 `vX.Y.Z`；发布时二者必须一致。改版本时以下位置同步：

| 位置 | 说明 |
|---|---|
| `Cargo.toml` | 权威版本号 |
| `Cargo.lock` | 跑 `cargo check` 自动更新 |
| `vscode-ayanami/package.json` | VSCode 插件版本 |
| `std/ayanami.toml` | 标准库包版本（子仓 Ayanami-std；先在子仓提交，再更新主仓指针） |
| `README.md` | 安装产物文件名、ayanami.toml 示例 |

- 编译器源码禁止硬编码版本号：已统一用 `env!("CARGO_PKG_VERSION")` 自动跟随（`src/compiler/build.rs`、`src/main.rs` 的 `new` 模板）。
- 校验：`./scripts/check_version.sh`（版本漂移时报错退出），发布前必跑。
- 发布流程：改版本 → `cargo check` → `./scripts/check_version.sh` → `./scripts/package_release.sh`（构建 tar + vsix）→ 提交 → 打 `vX.Y.Z` 标签 → 推送分支和标签。

## 文档先行

新功能/子系统必须**先写文档再动代码**：

1. 设计文档写到 `docs/`（如 `docs/annotations.md`），含目标、语法/接口、语义、阶段、验收标准。
2. 概念与代码的映射更新 `docs/knowledge-graph.md`；模块依赖变化后跑 `python3 scripts/gen_module_graph.py`。
3. 实现落地后回到设计文档更新「实现状态」。

## 语言与文档索引

- 知识图谱：`docs/knowledge-graph.md`（概念→代码地图）、`docs/module-graph.md`（生成物）。
- 标注系统设计（标注式编程 A0–A5）：`docs/annotations.md`。
- 语言语法、类型系统、所有权（默认移动 + `ref`/`ref mut`）、泛型、枚举布局、操作符重载：见 `README.md`。
- 标准库 API：[Ayanami-std](https://github.com/ayanami1ei/Ayanami-std) 子仓的 `README.md` 与 `std/**/*.aya`。
- VSCode 插件：`vscode-ayanami/README.md`。
- 快速事实：参数顺序是 `类型 名称`；默认所有权（非 Copy 值移动），`[T]`/`[T; n]` 为拥有堆数组，`self` 消费 / `ref self` 借用 / `ref mut self` 可变借用（原语用 `self`）；无 `unique`/`shared`/`weak`/GC；泛型单态化；枚举字段 `_tag` / `_data_V`；操作符映射到 `add` / `eq` / `index` 等方法。

<!-- BEGIN BEADS INTEGRATION v:1 profile:full hash:bacef91e -->
## Issue Tracking with bd (beads)

**IMPORTANT**: This project uses **bd (beads)** for ALL issue tracking. Do NOT use markdown TODOs, task lists, or other tracking methods.

### Why bd?

- Dependency-aware: Track blockers and relationships between issues
- Git-friendly: Dolt-powered version control with native sync
- Agent-optimized: JSON output, ready work detection, discovered-from links
- Prevents duplicate tracking systems and confusion

### Quick Start

**Check for ready work:**

```bash
bd ready --json
```

**Create new issues:**

```bash
bd create "Issue title" --description="Detailed context" -t bug|feature|task -p 0-4 --json
bd create "Issue title" --description="What this issue is about" -p 1 --deps discovered-from:bd-123 --json
```

**Claim and update:**

```bash
bd update <id> --claim --json
bd update bd-42 --priority 1 --json
```

**Complete work:**

```bash
bd close bd-42 --reason "Completed" --json
```

### Issue Types

- `bug` - Something broken
- `feature` - New functionality
- `task` - Work item (tests, docs, refactoring)
- `epic` - Large feature with subtasks
- `chore` - Maintenance (dependencies, tooling)

### Priorities

- `0` - Critical (security, data loss, broken builds)
- `1` - High (major features, important bugs)
- `2` - Medium (default, nice-to-have)
- `3` - Low (polish, optimization)
- `4` - Backlog (future ideas)

### Workflow for AI Agents

1. **Check ready work**: `bd ready` shows unblocked issues
2. **Claim your task atomically**: `bd update <id> --claim`
3. **Work on it**: Implement, test, document
4. **Discover new work?** Create linked issue:
   - `bd create "Found bug" --description="Details about what was found" -p 1 --deps discovered-from:<parent-id>`
5. **Complete**: `bd close <id> --reason "Done"`

### Quality
- Use `--acceptance` and `--design` fields when creating issues
- Use `--validate` to check description completeness

### Lifecycle
- `bd defer <id>` / `bd supersede <id>` for issue management
- `bd stale` / `bd orphans` / `bd lint` for hygiene
- `bd human <id>` to flag for human decisions
- `bd formula list` / `bd mol pour <name>` for structured workflows

### Sync

bd stores issue history in Dolt:

- Each write auto-commits to Dolt history
- Use `bd dolt push`/`bd dolt pull` for remote sync
- Do not treat `.beads/issues.jsonl` as the sync protocol

**Architecture in one line:** issues live in a local Dolt DB; sync uses `refs/dolt/data` on your git remote; `.beads/issues.jsonl` is a passive export. See https://github.com/gastownhall/beads/blob/main/docs/core-concepts/sync-concepts.md for details and anti-patterns.

### Important Rules

- ✅ Use bd for ALL task tracking
- ✅ Always use `--json` flag for programmatic use
- ✅ Link discovered work with `discovered-from` dependencies
- ✅ Check `bd ready` before asking "what should I work on?"
- ❌ Do NOT create markdown TODO lists
- ❌ Do NOT use external issue trackers
- ❌ Do NOT duplicate tracking systems

For more details, see README.md and https://github.com/gastownhall/beads/blob/main/docs/getting-started/quickstart.md.

## Agent Context Profiles

The managed Beads block is task-tracking guidance, not permission to override repository, user, or orchestrator instructions.

- **Conservative (default)**: Use `bd` for task tracking. Do not run git commits, git pushes, or Dolt remote sync unless explicitly asked. At handoff, report changed files, validation, and suggested next commands.
- **Minimal**: Keep tool instruction files as pointers to `bd prime`; use the same conservative git policy unless active instructions say otherwise.
- **Team-maintainer**: Only when the repository explicitly opts in, agents may close beads, run quality gates, commit, and push as part of session close. A current "do not commit" or "do not push" instruction still wins.

## Session Completion

This protocol applies when ending a Beads implementation workflow. It is subordinate to explicit user, repository, and orchestrator instructions.

1. **File issues for remaining work** - Create beads for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **Handle git/sync by active profile**:
   ```bash
   # Conservative/minimal/default: report status and proposed commands; wait for approval.
   git status

   # Team-maintainer opt-in only, unless current instructions forbid it:
   git pull --rebase
   bd dolt push
   git push
   git status
   ```
5. **Hand off** - Summarize changes, validation, issue status, and any blocked sync/commit/push step

**Critical rules:**
- Explicit user or orchestrator instructions override this Beads block.
- Do not commit or push without clear authority from the active profile or the current user request.
- If a required sync or push is blocked, stop and report the exact command and error.

<!-- END BEADS INTEGRATION -->
