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
2. **`String::new(本地数组, len)` 悬空问题已修（2026-10-17 实测）**：`fn make() -> String { buf = [char; 3]; ...; return String::new(buf, 3 as usize) }`
   返回后使用正常（探针 `tests/probe_string_new.aya`）。`StringBuf.to_string/substring` 可直接用该模式。
3. **C 的 `int` 返回值上 32 位是脏的**（如 `fgetc` 的 EOF 会变成大正数）：extern 返回定宽 `i32`
   并在调用点显式 `as int`（新版 rust 不再隐式转换 i32→int，2026-10-17 起编译报错）。
   文件 IO 用 `fread/fwrite` 块读写（`#154` 已修，不必再逐字节绕路）。
4. `out` 是保留关键字（asm 操作数），不能用作标识符。
5. **嵌套导入按入口文件目录解析**，不是按当前文件目录。统一用 `selfhost/ayanami.toml` 的
   `[dependencies]` 别名导入（`import "aya_file"` 等）。
6. **/proc 文件 `ftell` 为 0**：读文件不能依赖 fseek/ftell 定长，要循环读到 EOF（`read_file` 已处理）。
7. **std 多源重复 import 破坏泛型方法解析**（#74）已修；base 模块仍保持「不 import std、边界用自有结构体
   （如 `ReadResult`）」的风格以减少依赖。
8. `extern` 调用会消耗**局部**数组实参（标记 moved），但**不消耗形参**——这是坑 2 中 forget 模式的原理。
9. **`if`/`else` 分支里首次赋值的变量不会逃出分支**（块作用域）：`if c { x = 1 } else { x = 2 }` 之后用 `x`
   会报 `undefined variable x`。要先在分支外赋初值（`x = 0`），再在分支里改。
10. **泛型 impl 方法体错误被吞（#72）已修**（2026-10-05）。
11. 发现 rust/编译器缺陷时用 `gh issue create --repo ayanami1ei/Ayanami-language --label bug --label type::bug --label selfhost` 提 issue（附最小复现），
   并在 bd 里记录 issue 链接（bead 描述）。已提：#72 泛型 impl 错误被吞、#73 lcl 不消耗数组实参、#74 重复 import std。
12. **`ref String` 形参直接存入结构体字段会双重释放**（2026-10-17 用探针 `tests/probe_ref_field.aya` 复测，
    仍然存在）。规则：**把 ref 形参放进拥有字段/返回值，一律显式 `.copy()`**。
13. **枚举不做 `==` 比较**（2026-10-17 复测仍报 `no matching overload of ne (E, E)`）；
    用 **`match`** 分派（惯用法，模式用裸变体名：`match x { A => ..., B => ... }`；
    注意 `E::A` 限定形式会被检查器拒绝）。AST/Token 的 tag 存 int 属表示设计（并行数组），不算绕路。
14. **namespace 内的函数互相调用要写全限定名**（`Path::last_sep(s)`），裸名报 `undefined function`；
    `log::raw_error` 这种「模块别名当命名空间」的写法不成立——`import "aya_log"` 后直接用 `raw_error(...)`。
15. **`[T]` 数组元素是拥有值**：`String` 元素直接放进结构体字段同样要 `.copy()`（同坑 12）。
16. **注释只有 `//`**；`#` 只用于属性（`#[alloc]`/`#[state]`），写成 `# 注释` 会被当运算符报
    `expected '(', found Operator(+)`。批量生成代码时最容易踩。
17. **`if/elif/else` 分支内首次赋值的变量不逃出块**：需要在分支内赋值、分支后使用，先在外层声明
    （`v = mk_int("0"); if ... { v = ...; }`），否则报 `undefined variable`。
18. **值在分支里被移动会传染**：`a = b` 移动 `b` 后再用 `b` 报 `use of moved value`；
    多处使用一律 `.copy()`（`ptext = full.copy() + "|"`）。
19. **emit 侧分派顺序**：`Ident` 局部变量按 `local_kinds` 分派，新 kind 必须插在 `kd >= 2`（struct）
    这类宽条件**之前**；数组专用查询 `elem_of_local` 会按 kind==10 提前返回 -1，枚举局部要用
    `etype_of_local`（不做 kind 检查）。
20. **函数/方法签名要按参数类型传 `ptr`**：枚举/结构体参数在 LLVM 签名里是 `ptr %aN`（不是 `i64`），
    调用点同理；`Emitter` 维护 `fn_pkinds` 签名表（`add_fn_sig` 收集，`fn_param_kind` 查询）。
21. **Emitter 表容量要留足**：自举合并源有 300+ 函数 / 40k AST 节点 / 1500+ 字符串字面量；
    fn 表 1024、pkinds 8192、strs 4096、locals 512/函数（超限会静默丢条目 → ABI 错乱）。
22. **`elem_bytes` 必须与 `elem_stride` 一致**：`[String; n]` 元素 16 字节（ptr+len），
    曾经 malloc 按 8 字节而 GEP 按 16 字节 → 堆越界 → 大文件解析后 AST 表被踩坏（tags 变指针）。
23. **char 载入用 `zext i8`**（0-255 码点）；只有 `fgetc` 这类 C `int` 才 `sext i32`。
    曾经 char 用 sext → UTF-8 字节 ≥128 变负数 → 中文注释把 lexer 带偏。
24. **alloca 必须前置到 entry 块**：emit 时把局部 alloca 写进 `pre_buf`，函数体收尾时拼回
    `entry:` 之后，否则分支内声明的局部在支配关系上不合法（llc: instruction does not dominate）。
25. **`break`/`continue`/`elif`/char 字面量/枚举方法**都要发射：`terminated` 标志避免终结符后继续发指令；
    枚举接收者（`c.kind.tag()`）走 `find_method_id(枚举名, 方法)`。
26. **同名局部不同类型**（源码里常见复用）：`declare_local` 按 form（int/ptr/String 双槽）分别分配存储，
    但源码里最好避免同名跨类型复用。

