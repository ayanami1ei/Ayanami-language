# 自举侧 `#[check]` 只读检查插件

> 状态：已实现（2026-10-25）。`example/test_check.aya` 与 stage-0 输出一致；
> 全量 diff 154/154，固定点 md5=`cb28482101b87d647ed221436870f56b`。

## 1. 目标与 stage-0 对照

stage-0 的 A5d-3b：库里的 `#[check] pub fn f(ref MirFunction m)` 在编译期被调用，
读取被注解函数的 MIR；只读检查通过 `warn`/`error` 上报诊断，不影响代码生成。

stage-0 的实现链：MIR → schema v3 blob → `llc -relocation-model=pic` + C shim
（`__ayanami_pass_run`/`__ayanami_diag_take`）→ `.so` → `dlopen`。
细节见 rust 侧 `src/compiler/macro_expand/{check_plugin,pass_plugin}.rs` 与 `std/mir.aya`。

**自举侧没有 MIR 阶段**（HIR → LIR），因此采用两条等价路径：

1. 用**合并 AST** 生成一个函数级的只读 `MirFunction` 视图（保守聚合，不逐节点）；
2. 从**已经发射的合并 LLVM IR** 构建 PIC `.so`，用 C driver + shim 加载调用。
   因为插件由自举编译器自己编译，`MirFunction` 直接使用原生 Ayanami 结构体布局，
   **不需要** schema v3 blob 编解码。

## 2. 注解识别

- 插件函数：库内带 `#[check]` 的函数；其所在文件 stem 必须等于调用注解的前缀。
- 调用点：`#[<lib>::<fn>]`（如 `#[check_lib::warn_side_effects]`）。
- 注解原文由 parser 记录在 `attrs_text/attrs_line/attrs_eline`；`m_attr_name` 取名字，
  `cp_fn_after` 取注解结束行之后的第一个 `FnDecl` 作为目标函数。
- 诊断后缀使用**注解**的原始 `line:col`（本实现 col 固定为 1），由 `SrcMap` 在
  `check_program` **之前**映射（`smap` 会被 check 移动，见 `prepare_check_plugins`）。

## 3. MirFunction 视图（保守聚合）

`cp_scan_flags` 在目标函数的行范围（`FnDecl` 行 → 下一个顶层 item 行）内扫描 AST：

| 位 | 含义 | 触发 |
|---|---|---|
| bit0 | 非空 | 有函数体或参数（→ `node_count = 1`） |
| bit1 | store | 有参数，或 `Assign`/`FieldAssign`/`IndexAssign`/`DerefAssign` |
| bit2 | alloc | `ArrayLiteral`/`ArraySized`/`ArrayRepeat`/`StructLiteral`/`EnumConstruct`/`ToUnique` |
| bit3 | call | `FnCall`/`MethodCall`（`callee_pure = 0`） |
| bit4 | asm | `Unsafe` |

`scan_effects`（`std/mir.aya`）只要看到 alloc/asm/store/非 pure call 就返回 false，
插件据此上报 “function may have side effects”。聚合视图与 stage-0 的整函数判定语义一致，
但**没有逐节点精度**（只读检查够用；需要编辑/精确定位时再补 MIR）。

## 4. 构建与调用

`prepare_check_plugins`（check 前）→ `run_check_plugins`（emit + link 后）：

```
llc -relocation-model=pic -filetype=obj <work>/<stem>.ll -o <work>/<stem>_check.pic.o
gcc -c -fPIC <runtime.c> -o <work>/check_rt.pic.o
gcc -c -fPIC <work>/check_shim.c -o <work>/check_shim.o
gcc -shared <pic.o> <rt.pic.o> <shim.o> [link_extra] -lm -o <work>/<stem>_check.so
gcc <work>/check_driver.c -ldl -o <work>/check_driver
<work>/check_driver <work>/<stem>_check.so
```

- `.so` 里包含合并程序的全部函数（含 `main`，不执行）；插件符号按
  `aya.<fn>` 解析，找不到时回退 `aya.<lib>.<fn>`。
- shim 定义 `__ayanami_diag_emit` 并暴露 `__aya_set_diag_cb`；C driver 注入回调，
  回调按 `(check #[<lib>::<fn>] at line:col)` 后缀写 stderr，级别 2 输出 `error:`。
- C driver 按原生布局构造 `MirFunction`（18 个 8 字节字段，数组指针 + `calloc`），
  每个调用点调用一次插件；`node_count` 为 0 时插件自行返回。
- 任何构建步骤失败只打印 `check plugin: ... failed` 并继续（不阻断正常编译）。
- 执行位置：正常可执行文件链接成功之后、`run` 执行之前；因此诊断顺序是
  效果警告 → 宏诊断（driver 已移到回放之后）→ 插件诊断 → 程序输出。

## 5. 实现位置

| 位置 | 内容 |
|---|---|
| `selfhost/src/driver/check_plugin.aya` | 注解扫描、`CheckSpecs`/`prepare`、AST 标志、C shim/driver 模板、构建/调用 |
| `selfhost/src/driver/build.aya` | check 前 `prepare_check_plugins`；link 后 `run_check_plugins` |
| `selfhost/ayanami.toml` | `aya_check_plugin = "src/driver/check_plugin.aya"` |
| `std/src/meta/mir.aya` | `MirFunction` 布局 + `scan_effects`/`warn`/`error`（编译器提供 `__ayanami_diag_emit`） |

## 6. 限制与后续

- 只读检查（`replace_*`/`delete_stmt` 等编辑接口未接回自举 IR；需要 MIR 或 HIR 重写）。
- 插件函数符号按 IR 文本搜索；嵌套命名空间/同名的库需要更精确的符号表。
- `.so` 从**整个合并程序**构建（含 std 与入口），比 stage-0 的按库插件大；
  测试规模可接受，后续可按库切分。
- `check` 命令目前只跑 HIR 检查，不运行 `#[check]` 插件；`build`/`run` 会运行。
- 效果视图为函数级聚合（bit0–bit4），不是 stage-0 的逐 MIR 节点视图。
