# DWARF 调试信息（M7 / 038.3）

> 状态：**第一阶段已实现**（2026-11-18）。关联 Beads `038.3`、GitHub #36。

## 1. 目标

让原生可执行文件能够被 GDB/LLDB 按 Ayanami 源码定位：

- `break file.aya:line` 命中真实语句地址；
- `info line` / `image lookup` 能显示源码文件、行号和列号；
- `info locals` / `frame variable` 能列出局部变量和参数；
- debug 与 `--release` 构建都保留行表；优化后已被消除的变量允许显示为不可用。

## 2. IR 映射

MIR 语句本来就有 `Span`。LIR 在降级时把当前语句位置附着到语句期间产生的每条
`LirNodeBox`；没有声明位置的合成指令回退到函数位置。LLVM 发射器据此生成：

- `source_filename`；
- `DICompileUnit` / `DIFile`；
- 每个函数的 `DISubprogram`；
- 每条机器指令的 `DILocation`；
- 局部变量的 `DILocalVariable` 与 `#dbg_declare`；
- DWARF 5 和 Debug Info Version 模块标志。

局部变量名来自 HIR/MIR local 表；形参按 ABI 序号标记为 `arg`。基元类型有精确宽度和
有符号性，命名结构体以结构类型名表示；本阶段不导出完整字段偏移、枚举变体和词法作用域。

## 3. 构建模式

- 默认开启；`AYANAMI_DEBUG_INFO=0`（或 `false`）关闭。
- debug 构建沿用 `llc` 默认代码生成级别，避免改变既有程序行为。
- `--release` 仍使用 `opt -O3` + `llc -O3`，但保留 IR 元数据，使行表尽量存活。
- 调试开关参与对象缓存键，避免 debug/release 或开关前后复用到错误对象。
- 链接不剥符号；编译器不修改 GCC 的默认调试段策略。

## 4. 实现位置

- `src/lir/ir/mod.rs`：`LirNodeBox.span`。
- `src/lir/lower/fn_lower.rs` / `ctx.rs`：MIR 语句 → LIR 指令位置传播。
- `src/lir/emit/debug.rs`：DWARF 元数据、局部变量与位置表。
- `src/lir/emit/mod.rs` / `functions.rs`：带源路径发射及 `!dbg` 附着。
- `src/driver/mod.rs`：`llc -O3`（release）与调试缓存键。
- `scripts/check_debug_info.sh`：readelf + GDB + LLDB 回归。

## 5. 已知边界

- 从 `.lcl` 反序列化的依赖函数没有原始 `Span`，只保证消费方源码函数可调试。
- 未发射结构化类型成员、枚举判別值、闭包捕获环境布局；复杂值可能只显示类型名。
- release 优化后，变量可能被寄存器提升、拆分或消除；行级断点仍优先保留。
- 当前没有内联函数调用栈、宏 token span 到机器指令的逐 token 映射。

## 6. 验收

`scripts/check_debug_info.sh` 检查：

1. `AYANAMI_DEBUG_INFO=0` 不产生 `.debug_info`，程序行为不变；
2. debug 构建包含 `.debug_line` 且能命中源码第 4 行；
3. GDB `info locals` 和 LLDB `frame variable` 能看到 `a = 40`、`b = 2`；
4. `--release` 构建仍能由 `readelf`/GDB 解析源码行。