27. **函数体被静默丢弃（已修，2026-10-17）**：`emit_fn_named` 未复位 `e.terminated`，上一个函数以
    `return`（尤其 match 臂内 return）结束会残留标志，导致**下一个函数体整段被跳过**（发射成空函数）。
    回归用例 `tests/e2e/e_terminated_leak.aya`；修复：函数入口复位 `terminated`。

28. **枚举载荷字段访问 `o._data_S._0` 需整体识别（已修，2026-10-17）**：载荷槽由
    `payload_slot`/`variant_ptype` 计算（tag 占槽 0，String 载荷占 2 槽）；必须在 `FieldAccess`
    入口把 `_data_<V>._<N>` 作为**组合**处理——先发射内层会退化成 0（外层拿不到枚举值）。
    回归用例 `tests/e2e/e_payload_field.aya`；配套 prelude 增加 `Option[T]`（is_some/is_none/unwrap_or/or）。

29. **lambda 形参不得被捕获扫描当自由变量（已修，2026-10-18）**：`cap_scan_node` 只看
    `local_kind(name) >= 0`，外层同名局部（如 for-in 的 `x`）会被误捕获进 `(int x) -> int { x * 2 }`，
    导致闭包读到外层旧值/未初始化槽（段错误）。修复：`lam_param_of` 排除 lambda 自身形参。
30. **哨兵/合成 id 不得与真实 AST 节点 id 冲突（已修，2026-10-18）**：`elem_llvm_m` 曾把
    `ety >= 2000` 一律当哨兵放行，合并 std 后真实节点 id 超过 2000（`LinkedList` 的 `[T]` 字段元素
    类型被误判 `ptr`，llc 报 `'%t23' defined with type 'i64' but expected 'ptr'`）。修复：合成 id
    改负值（`-1000-si` / `-2000-ei`），哨兵只按精确值 2000-2002 判定；泛型元素（Named/Generic 且非
    结构体/枚举/接口）归 i64。
31. **for-in/接口约定的两个 ABI 要点**：迭代器隐藏局部的 `.p` 是**槽地址**，静态 `next()` 调用前必须
    `load ptr` 取值（接口分支同理）；接口值 = `malloc(16)` 的 `{data, vtable}` box，`data` 是方法
    `self` 直接可用的指针（结构体/枚举取堆指针，int/float/String 先装箱成堆单元），vtable 槽位 =
    接口方法声明顺序，接口名从局部 `etype`（形参序言记录）取。

32. **import 解析三层回退（2026-10-18）**：`deps.path_of`（ayanami.toml）→ 相对路径
    （`Path::join(fdir, al)`，`.aya`）→ 文件名在 `std/src/{core,collections,system,meta,dev}`
    查（std 源码合并，**默认开**，`AYANAMI_STD_SRC=0` 关闭）；`.lcl` 预编译包回退同目录 `.aya` 源码
    （自举读不了 .lcl）。自举源码自身的 import 全部命中 manifest，std 回退不会影响固定点。

33. **诊断坐标回映原文件（已修，2026-10-18）**：导入模块内的函数报 warning/error 时，stage-1 曾显示
    合并后坐标（如 `test_pass.aya:933`，指向导入占位空行）。修复：`SrcMap`（每追加文件的
    路径/起始行/行数，`merge_source_map` 记录，`collect_lines` 返回行数）；Checker 持 SrcMap，
    `map_pos` 把合并行解析回原文件/原行，`err`/`warn`/`inline_error` 全部走映射——
    test_pass 等与 stage-0 逐字一致（含原源码行与脱字符）。

34. **`[char]` 字面量元素必须是 i8（2026-10-24）**：`elem_type_of` 对 `LK::Char` 曾返回 -1 →
    默认 i64/stride 8，`String::new(buf, n)` 直接接管缓冲区后按 i8 读 → `s.index(1) != 'b'`。
    修复：char 字面量返回合成 id `2002`（elem_slot/strides/bytes 均为 1 字节）。

35. **实例接收者的方法查找要先剥离 `__m<k>`（2026-10-24）**：局部 `etype` 记录的是结构体实例名
    （如 `ArrayList__m3`），`find_method_id`/`method_ret_type` 必须用**源类型**（`ArrayList`）查找，
    再用实例实参做方法特化；ForIn 的 `next()` 同理（`emit_forin_next`）。

36. **实例查名要节点 id 优先、类型名兜底（2026-10-24）**：同一泛型实参在 AST 里可能有多个类型节点
    （调用点 `[E]` 与声明处标注各一个）；只按节点 id 匹配会漏（局部退回源类型 → push 不走特化）。
    `find_struct_inst_name(ast, si, argty)` 先比 id，再比 `type_arg_name`。

37. **impl 方法的类型参数在 ImplBlock 上（2026-10-24）**：`FnDecl.cc` 对方法为空，`spec_register`
    需扫描同名 ImplBlock 的 `bb` 取第一个类型参数名（`impl_gparam`）。

38. **`ref String` 形参指针 ABI = pk 32（2026-10-25 已合入）**：`ref String` 传指向连续
    `{ptr, len}` 对的指针。实现要点：
    - String 局部 `.ptr`/`.len` 合并为 `alloca [2 x i64]` + `getelementptr +8`（原先两个独立
      alloca 不连续，写 +8 会踩到相邻槽）；
    - 调用点：局部/ref 实参直接取对地址；字面量/字段/调用结果用**入口块预分配**的临时对
      （`ref_string_temp`），且**只求值一次**（先 `emit_expr` 再 `addr_of_arg` 会把同一字面量
      注册两遍，自举源超过 8192 字符串槽后 llc 报 `@.str.8192` 未定义）；
    - 读/写：Ident kd32 装载双槽；赋值穿透写回；特化 kind 覆盖（`ty_call`/`inst_m2`）不得把
      32 改成 1；UFCS 接收者（`pkc==32`）同样要处理；
    - **关键坑**：临时对若在函数体/循环内 `alloca`，编译器自身热路径（`find_method_id`/
      `find_struct_idx` 常量名调用）会持续涨栈，stage-2 在 3.08GB 处段错误；改为入口块
      按调用点预分配后固定点恢复。
    现状：`set_hi(ref mut String)` 写回正常，test_ref_auto 通过；全量 diff 154/154；
    固定点 md5=cb284821（ref String 里程碑 a511d7b7 → 数组/Ptr/诊断对齐 857a37b1 →
    单态化警告回放 2c34e390 → #[check] 插件 cb284821）。

