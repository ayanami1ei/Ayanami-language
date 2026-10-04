# Compiler Bootstrap / Self-hosting 路线（Phase 0.4）

## 现状

- **stage0**：Rust 2024 编写的 Ayanami 编译器（本仓库），前端 → HIR → MIR → LIR → LLVM IR → 可执行文件 / `.lcl`
- 语言已有：静态类型 + 推断、泛型单态化、所有权/借用/NLL、接口与动态派发、
  标注与效应系统、文本宏 + 函数宏、FFI/inline asm、`.lcl` 包与工具链
- 自举差距（按 beads 里程碑）：
  - **M1** 数值与位运算（哈希、表驱动解析、位操作）
  - **M2** 闭包与捕获（回调式 API、迭代器实现）
  - **M3** 迭代器与 for（日常代码可维护性）
  - **M4** 引用存储 / arena / stable object（AST、符号表、图结构）
  - **M5** unsafe / FFI 安全边界（内存布局、系统接口）
  - **M6** const / 编译期求值（表、magic、常量折叠）
  - **M7** 并发 + Debug 信息（并行编译、可调试性）
  - **M8** 增量编译 + 模块/包 + Effect Plugin API（自举迭代速度、扩展性）

## 策略（分阶段，避免一次性全量自举）

1. **stage0 稳定化**：Phase 0 的回归/IR 快照/运行时安全清单（本阶段）——后续所有大改都必须过 `check_all.sh`。
2. **最小工具验证**：用 Ayanami 重写一个真实小工具（建议 `.lcl` 符号解析器或 `fmt`），验证语言可用性并积累标准库需求。
3. **stage1 前端对拍**：lexer/parser 用 Ayanami 实现，与 Rust 版在同一批 `.aya` 上对拍
   （token 流 / AST / 报错位置），先不求完整。
4. **stage1 全编译器**：依赖 M1–M5 完成；HIR/MIR/LIR 按现有 IR 定义逐层实现，
   以 IR 快照（`tests/ir_snapshots/`）作为等价性判据。
5. **自举验证**：stage1 编译自身源码产出 stage2；要求 stage2 能编译
   `example/` 全部用例且退出码与 stage0 一致（语义等价）。

## 前置能力清单（对应 beads 里程碑）

| 能力 | 里程碑 | 为什么自举需要 |
|---|---|---|
| 定宽整数 / 位运算 / usize | M1 | 哈希、位图、表驱动、指针宽度 |
| const / 编译期求值 | M6 | 表、magic、常量折叠 |
| 闭包 + 迭代器 | M2/M3 | 编译器内部数据结构与遍历 |
| 引用存储 / arena | M4 | AST、符号表、图与自引用结构 |
| unsafe / FFI | M5 | 内存布局、系统调用、LLVM FFI |
| 并发 + Debug | M7 | 并行编译、可调试 |
| 增量编译 / 模块 / Effect Plugin | M8 | 迭代速度、扩展性 |

## 风险与决策

- **不照搬 Rust 细节**（trait/借用规则）：用 Ayanami 自身模型，自举不得倒逼语言变成 Rust 复制品。
- **先对拍后替换**：stage1 与 stage0 并存，IR 快照 + 正/负回归作为等价判据。
- **保留 stage0**：自举完成后 Rust 版作为参考实现与救援通道，不删除。
- **编译器代码风格**：沿用本仓库约定（≤300 行/文件、按阶段分目录），便于两版结构对齐。

## 完成标准

- [ ] stage1 前端在全部 `example/*.aya` 上与 stage0 对拍一致（AST/错误位置）
- [ ] stage1 生成 LIR 与 `tests/ir_snapshots/` 一致（允许规范化差异需记录）
- [ ] stage1 能编译自身源码并产出可运行的 stage2
- [ ] stage2 编译 `example/` 全部用例，退出码与 stage0 一致
- [ ] 自举链路文档化（本文件更新「实现状态」）
