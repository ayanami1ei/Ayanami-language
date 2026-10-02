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
| 重新生成解析器 | `./gen_parser.sh`（会先编译 `../asuka`，较慢） |
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
3. `../asuka` 是**独立仓库**的路径依赖（文法驱动解析器生成框架），改它会影响本项目。
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

- 错误处理统一 `Result<_, String>`，没有自定义错误类型。
- IR 节点是 struct + trait object（`HirNode` / `MirNode`），字面量/节点转换走 `lower_to_mir`。
- 新增语言特性时同步整条链路：`ayanami.grammar` → AST → HIR → MIR → LIR → `emit` → example 用例。
- 注释中英混排，跟随所在文件的既有风格。
- 批量代码生成交给本地模型（见全局 `~/.config/opencode/AGENTS.md` 的 local-coder 政策）。

## 语言与文档索引

- 语言语法、类型系统、所有权（`shared`/`unique`/`weak`）、泛型、枚举布局、操作符重载：见 `README.md`。
- 标准库 API：`std/README.md` 与 `std/**/*.aya`。
- VSCode 插件：`vscode-ayanami/README.md`。
- 快速事实：参数顺序是 `类型 名称`；`shared self` 借用、`unique self` 消费；泛型单态化；枚举字段 `_tag` / `_data_V`；操作符映射到 `add` / `eq` / `index` 等方法。