39. **伪诊断只影响输出、不阻断发射（2026-10-24）**：自举源上 checker 会报 1 条
    `mk_str_tmp` 重载错误（MethodCall 返回类型恒为 ""，局部类型未知）——`cmd_build` 把 check
    计数当警告继续发射。修复 Binary 的 ref 自动解引用后 `pn` 伪诊断消失；MethodCall 返回类型
    仍待补（补全会引入新的转换诊断，需同步 check golden）。

40. **AST 容量 131072 且 push 无越界检查**：自编译合并源约 9.5 万节点（2026-10-24 实测
    `NODES 94858`），余量约 36%；再增大自举源/宏展开时要先提容量。

41. **`.lcl` 版本告警暂缓（2026-10-24）**：在 import 合并路径读取 `.lcl` 头提取 version 的代码
    （即使未被调用）会让 stage-3 构建崩溃/固定点失败（字符串处理路径的隐藏不稳定）。
    `test_import.aya` 因此仍 DIFF（stage-0 打印 0.6.5→0.6.6 的过期包告警）。

42. **实例表按名字查，`st` 索引 ≠ 实例索引（2026-10-24）**：实例名 `Name__m<k>` 的 k 是 `st_count`，
    但 `find_struct_idx(实例名)` 返回的是 `st` 表下标；直接用 `inst_st_argty[st_idx]` 会取到**另一个实例**
    的实参（多实例时 ABI/布局错乱，如 `ArrayList[int]` 的 push 用了 float 实例）。统一按
    `inst_st_names[i] == obj.b` 扫描。

43. **类型节点层的合成标记必须用负值（2026-10-24）**：合并 std 后真实 AST 节点 id 会达到 2000+，
    在 `ret_kind`/`ret_info`/`layout_enc` 里按精确值 2000-2002 判定会把真实 `String` 类型节点（如 id 2002）
    误判成 char/哨兵（`fmt_g` 返回类型被发成 i64，llc: `ret i64 @.str`）。类型节点层只用负标记
    （-4001 int / -4002 bool / -4003 String / -4004 float / -4005 char / -4006 fn / -1000-si / -2000-ei）；
    2000-2002 仅限 `elem_*` 元素类型上下文。

44. **负标记贯通所有替换判定（2026-10-24）**：`ret_kind_with`/`param_kind`/`ret_kind`/`ret_info`/
    `find_struct_inst_name`/`fill_struct_inst`/`layout_*` 里判断“是否有类型实参”要用 `!= -1`，
    不能写 `>= 0`，否则负标记被当成“无实参”，返回类型/形参 ABI 退回擦除版本。

45. **浮点元素读取返回 `mk_float`（2026-10-24）**：`emit_index_read` 对 double 元素曾返回 kind 0，
    `return` 处再 `sitofp i64 <double>` → 非法 IR。`[float]`/`LinkedList[float]`/`ArrayList[float]` 读取
    一律 kind 6；元素类型判断要覆盖 -4001..-4006 标记。

46. **语句级 `elif` 条件必须设 `no_struct`（2026-10-24，已修）**：`parse_if_stmt` 的 elif 条件
    未置 `no_struct=1`，条件以标识符结尾时（如 `elif x == y { ... }`）会被结构体字面量启发式
    误判为 `y { ... }`，报 `expected } after struct literal`。表达式 if 的分支本来就有该设置；
    语句级已补齐。新增类似条件语法时注意。

47. **GenericArg 包装：类型在 `tys` 不在 `aa`（2026-10-24）**：`Name[T]` 的类型实参链是
    `XT::GenericArg` 包装（`aa=-1`，实际类型在 `tys`，`nexts` 指向下一个实参）。`ty_name`/
    `unwrap_ty` 会解开包装；直接读 `ast.aa` 会拿到 -1（实例推断/布局替换全部失效）。

48. **多类型参数实例按首个实参匹配（2026-10-24）**：实例表记录 arg1/arg2（`inst_arg2_tys`/
    `inst_st_argty2`），`fill_struct_inst` 按 StructDef 泛型链替换 gp1/gp2；键与查找仍以
    `Name[arg1]` 为准（多实参的同源结构体在测试集内唯一）。具体泛型返回类型
    （`Wrap[Box[int]]`、`T=Box[int]`）按类型自身实参解析实例；字段类型为具体泛型时（fk==2）
    访问改用实参实例名，否则链式嵌套（`A[B[C[int]]]`）会按源布局解析。

49. **泛型枚举实例（2026-10-24）**：`Option[T]` 等泛型枚举的载荷类型 `T` 不替换会导致
    `Some(p)` 绑定成 i64、`p.v` 无法解引用。实现：枚举实例表（`ei_ens/ei_srcs/ei_args`），
    `fill_enum_inst` 复制变体并按首个类型参数替换载荷（slots 按替换后载荷重算，String 双槽），
    实例名追加到 `en_names`（`%enum.Option__mK`）。**必须在 `collect_instances` 预建**：
    `%enum` 类型定义在函数前统一发射，发射期新增实例会得到未定义类型。
    枚举构造按载荷值推断实参（静态推断 + 局部源结构体名 `ln_*` 兜底）；方法接收者为枚举实例时
    剥离 `__m<k>` 按源枚举特化；形参 `pk==20` 时按类型实参解析实例，match 绑定才能看到替换后的载荷。

