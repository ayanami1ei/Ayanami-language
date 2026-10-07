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
| 无文件 IO（std io 只有控制台） | `selfhost/src/base/file.aya` 用 `extern "C"` 包 `fopen/fread/fwrite/fclose`；FILE\* 用 `int` 承载 |
| 无 argv / 环境变量 | `/proc/self/cmdline`（NUL 分隔）读 argv；`/proc/self/environ` 读环境变量（不用 getenv：返回的 `[char]` 会被当拥有数组释放） |
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

### 2.1 已踩过的坑（实测，写代码必须遵守）

1. **`[T]` 数组没有 `.len` 字段**（它是裸指针）。要带长度就另存 `int count`（见 `sys.Args`）。
2. **不要用 `String::new(本地数组, len)` 构造 String**：std lcl 函数调用不会把 owned 数组实参标记为移动，
   调用方仍在帧退出时释放该数组，返回的 String 悬空（bd `Ayanami-language-3hh`）。
   正确做法：
   - 用 `StringBuf.to_string()`（内部是「局部数组 → `String::new` → 再移入 extern 调用抑制释放」模式）；
   - 或在**调用方**内联同样模式：`s = String::new(buf, n); strlen(buf);`（`buf` 必须是局部变量，
     形参版本无效——extern 调用不消耗形参）；
   - 或改用字符串字面量 / `substring` / `trim` / `copy` / `+` 等 std 安全构造。
3. **C 的 `int` 返回值上 32 位是脏的**（如 `fgetc` 的 EOF 会变成大正数）。单字节读写用
   `fread/fwrite` + `ref int` 单字节槽（size_t 返回，ABI 干净，且 `ref` 形参不搬移数组）。
4. `out` 是保留关键字（asm 操作数），不能用作标识符。
5. **嵌套导入按入口文件目录解析**，不是按当前文件目录。统一用 `selfhost/ayanami.toml` 的
   `[dependencies]` 别名导入（`import "aya_file"` 等）。
6. **/proc 文件 `ftell` 为 0**：读文件不能依赖 fseek/ftell 定长，要循环读到 EOF（`read_file` 已处理）。
7. **同一程序里 std 被多个源文件各自 import 会破坏泛型方法解析**（bd `Ayanami-language-8xa`）。
   base 模块一律不 `import "std"`，边界用自有结构体（如 `ReadResult`）而不是 `Option`。
8. `extern` 调用会消耗**局部**数组实参（标记 moved），但**不消耗形参**——这是坑 2 中 forget 模式的原理。
9. **`if`/`else` 分支里首次赋值的变量不会逃出分支**（块作用域）：`if c { x = 1 } else { x = 2 }` 之后用 `x`
   会报 `undefined variable x`。要先在分支外赋初值（`x = 0`），再在分支里改。
10. **泛型 impl 方法体里的编译错误会被吞掉**（GitHub issue #72）：整个 impl 被丢弃，调用方只报
   `type X has no method Y`。遇到「没有该方法」先检查该 impl 全部方法体（尤其私有方法），不要怀疑签名。
11. 发现 rust/编译器缺陷时用 `gh issue create --repo ayanami1ei/Ayanami-language --label bug --label type::bug --label selfhost` 提 issue（附最小复现），
   并在 bd 里记录 issue 链接（bead 描述）。已提：#72 泛型 impl 错误被吞、#73 lcl 不消耗数组实参、#74 重复 import std。

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

## 9. 实现状态

| 里程碑 | 状态 | 提交 |
|---|---|---|
| M0 | ✅ 完成 | `f7d0b58` |
| M1 | ✅ 完成 | `42a10da` |
| M2 | ✅ 完成 | `lexer/token.aya` + `lexer/lexer.aya`；golden 130 文件零差异 |
| M3 | ✅ 完成 | AST golden 118/119（唯一差异为 rust dump 自身失败）；错误定位 golden 8/8 |
| M4 | 🔄 进行中 | M4a/b 类型检查：诊断 golden **18/18**（未定义变量/函数、隐式转换、重赋值、未知方法/字段、重载不匹配、枚举变体、裸 return、不可变 ref 赋值）；接口/泛型/单态化待扩展 |

M1 交付：`base/file.aya`（extern C 文件 IO）、`base/sb.aya`（StringBuf）、`base/map.aya`（IntMap[V] 开放寻址）、
`base/intern.aya`（驻留，内联槽位表）、`base/sys.aya`（argv/env/system）、`base/log.aya`（stderr 诊断）；
全部 pub 函数带 effect 标注（构建零 warning）；8 个 `tests/base*_test.aya` 在 stage-0 全过。

过程中发现并提交的编译器缺陷（均附最小复现，见 GitHub issues）：

| Issue | 现象 | 自举绕过 |
|---|---|---|
| #72 | 泛型 impl 方法体错误被吞，报「类型没有该方法」 | ✅ 已修复（2026-10-05） |
| #73 | lcl 函数调用不消耗 owned 数组实参，`String::new` 悬空 | ✅ 已修复（2026-10-05）；forget 模式已移除 |
| #74 | 多源文件各自 import std 破坏泛型方法解析 | ✅ 已修复（2026-10-05） |
| #89 | 括号表达式：if/return 解析报错；跨模块泛型方法优先级丢失 | ✅ 已修复（2026-10-05） |
| #91 | 嵌套结构体字段的 ref mut 变更丢失 | ✅ 已修复（2026-10-05） |
| #105 | std/runtime.c 缺 M1.7 ovf 助手，debug 构建链接失败 | ✅ 已修复（runtime 项目化） |
| #122 | 字符串字面量含非 ASCII 生成非法 LLVM 常量 | ✅ 已修复 |
| #130 | 数组元素字段赋值 panic（FieldStore needs var_id） | ✅ 已修复 |
| #131 | if/elif 链首个 elif 之后的字符串字面量丢失 | ✅ 已修复 |
| #132 | 比较结果赋给局部变量被当成 i64 | ✅ 已修复 |
| #146 | `&&`/`||` 不短路（右侧总被求值） | ✅ 已修复（2026-10-06） |
| #147 | while 条件含 `&&` + 方法调用时条件不更新（死循环） | ✅ 已修复（2026-10-07）；现有 `while true + break` 保留 |
| #150 | 未知枚举变体静默通过（E::C 当 _tag=0） | ✅ 已修复（2026-10-08） |
| #153 | 结构体字面量未知字段不报错（codegen GEP 越界） | ✅ 已修复（2026-10-09）；check golden 覆盖 |
| #154 | extern `ref` 参数写不回调用方（临时副本）→ read_file 死循环/内存暴涨 | ✅ 已修复（2026-10-10） |
| #155 | bool 传 println(int) 生成 i1/i64 非法 IR | ✅ 已修复（2026-10-10） |
| #156 | 循环重赋值 moved（Fn 值累加器误报） | ✅ 已修复（上游） |
| #157 | release 优化：extern 调用后 load 被常量折叠（忽略指针写入） | ✅ 已修复（2026-10-11，release 验证 t==r） |
| #122 | 字符串字面量含非 ASCII 生成非法 LLVM 常量 | 未修；单测用 ASCII 片段 |
