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
| 重新生成解析器 | `./gen_parser.sh`（会先编译 `asuka/` 子仓，再拆分生成物） |
| 校验文件行数 | `./scripts/check_file_sizes.sh`（默认上限 300 行） |
| 安装版 CLI | `cd install && ./ayanami run ../example/test_struct.aya` |

- `install/` 是发布包：`ayanami`、bundled `llc`、`libLLVM.so.21.1`、`runtime.c`、预编译 `std/`。
- `cargo build` 有 60+ 条 warning 属常态，不是错误。
- 编译 `.aya` 需要 `llc` 与 `gcc`：`cargo run` 用系统 `llc`（已装 `/usr/bin/llc`），安装版用同目录 bundled `llc`。

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
3. `asuka/` 是 **git 子仓**（文法驱动解析器生成框架）：在子仓内修改后要先在子仓提交并推送，主仓再提交新的子仓指针；`gen_parser.sh` 从子仓构建。
4. `SYMBOLS.md`、`src/generated/`、`target/`、`build/` 都是产物或生成物，不要整读；`.lcl` 是二进制包，不要读。
5. `example/*.aya`（26 个）是端到端测试的主要手段；新特性至少配一个 example 用例。
6. `std/` 交付预编译 `.lcl`，源码为 `std/**/*.aya`。

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

## 提交前检查

每完成一段必须通过检查并推送（`git push origin rust`）：

```bash
./scripts/check_all.sh        # 版本号 + 文件行数 + 符号地图是否过期
cargo check --all-targets
cargo test
```

## 分支约定

- `rust`：**开发分支**，日常提交都进这里（跟踪 `origin/rust`）。
- `release`：**发布分支**，从发布点拉出，只做发布修复与版本合并；发布用标签 `vX.Y.Z` 标记。
- `origin/master`、`origin/runtime`、`rust-old` 是旧实现/历史分支，不要在其上开发。
- 工作节奏：**每完成一个可验证的阶段就提交并推送**（`git push origin rust`）。

## 版本号同步（必须一起改）

权威版本号是 `Cargo.toml` 的 `version`，不得低于最新发布标签 `vX.Y.Z`；发布时二者必须一致。改版本时以下位置同步：

| 位置 | 说明 |
|---|---|
| `Cargo.toml` | 权威版本号 |
| `Cargo.lock` | 跑 `cargo check` 自动更新 |
| `vscode-ayanami/package.json` | VSCode 插件版本 |
| `std/ayanami.toml` | 标准库包版本 |
| `README.md` | 安装产物文件名、ayanami.toml 示例 |

- 编译器源码禁止硬编码版本号：已统一用 `env!("CARGO_PKG_VERSION")` 自动跟随（`src/compiler/build.rs`、`src/main.rs` 的 `new` 模板）。
- 校验：`./scripts/check_version.sh`（版本漂移时报错退出），发布前必跑。
- 发布流程：改版本 → `cargo check` → `./scripts/check_version.sh` → 构建 vsix/tar 产物 → 提交 → 打 `vX.Y.Z` 标签 → 推送分支和标签。

## 语言与文档索引

- 语言语法、类型系统、所有权（默认移动 + `unique` + `ref`）、泛型、枚举布局、操作符重载：见 `README.md`。
- 标准库 API：`std/README.md` 与 `std/**/*.aya`。
- VSCode 插件：`vscode-ayanami/README.md`。
- 快速事实：参数顺序是 `类型 名称`；默认所有权（非 Copy 值移动），`unique` 是独占堆指针，`self` 消费 / `ref self` 借用 / `ref mut self` 可变借用（原语用 `self`）；无 `shared`/`weak`/GC；泛型单态化；枚举字段 `_tag` / `_data_V`；操作符映射到 `add` / `eq` / `index` 等方法。