50. **match 标签必须带 match 唯一标记（2026-10-24，已修）**：`emit_match` 的
    `match.arm.<i>`/`match.body.<i>`/`match.guard.<i>` 此前只按臂序号命名，**同一函数多个 match**
    会重名（llc: Terminator found in the middle of a basic block / dominance）。改为
    `match.arm.<mtag>_<i>`（mtag = 生成 end 标签前的 `e.tmp`）。

51. **无类型实参的结构体字面量按字段推断（2026-10-24）**：`Box { v = 42 }`、
    `Pair { first = 1, second = "s" }` 在 `collect_instances` 预建实例（按 StructDef 字段类型
    gp1/gp2 从字段值推断），发射端 `infer_literal_inst` 复算同一实例名。负标记作字段类型时
    要单独设置 fk（String 双槽/浮点/结构体/枚举），不能走 `fty>=0` 分支。

52. **结构体形参的具体泛型类型用实例布局（2026-10-24）**：`fn deep[T](ref Box[Box[T]] b)` 的
    `b.v` 若用源 `Box` 布局（字段 T 视为 i64）会丢掉嵌套实例链。形参序言 `pk>=100` 时，
    若形参类型是 Generic 且实例表存在同源、实参基名相同的实例，局部 etype 改用该实例。

53. **`.lcl` 版本告警已可用（2026-10-24）**：此前实现（读 `.lcl` 头提取 `version="..."` 后对比
    `ayanami_version()`）会破坏固定点；根因实为坑 46 的 elif 解析缺陷（新增代码触发），修复后
    该实现稳定。import 合并阶段读取 `.lcl` 文本头即可（自举不消费包体，仍回退同目录 `.ape`
    `.aya` 源码）。

54. **枚举字段的具体泛型实例（2026-10-24）**：`self.left`（类型 `Option[Ptr[Tree]]`）的
    FieldAccess `fk==4` 若用源枚举名，后续 `_data_Some._0` 的载荷类型是未替换的 `T`。
    字段类型是具体泛型时按实例表解析实例名（与 fk==2 结构体字段同理）。

55. **Ptr 载荷访问返回解引用标记（2026-10-24）**：`_data_<V>._<N>` 的载荷类型为 `Ptr[Inner]`
    时按 i64 装载并返回标记 `10001+si`，`r.get()` 才能走裸指针解引用。
    仍缺：`ref <枚举载荷字段>` 的**字段地址**语义（test_ptr_tree 的 `r = ref self.left._data_Some._0`
    当前拿到的是值临时，写不回/链式 get 取不到目标）。

56. **宏引擎（源码级解释执行，2026-10-24）**：`selfhost/src/macro/expand.aya`。
    解析器记录注解原文（`attrs_text/line/eline`），`#[macro]` 打 dd 位 18（262144）；
    展开在 driver 首次解析后：表达式宏 `#name(...)` 自右向左（内层优先）就地替换，
    item/语句注解按行范围替换（item 结束 = 下一个 item/注解行 - 1；语句按括号平衡）；
    `input` = 被标注源码（item 含末尾换行），其后为实参源码；`__line/__col/__file` 注入；
    宏内局部**同名赋值必须更新绑定**（否则 m_lookup 首次命中旧值，panic 宏会截断）；
    仅 bit18 的函数参与宏解析（`#panic` 也是 std 宏，但普通函数不是宏）；
    宏诊断先入缓冲，`check_program` 之后输出（对齐 stage-0 的顺序与格式）。
    宏展开后重新解析并平移 SrcMap 导入区。

57. **字符串拼接的效果（2026-10-24）**：任一操作数（base_type）为 String 时，
    推断为 `alloc + state`（对齐 stage-0 的 `String.add`/`int.to_string`）。
    此前只按字面量判 alloc，导致宏库 `my_line/tok_twice` 的 state 缺失。

58. **宏库的链接错误消息（2026-10-24）**：含 `#[macro]` 的库在 stage-0 会先构建插件，
    失败消息带 `link failed: driver error:` 包装（与 import "mir" 的插件库一致）。

59. **stage-0 效果诊断的「无位置 + 按实例重复」（2026-10-25，部分对齐）**：
    - stage-0 的 std 以预编译 `.lcl` 提供，**不对 std 源跑效果诊断**；自举默认合并 std 源，
      必须按 SrcMap 路径跳过 `std/`（`is_std_src_path`），否则 `println` 等 std 函数会多出
      state 警告（test_std 等 11 例回归）。
    - 泛型单态化实例的 span=0 → 效果警告**无位置**（`raw_warn`）；`test_multi_constraint` 转正。
    - `String.add/to_string/substring/...` 的实现带 `#[alloc] #[state]`，调用点要同时标记
      alloc+state（`is_std_state_name`）；`Ptr.new`/`Box.new` 仅 alloc。
    - stage-0 按**单态化实例数**重复警告，自举已对齐（2026-10-25）：
      checker 的 `CheckOut` 按 FnDecl node 记录泛型 pub 警告（defer 模式下不打印）；
      emitter 的 `SpecOut` 暴露 `spec_nodes` 顺序；driver 在 `emit_program` 后按 spec 顺序回放
      （每个 spec 一组，alloc 先于 state），宏诊断移到回放之后。
      已确认 spec 顺序与 stage-0 完全一致：`__i0..i3` = get_mut/with_capacity/alloc/len
      （len 无效果）→ batch1；`__i4..i8` = get_mut/get/with_capacity/alloc/len → batch2。
      test_arena 的 9 条警告逐条一致。

60. **Ptr[Struct] 的自动解引用（2026-10-24）**：`fn leaf() -> Ptr[Tree]` 的**返回值**必须带
    解引用标记（`10001+si`），否则局部是裸 i64，`t.left = ...` / `t.contains(x)` 无接收者类型。
    FieldAssign 与 MethodCall 的 Ptr[Struct] 接收者要 `inttoptr` 成结构体再走正常路径；
    `ref <Ptr 值>`（如枚举载荷字段 `_data_Some._0`）按裸指针值存储（读语义等价，
    写穿/字段地址语义仍缺，通用方案见 bd izr）。

61. **多类型参数函数特化 + 发射期类型定义（已合入，2026-10-25）**：
    - 结构体字面量多类型实参保留为 GenericArg 链（解析器）；
    - `spec_register2`/`spec_params2`/`spec_args2` + `cur_inst_param2/arg2`，替换点覆盖
      param_kind/ret_kind/ret_info/pk_of_ty/layout/数组元素；
    - 方法接收者取 `inst_st_argty2`，调用点按 T/U 双替换；
    - 发射期类型定义缓冲 `type_buf`（`emit_program` 返回 `type_buf + buf`），
      `fill_struct_inst`/`fill_enum_inst` 在 body 阶段补 `%struct`/`%enum` 定义；
    - 枚举构造改为**先求值载荷**再按 Val 推断实例（`val_marker`），载荷存储抽为
      `enum_store_payload`（≤8 载荷用局部值，不分配数组）；
    - 返回 `Option[U]` 的方法调用点按替换后的实参补建枚举实例。
    **内存坑（已定位）**：`impl_gparam`/`impl_gparam2` 曾被移出 `if inst_m2 >= 0` 变成**每次
    方法调用都执行**（O(ast.len) 扫描 + String 复制）——stage-2 自编译峰值从 3.58GB 涨到
    7.64GB，4GB 上限下 stage-3 崩溃。移回特化分支后峰值 3.64GB，固定点恢复。
    结论：**表达式发射热路径里禁止 O(ast.len) 的全 AST 扫描**（即使不分配内存，扫描 +
    返回值复制在 stage-1 编译产物中会放大内存/时间）。
    剩余：`TakeIt[T,I]` 这类从参数化约束（`I: It[T]`）反推 T 的推断未做（2026-10-25 已补，见坑 63）。

62. **r1 用户引用协议（get/get_mut，2026-10-25）**：结构体 `R` 若无字段/方法，但实现了
    `get(ref self) -> ref T`（读）或 `get_mut(ref mut self) -> ref mut T`（写），则
    `r.field` / `r.field = v` / `r.method()` 自动解引用：
    - checker：FieldAccess/FieldAssign/MethodCall 成员缺失时若类型有 `get`/`get_mut` 则放行；
    - emitter：成员缺失 → `emit_protocol_deref` 调用协议方法（按接收者实例实参特化，
      `ref T` 的 ret_kind/ret_info 按 T 替换）→ 对解引用结果重试；
    - 接收者可能是实例名 `R__m<k>`：协议方法查找前先剥离 `__m<k>`；
    - 结构体字段类型 `Ptr[Inner]` 读取要返回解引用标记（并支持 `cur_inst_param` 替换），
      否则 `self.inner.get()` 链断；
    - 无类型实参字面量：`Ptr[T]` 字段从值 `Ptr.new(X)` 推断 T
      （`MyPtr { inner = Ptr::new(Point{..}) }` → `MyPtr[Point]`）。

63. **参数化约束推断与四类型参数特化（2026-10-25）**：`I: It[T]` 的 `T` 由具体实现反推——
    `infer_bound_propagate` 取 `GenericParam.tys`（约束 `It[T]`）的内层类型名，按被约束参数
    已推断的具体结构体找同名 impl 方法，统一返回类型（实现 `Option[int]` vs 接口 `Option[T]`）
    得到 `T`；`spec_register4`/`cur_inst_param3/4` 把特化扩到 4 个实参（`ZipIt[T,U,I,J]`），
    `fill_struct_inst4` 替换 gp1..gp4；结构体字面量按 GenericArg 链逐项替换（含具体实参）。
    调用点返回泛型结构体时必须**立即补建实例**：worklist 特化在 main 之后才发射，而 for-in
    在 main 发射期就需要实例名，否则退化为源方法/源布局（IR 仍合法但运行错误）。
    **递归坑**：推断出的类型实参可能是未解析的类型参数节点（与 callee 的参数同名），
    `pk_of_ty` 按**名字**替换会 `T→T` 无限递归（栈溢出段错误，test_arraylist 等 13 例）。
    修复：`resolve_inf_arg` 先把调用者特化上下文的 T/U/I/J 替换为具体实参，再用
    `type_is_gparam` 拒绝未解析参数。`test_iter_infer` 转正（全量 diff 150/154）。

64. **`[Ptr[T]; n]` 定长数组与 Ptr 元素标记（2026-10-25）**：
    - 解析器原先只认 `la2 == ';'` 的 `[T; n]`，`[Ptr[T]; n]` 被解析成下标表达式重复
      （`%v_Ptr`/`%v_T` 未初始化，llc 报 i64/ptr 类型错）。对齐 stage-0：关键字类型 /
      大写开头标识符 / 紧跟 `;` 的标识符先试 `parse_type`，后跟 `;` 才判定 ArraySized，
      否则 `mark/restore` 回溯到表达式路径。
    - `emit_index_read` 的 `Ptr[Struct]` 元素要返回解引用标记 `10001+si`（并按 cur_inst
      替换 `Ptr[T]` 的 T），否则 `ref a[i]` 后 `.get()` 退化为返回 0（test_arena 的
      Arena.get 段错误）。
    - 自举脚本内存上限 4GB→6GB：ref String + 效果诊断后 stage-2 VmPeak ~4.13GB
      （RSS 仍 ~3.6GB），4GB 会误杀（表现为 stage-3 段错误）。

65. **`#[check]` 只读检查插件（2026-10-25，自举实现）**：stage-0 的 A5d-3b 走
    MIR schema v3 blob + PIC `.so` + dlopen；自举没有 MIR，改为「合并 AST 的函数级
    MirFunction 聚合视图 + 从已发射合并 IR 构建 PIC `.so` + C driver/shim 调用」，
    原生结构体布局、无需 blob 编解码。注解识别用 parser 的 `attrs_*`（`#[check]` 插件
    定义 + `#[lib::fn]` 调用点），SrcMap 映射必须在 check **之前**做完（`smap` 会被
    check 移动）。插件诊断经 `__ayanami_diag_emit` 回调加
    `(check #[lib::fn] at line:col)` 后缀。实现位置与限制见 `docs/check-plugins.md`；
    test_check 转正（154/154）。

### 2.2 本机 stage-0 与 golden 环路（可复现）

```bash
./scripts/selfhost-stage0.sh init    # 建立 selfhost/.stage0/{bin/ayanami,bin/llc,runtime.c,std}
./scripts/selfhost.sh stage0         # 打印实际使用的 stage-0
./scripts/selfhost.sh test           # M1 base 自测（10 个）
./scripts/selfhost.sh tokens         # 词法 golden（132 文件）
./scripts/selfhost.sh ast            # AST golden（5/5）；errs（8/8）；check（29/29）；e2e（13/13）
./scripts/selfhost.sh cli            # 用 stage-0 构建自举 CLI → selfhost/build/ayanami
./scripts/selfhost.sh diff [GLOB]    # stage-0 vs stage-1 输出/退出码差分
```

要点：
- **安装版编译器**（`~/ayanami/ayanami`）的 `runtime.c` 是转发文件（`#include "../std/runtime.c"`），
  若 `~/ayanami/std/runtime.c` 缺失则链接失败；`selfhost-stage0.sh init` 生成的沙盒用仓库 `std/runtime.c`
  绕开该问题，脚本自动用 `AYANAMI_RUNTIME` 指回仓库。
- 若仓库内已有 `target/release/ayanami`（`cargo build --release`），脚本优先用它（driver 自行定位 `src/runtime.c`），
  只需补 std 包：`(cd std && AYANAMI_BIN=$PWD/target/release/ayanami ./scripts/build.sh --install $PWD/target/release/std)`。
- golden 中间产物统一放 `build/`（`AYANAMI_SELFHOST_WORK` 可覆盖），不再依赖 `/tmp/opencode`。
- 自举源码改完必须 **零 effect 告警**：`python3 scripts/selfhost-annotate.py` 按编译器告警自动补
  `#[alloc]`/`#[state]`/`#[io]`（幂等，可反复跑至收敛）。

## 3. 目录结构

```
selfhost/
  README.md            # 快速上手（构建/测试命令）
  src/
    main.aya           # CLI 入口：读 /proc/self/cmdline，按 Cmd 标签分派
    base/              # 编译器内部基础库（先于 std 缺失部分）
      file.aya         # 文件读写（extern C）
      sb.aya           # StringBuf
      map.aya          # IntMap / StrMap（开放寻址）
      intern.aya       # Symbol 驻留（u32）
      log.aya          # 诊断输出（rustc 风格）
      sys.aya          # argv/env/system
      path.aya         # 路径工具（last_sep/dir/file_name/stem/join）
      cli.aya          # 子命令 + --std/--runtime/--release 解析
    lexer/             # token.aya / lexer.aya
    ast/               # 节点 enum、打印
    parser/            # 手写递归下降（对齐 rust 版回退解析器）
    hir/               # 降级、类型解析、接口、单态化
    mir/               # 降级、内存策略、借用检查（未实现，emit 直接从 AST 出）
    lir/               # LLVM IR 文本发射（emit.aya）
    driver/            # build.aya：check/build/run 管线 + llc/gcc 调用
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
    入口 `./scripts/selfhost.sh test`，失败即退出（当前 10 个全过）。
2. **差分测试**：`./scripts/selfhost.sh diff <glob>` 同跑 stage-0/stage-1，比较 stdout（剔除双方进度行）
   与退出码；当前 `tests/e2e/*.aya` 17/17 一致。
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

同步注意（2026-10-17 第二次同步：rust 0.6.6（3900cd9）/ std 0cf5e1b）：
- `.gitignore` 与 origin/rust 有合并冲突（双方各自追加规则），解决时保留两侧；
- 新版 rust 收紧 `i32`→`int` 隐式转换，自举源码里 extern i32 返回值需显式 `as int`；
- 旧绕路批量清理（本轮）：`system_cstr` 包装（#160/#166 已修）、`fgetc` 逐字节读（#154 已修）、
  lexer 的 `&&` 嵌套（#147 已修）、`String::new` 悬空注释（已修）；
  仍保留：`ref String` 入字段需 `.copy()`（复测仍在）、枚举 `==` 不支持（复测仍在）。

同步注意（2026-10-17，rust 0.6.6 / std 3b35778）：
- std runtime 移除 `__ayanami_int_to_char`（字符转换改纯 Ayanami）：自举源码里的 15 处调用改为 `(c) as char`
  （与 std 新写法一致），并删除对应 extern 声明；
- std 与 rust 都在频繁更新，合并后必须重跑 `cli` + 全套 golden + `selfcompile`（stage-0 行为变化会直接反映在 diff 数上）；
- `sync` 脚本的「rust 有更新才重建」判断基于合并前后 HEAD 比较，**手动合并后要再跑一次 sync 或手动 `cargo build --release`**，
  否则 stage-0/std 是旧的。

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
| M4 | 🔄 进行中 | M4a-e 类型检查：诊断 golden **29/29**（+ match 穷尽/未知变体/模式解析：变体/字面量/区间/通配/or/结构体解构）；接口实现检查待扩展 |
| M5 | 🔄 进行中 | M5a-e 端到端：**17/17** e2e 与 stage-0 输出/退出码一致；**自举 CLI 可用**（`selfhost/build/ayanami <check\|build\|run>`，见下）|
| M6 | 🔄 进行中 | M6a 数组 ✅；M6b 枚举 + match ✅；M6c 字段类型/字符串内建/ABI ✅；M6e import 合并 ✅；M6f 枚举载荷类型/模式绑定 ✅；M6i const/static + unsafe ✅；M6j ref 原语 ✅；M6l 浮点 ✅；M6s 闭包捕获 ✅；M6t 运算符重载/区间穷尽/ref 数组/块尾语句/get_unchecked/#panic ✅；M6u prelude（String/char/int 方法补全）/int.to_string/静态结构体全局 ✅；M6v 接口 vtable 动态派发 + for-in 迭代器协议 + 哨兵/合成 id 冲突修复 ✅；M6w ref 结构体解引用 + `?` 运算符 + import 三层回退 + **std 源码合并转正（默认开）** ✅。example diff **108/153**（2026-10-18，同步 rust 后例子数 145 -> 153）；运行时对照 **135/153**（stage-0 vs stage-1 默认模式） |
| M7 | ✅ 固定点达成（2026-10-15） | `./scripts/selfhost.sh selfcompile`：stage-1 编译自身 → stage-2，stage-2 再编译 → stage-3，**stage-2 == stage-3**（2026-10-18 md5 `e40d8504945c9f73c3f7d82fbfba4f6d`） |

M6z 交付（2026-10-18，UFCS + 驱动消息对齐）：**UFCS 方法语法**（新 std「方法语法链」依赖，
如 `f.replace_int(i, v)` 调自由函数 `replace_int(ref mut MirFunction, ...)`）：检查器在方法
找不到时回退同名自由函数（首参去 ref 后与接收者同名）；发射器 `find_fn_by_recv` +
`emit_ufcs_call`（接收者按 `fn_param_kind(full,0)` 以 i64/ptr/临时槽/浮点传入）。
**驱动错误消息对齐**：llc/链接失败统一 `link failed: driver error: ...`（对齐 rust target.rs
包装格式）；`check warnings: N` 计数行默认不打印（`AYANAMI_SELFHOST_DEBUG=1` 开启）。

M6y 交付（2026-10-18，诊断坐标回映）：**SrcMap** 行映射（见坑 33）。剩余 diff 类别主要是
新语言特性：宏引擎（7 例）、单态化（3 例）、Ptr 内置（1 例）、asm（2 例）、标注检查
（`#[check_*]`，5 例）、ref String ABI（1 例）。

M6x 交付（2026-10-18，rust/std 同步 + String 结果槽）：

**同步**：rust 源合并 `origin/rust`（20c03bd，含限定变体模式放行等 84 文件差异；冲突
regression/gen_symbols/ir_snapshot 采用 `cmp -s`）+ std 子仓到最新 main（0ec26cc -> b219a85 ->
0bd1bfd：first/last/unwrap/expect 补 `#[alloc]`）+ stage-0 重建 + std .lcl 重建（26 模块）。
**if/match String 结果槽**：新 std `ParseError.what`（返回 String 的 match）暴露缺口——结果槽
为 i64，String 尾值此前只存 ptr（丢 len + 非法 IR）。改为 16 字节单元装箱、槽存单元指针
（`store_match_val` 返回标志、`unpack_str_res` 拆包、`emit_if_branch`/`emit_if_expr`/`emit_match` 汇总）。
**验收**：固定点保持（md5 `8b58e3e5`）；diff 101/153、运行时对照 135/153；`check_all` 全过
（IR 快照按新 rust 重生成；回归 positive=146/negative=89/runtime=52）。

M6w 交付（2026-10-18，ref/`?`/import/std 转正）：

**ref 结构体/枚举解引用**：`r = ref mut s`（kind 30）记录被引用目标类型（结构体 5000+si / 枚举
6000+ei），Ident kd 30 命中时再解一层返回结构体/枚举值——`r.get()`/`r.v`/`r.v = x` 全部可用
（test_ref_receiver、test_nll 转正）。**`?` 运算符**（`ET::TryOp`）：tag != Ok/Some 时 ptrtoint
后从当前函数返回，否则取载荷（`emit_variant_payload` 由 for-in 泛化；test_try 转正）。
**import 三层回退**：见坑 32。**std 源码合并转正**：默认开（`AYANAMI_STD_SRC=0` 关闭），
自举源码 import 全走 manifest 不受影响，固定点验证通过——diff 99 → 113（std 集合/接口例子群检查输出对齐）。
**遗留**：std 代码 4 条 effect 警告（arraylist/linkedlist `first`/`last` 缺 `#[alloc]`，上游同样缺；
std 子仓 pinned 0cf5e1b、main 领先 67 提交，待 std 同步后提交修复）。

M6v 交付（2026-10-18，接口 vtable + for-in）：

**接口 vtable 动态派发**（对齐 rust 侧 `{data, vtable}` 装箱设计）：接口形参按 `ptr` 传（box 指针）；
调用点对具体值自动装箱（`emit_box`：malloc(16) `{data, vtable}`，int/float/String 先装箱成堆单元）；
接收者 kind 21 经 `emit_iface_dispatch` 间接调用（vtable 槽位 = 接口方法声明顺序）；接口返回类型
（`ret_kind 6`）装箱 + `ptrtoint`，调用点 `inttoptr` 还原；接口结构体字段（`fill_struct fk 6`）与字面量
自动装箱；`emit_vtables` 按登记的 (类型.接口) 对发射 `@vtable.<T>.<I>` 全局（条目 = impl 方法本体，
primitive 的 `self` 本就是指向值的指针，与 box `data` 一致，无需 thunk）。
**for-in 发射器**（此前完全缺失）：`ST::ForIn` 直接发射迭代器协议（`next() -> Option`，tag==Some 循环），
`emit_forin_next`（结构体/枚举静态调用、接口 vtable 派发）+ `emit_forin_payload`；break/continue/嵌套可用。
**哨兵/合成 id 冲突修复**：见坑 30；**闭包形参遮蔽修复**：见坑 29。
验收：diff 88 → **91/145**（test_oop/test_for_in/test_iter_infer 转正）；std 合并模式下
test_iface_box/test_iface_field/test_constructors/test_collection_any/test_usage_infer/test_early_return_move/
test_arraylist 行为通过；固定点保持。

M6u 交付（2026-10-17）：**prelude**（`selfhost/src/base/prelude.aya`，纯 Ayanami 实现 std 缺失方法：
String trim/contains/index_of/repeat/to_upper/to_lower/parse_int/is_int、char is_space/is_digit/is_alpha/is_alnum/
is_upper/is_lower/to_digit/to_upper/to_lower、int to_string；语义对齐 std/src/core/string.aya），
driver 的 `merge_source` 无条件追加（候选路径探测）；emitter 按方法名分派到 `aya_*` 函数；
**静态结构体全局**（kind 4 = global i64 堆指针 + `@__aya_init_globals` 启动初始化函数，main 入口调用；
Ident/Ref 装载 inttoptr 还原结构体值）。

M6t 交付（2026-10-17，闭包捕获之后）：**运算符重载**（结构体/枚举二元运算走 `add/eq/...` 方法，
对应 rust `binary_op_to_fn_name`）；**整数/字符区间穷尽分析**（对应 rust `scalar_exhaustive`：字面量/区间/字符模式
按 u8/i8/u16/i16/u32/i32/char 值域合并覆盖）；**ref 数组/结构体取值**（数组引用取数据指针、结构体/枚举取堆指针，
与形参 ABI 一致；kd 3 数组引用形参按数组装载）；**块尾语句发射**（`emit_block_all`：if/while/for/unsafe 体的
块尾表达式此前被提升出语句链后丢弃，如 `if lo < j { qsort(ref mut a, lo, j) }` 递归调用消失）；
**get_unchecked/get/index 数组方法**；**`#panic` 宏**（调用运行时 `__ayanami_panic_at`，退出 101）；
**内置 declare 去重**（`decl_has`，修 memcmp 重复 declare）；**指针类值槽写入统一**（`store_slot_i64`）。

M6s 交付（2026-10-17）：**闭包捕获**。Fn 值统一为「环境块指针」（块首槽 = 代码指针，间接调用
`code(args..., env)`，env 作末位实参；命名函数/非捕获 lambda 用静态 env 全局 `@fnenv.*`/`@lamenv.*`，忽略末位 env）。
捕获分析在发射闭包值时扫描 lambda 子树（含嵌套 lambda 的引用传递），以外层局部表为准收集捕获名（同名去重、上限 32）；
lambda 定义追加末位 env 形参并从 env 恢复捕获局部（标量/引用按 kind 30 写回 env；String/数组/结构体/枚举/浮点按种类装载）。
结构体字段类型表新增 `st_ftype`；新增 `CallExpr` 发射（字段闭包调用 `(h.cb)(x)`）；`Assign` 记录 RHS 的 Fn 类型
（lambda/命名函数/Fn 局部/结构体字段），支持 `f = lambda; f(x)`。附带修复：match/if 表达式结果槽对指针类值未 `ptrtoint`；
AST 捕获遍历按标签白名单走子节点（`ET::Literal` 的 `bb` 存 LK、`ET::Ref` 存 mutable 标志等非节点字段）。
验收：`test_closure_capture` / `test_closure_hof` / `test_fn_once` / `test_fn_value` / `test_fn_fold` 全过（含 `__ayanami_live_allocs()==0`）；
diff 64 → 69/143；固定点保持。

M6b 交付：`enum` 值表示为 `%enum.E = { i64 tag, i64 payload... }` 堆指针（`Val.kind=4`）；
`E::V`/`E::V(args)` 构造（parser 产出 `Ident "E.V"`/`FnCall "E.V"`）；`._tag` 取标签；
`match` 语句与表达式（结果 alloca 合流）支持变体/字面量/`_`/bool/or 模式与载荷绑定；
函数签名表让枚举/结构体参数按 `ptr` 传递（`add_fn_sig`/`fn_param_kind`）。

M5 交付（M5d/M5e）：`src/main.aya` + `base/cli.aya` + `base/path.aya` + `driver/build.aya`。
stage-1 CLI 现可 `check` / `build` / `run` 单文件程序：parse → check → LLVM IR → `llc` → `gcc`（+ runtime.c）
→ 运行并透传退出码；`--std DIR` / `--runtime FILE` / `--release` 已解析，默认读 `AYANAMI_STD` / `AYANAMI_RUNTIME`。
`./scripts/selfhost.sh cli` 构建该 CLI，`./scripts/selfhost.sh diff <glob>` 做 stage-0/stage-1 差分
（2026-10-11 实测：`tests/e2e/*.aya` 13/13 一致，含退出码）。

M1 交付：`base/file.aya`（extern C 文件 IO）、`base/sb.aya`（StringBuf）、`base/map.aya`（IntMap[V] 开放寻址）、
`base/intern.aya`（驻留，内联槽位表）、`base/sys.aya`（argv/env/system）、`base/log.aya`（stderr 诊断）；
全部 pub 函数带 effect 标注（构建零 warning）；10 个 `tests/*_test.aya` 在 stage-0 全过。

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
| #157 | release 优化：extern 调用后 load 被常量折叠（忽略指针写入） | ✅ 已修复（2026-10-11） |
| #166 | 跨模块调用 extern（数组实参）符号被 mangle（system_arr_char） | ✅ 已修复（2026-10-12） |
| #168 | `return <void 调用>` 生成 alloca void（llc 报错） | 未修；去掉 return 规避 |
| #171 | clone char 数组越界（elem_size） | ✅ 已修复（2026-10-17） |
| #173 | 结构体字面量按声明顺序赋值 | ✅ 已修复（2026-10-17）；新字段仍追加到结构体末尾 |
| #181 | 兄弟边界启发式误判：`len`/`count` 作已用计数时合法 push 写入被误报越界（#175 回归） | 规避：`Ast`/`Deps`/`Seen` 增加 `cap` 字段（容量优先） |
