# 异步（async）设计提案

> 状态：**设计提案（Proposed，未实现；语法与约定在实现开工时冻结）**（2026-10-18）。
> 关联：bd `Ayanami-language-038.4`、GitHub #63。
> 联动：M7 并发（`038`，未开工）、效应系统 A3（`docs/annotations.md`；A3a–c 已落地，A3d 待做）、
> 闭包 M2 ✓、环境注入（`docs/env-injection.md`——异步运行时经 `env::runtime` 提供）。
> 原始规划记录：`docs/future-directions.md`。

## 1. 目标（核心决策）

- 编译器只提供**最小面**：一个 `Future` 协议接口（poll/Waker）+ `#[async]` **标注式**语法；
- `#[async] fn` 降级为**普通 tag 枚举状态机**——复用现有枚举机制，**无隐藏表示**：
  状态机就是普通值（可移动 / 装箱 / 存字段 / 传参）；
- **运行时完全可换**（同 Rust）：执行器不绑定编译器；std 仅提供可选的最简执行器；
  用户/库自带运行时（经 `env::runtime` 环境提供）；
- **无 `Pin` / 无安全自引用**（沿用 M4 结论「不引入安全自引用字段」）：
  状态整体由 `ref mut self` 访问，poll 内直接枚举字段访问，天然无自引用。

## 2. 现状（2026-10）

| 能力 | 现状 |
|---|---|
| 协程 / async | 无 |
| 枚举（状态机载体） | tag + 每变体载荷结构（`collect_enum.rs`）✓ |
| 闭包 / 接口虚调用 | M2 ✓（胖指针 + vtable） |
| 泛型单态化 / 弱链接去重 | ✓（`.lcl` 源码重实例化 + `linkonce_odr`） |
| 效应 | A3a–c ✓；A3d（`?` 空效应消除）待做 |
| 并发原语 | 无（M7 `038` 未开工） |

## 3. 设计

### 3.1 语法（标注式 + 前缀 `await`）

```ayanami
#[async]
fn fetch(int id) -> Data {
    d = await read_file(id)     // `await` 前缀表达式（与 `clone`/`move`/`ref` 前缀风格一致）
    return parse(d)
}
```

- `#[async]`：标注式（与 `#[extern]`/`#[copy]` 一致）；自由函数与方法均可；
- 返回类型写业务类型 `T`；**实际返回类型为编译器生成的状态机**（实现 `Future[T]`）；
- `await expr`：前缀表达式；`expr` 须为 `Future[T]`；非 async 上下文使用报错；
- 泛型：`#[async] fn f[T](...)` 允许——状态机随泛型单态化生成（同现有特化路径）。

### 3.2 最小接口（编译器提供的全部“运行时契约”）

```ayanami
// std 侧契约（示意；实现开工时冻结签名）
interface Future[T] {
    fn poll(ref mut self, ref mut Waker) -> Poll[T];
}

enum Poll[T] {
    Ready(T),
    Pending,
}

// 最小唤醒器：单方法接口（胖指针；避免裸函数指针——安全代码禁用 `fn` 指针类型）
interface Waker {
    fn wake(ref self);
}
```

- 编译器**只认这个协议**：状态机 `impl Future[T]`；`await` 展开为「poll → Pending 则挂起返回 /
  Ready 则继续」；
- 执行器（`block_on` / 多线程运行时 / 事件循环）**全部在协议之外**——std/库/用户实现；
- std 可选提供最简 `block_on`（单线程）——不作为编译器依赖。

### 3.3 状态机降级（纯 tag 枚举）

```ayanami
// 概念形态（编译器生成；命名内部化）
enum __Async_fetch {
    Start,
    Await0 { id: int, tmp: ... },   // 跨 await 存活的局部
    Done(Data),
}
```

- 状态 = 枚举变体（tag ✓）；跨 await 的局部变量 = 变体字段 ✓；
- `poll` = 对 tag 的 `match` + 直接字段访问（**无自引用**：状态整体是 `ref mut self` ✓）；
- 生成函数/类型：命名内部化 + `linkonce_odr` 弱链接（与泛型特化一致 ✓ 跨模块去重 ✓）；
- drop：普通枚举 drop ✓（**取消 = drop 状态机**，无额外语义 ✓）；
- 与 `.lcl`：状态机随泛型源码重实例化生成 ✓（同一编译器布局一致 ✓）。

### 3.4 运行时互换与 `env::runtime`

- 状态机不携带运行时状态 ✓；执行器经**环境注入**提供（`docs/env-injection.md`）：
  `[R: env::Runtime]` 声明可自定义运行时；使用处自动推导 ✓；
- 线程边界：状态机是普通值 → 可在任意线程轮询；`Send`/`Sync` 等价模型见 M7 `038.2`（另行文档）；
- 与效应系统：`#[async]` 可与效应标注叠加（`#[throws]` 等 ✓）；A3d 落地后 `await` + `?` 组合语义明确。

## 4. 阶段与验收

| 阶段 | 内容 | 验收 |
|---|---|---|
| **A1** | 语法 + 状态机降级（MIR 级 CPS）+ 最简 `block_on` | 正例：串行 await 链、循环内 await、跨 await 借用（NLL 延续）、泛型 async、`await` 嵌套 Future；负例：await 非 Future、非 async 上下文 await、状态机跨 await 持有 `ref` 到已失效值 |
| **A2** | 状态机优化（单态化尺寸、poll 无分配、Ready 快速路径） | IR 快照：状态机尺寸/调用形态；无堆分配（除显式装箱） |
| **A3** | 与 M7 并发衔接（多线程执行器、Send/Sync 等价） | 跨线程轮询 + 并发执行用例；与 `038.2` 联合验收 |

## 5. 非目标

- 编译器内置执行器/线程池（运行时归 std/用户 ✓）；
- 有栈协程（与单二进制 / 零成本目标冲突）；
- `Pin` 类机制（状态机无自引用，不需要）；
- `async` 作为语言关键字（保持标注式 ✓）。

## 6. 实现约定（要点）

1. **语义契约优先**：先冻结 §3.2 协议（名字/签名/ABI）再实现降级；
2. 状态机是**普通值**：可移动/装箱/存字段/跨模块，无隐藏寄存器或特殊 ABI；
3. 零成本优先：poll 无分配；执行器开销在协议外、可见；
4. 与泛型/弱链接/`.lcl` 重实例化一致（复用现有特化路径）；
5. 测试口径：正/负例 + `runtime_safety` + IR 快照 + std 套件（同仓库约定）。

## 7. 实现路线（2026-10-24 更新）

- **A1a（已完成 2026-10-24）**：协议（`Future`/`Poll`/`Waker`/`Context`）+ 单线程 executor +
  **手写**状态机正例：`example/test_async_manual.aya`（`block_on[F: Future[int]]` 泛型执行器，
  `Countdown` 手写状态机，前两次 Pending 后 Ready(42)）；零编译器改动。
  发现：`poll` 修改 `ref mut self` 会触发 `state` 效应告警 → 用 `#[state]` 显式声明（示例已加）；
  协议类型仍位于示例本地（std 侧由用户仓维护，后续按方法形状识别接入）。
- **A1b**：`#[async]`/`await` 作为 **A5e 结构化变换注解**（HIR 程序 → HIR 程序）内建实现：
  编译器生成普通枚举 + `impl Future`/`poll`，产物只用既有构造；协议按**方法形状**识别
  （`poll(ref mut self, ref Context) -> Poll[T]`），零名字耦合。
- **A1c**：受限子集先落地——v1 禁止 `ref` 跨 await（自引用需堆钉住，后置）、禁止 `#[sync]` 内 await；
  循环与提前 `return` 必须支持（状态机拆入口/恢复）。
- **开放**：A5e 接口稳定后再按 A5d 模式对第三方 `.lcl` 开放 `#[transform]`，async 为参考实现。
- 详见 `docs/annotations.md` §8.6（A5e 设计）与 ADR-19。

### 7.1 A1b 第一切片（`#[async]` 无 await，2026-10-24 已完成）

先不引入 `await` 语法，只做**协议接线**，把变换跑通：

```ayanami
#[async]
fn add(int a, int b) -> int { return a + b }
```

编译器在 **AST 层**（HIR 之前）生成普通构造：

```ayanami
struct __add_Fut { int a, int b, int _done }
fn add(int a, int b) -> __add_Fut { return __add_Fut { a = a, b = b, _done = 0 } }
fn __async_run_add(int a, int b) -> int { return a + b }   // 原函数体原样搬移

impl __add_Fut {
    fn poll(ref mut self, ref Context cx) -> Poll[int] {
        if self._done == 1 { return Poll::Pending() }
        self._done = 1
        return Poll::Ready(__async_run_add(self.a, self.b))
    }
}
```

- **约束（v1）**：仅顶层自由函数；非泛型；参数均为 Copy（字段拷贝）；暂不支持 `await`
  （后续切片再引入挂起点）；`Poll`/`Context` 需在调用方作用域可见（先由示例/std 提供，
  编译器不绑定名字，按方法形状与类型名解析）。
- **为什么先拆 runner**：原函数体原样保留（含嵌套 `return`/循环），避免第一切片就做
  变量提升与返回值改写；后续 `await` 切片再把 runner 内联进 `poll` 并引入状态枚举。
- **管线位置**：解析后、HIR 降级前（与 `cfg` 过滤同层）；产物只使用既有 AST 构造。
- **验收**：`example/test_async_sugar.aya`：`block_on(add(1, 2)) == 3`；
  `#[async]` 用于泛型/方法/非 Copy 参数 → 明确报错；IR 快照无新增节点种类。

后续切片：A1b-2 直线 `await`（单挂起点）→ A1b-3 循环/if 与 `ref` 跨 await 禁用 → A1b-4 高阶组合。

**实现备注**：`src/hir/async_transform.rs`；`#[async]` 在 HIR 前消费（宏展开期由
`expand_util::is_compiler_attr` 放行）。踩坑：示例最初用 `fn add`，与 std `io` 导入的
`add(int,int)` 重载同名，签名查找命中导入版本（返回 int）→ 构造函数体报
`return type mismatch`；**用户函数与导入重载同名时需换名**（既有解析行为，非本特性引入）。

### 7.2 A1b-2（直线 `await`）实现约束（2026-10-24 更新）

**结论：`await` 状态机必须在 HIR（类型检查后）做，AST 层不可行。**
第一切片之所以能在 AST 层完成，是因为无挂起点、参数类型可直接抄写；一旦出现 `await`，
状态结构体需要保存**被等待 future 的类型**与跨挂起点存活的局部变量类型，AST 层拿不到
（宏/展开期也拿不到）。因此：

1. `#[async]` 语法仍由 AST 变换识别并**标记**（保留函数体，不生成机器）；
2. 新增 HIR 级变换（`lower_items` 之后、MIR 之前）：此时 `HirFn` 已带参数/局部/HIR 类型，
   可生成 `enum __f_State` + 字段（被测 future、跨 await 局部）+ `poll`；
3. `await e` 的降级：
   - `e` 求值得到 future `f`（类型已知），存入状态字段；
   - `match f.poll(cx) { Ready(v) => 绑定并继续, Pending => 置状态并 return Pending }`；
   - 恢复入口按状态字段跳转（循环/if 需先做 CFG 拆分）。
4. v1 限制：单挂起点、直线代码（无 await 跨循环/分支）；`ref` 跨 await 禁用。

验收：`#[async] fn f() -> int { x = await g(); return x + 1 }` + `block_on(f())`；
负例：两个 await / await 在循环内 / ref 跨 await → 明确错误。

### 7.3 A1b-2 第一切片（已完成，2026-10-24）

已落地最小直线 `await`（AST 层，语法糖 `await expr` → 内部 `__await(expr)`）：

- 限制：`await` 必须是函数体开头的**连续赋值前缀**（0..N 个 `x = await g(args)`）；`g` 为同程序 `#[async]` 函数；
  `args` 只能是本函数参数；await 载荷支持 `int`/`float`/`bool`/`String`（默认值初始化）；每个 await 一个状态段；后续 await 的实参可引用更早的 await 值（进入该状态时用真实值重建 future，构造器用默认值占位）。
- 展开：构造 `__f_Fut`（参数 + `__g_Fut __w` + `y` + `_state`）；构造器立即创建内层 future；
  `poll` 首次轮询内层，`Pending` 直接返回，`Ready(v)` 写 `y` 后调用 `__async_after_f(<params>, y)`
  执行后置语句（参数名不变，无需重写函数体）。
- 诊断：`await` 在 `#[async]` 外 / 目标非 async / 循环或分支内 await → 明确报错（`await_in_loop`）。
- 用例 `example/test_async_await.aya`（`twice` = `await inc` + 1）；负例 `await_outside_async`、
  `await_not_async`。
- 分支内 await（第一切片）：`y = <默认>; if cond { y = await g(<参数>) }; <post>`
  支持（take helper 求值条件；state0 分支 → state1 轮询 → state2 后置）；条件/实参限参数。
- 循环内 await：`y = <默认>; while cond { y = await g(<参数或 y>) }; <post>`
  支持（poll 内 `while true` 状态循环；state0 重估条件并重建 future，Ready 后回 state0）。
- if/elif/else 多分支 await（每个分支一个 await，汇合到 merge 状态；条件/实参限参数）：两分支的 await 都写入同一变量并汇合到 state3；两个内层 future
  各占一个字段（类型可不同，载荷类型须一致）。
- 循环内 break/continue（2026-10-25）：`while cond { [if c { break/continue }] y = await g(args) [if c { break/continue }] }`；
  await 前的控制条件生成 `__async_pre_<f>`（返回 1=break→state2、2=continue→回 state0），
  await 后的控制条件生成 `__async_post_<f>`（Ready 后求值；continue 跳过后续控制）。
  条件/实参限参数或循环变量 `y`。用例 `find` / `find_after` / `find_ctl`。
- 循环体内连续多个 await（2026-10-25）：同一循环变量的连续 `y = await g(args)`（1..N 个）支持；
  每个 await 一个 future 字段，Ready 后立即用更新后的 `y` 重建下一个 future（状态链），
  最后一个 Ready 后执行 post 控制并回 state0。用例 `double_inc` / `triple_inc` / `double_find`。
  负例：`await_loop_mixed_stmt`（await 之间夹杂普通语句）、`await_loop_mixed_type`（结果类型不一致）。
- if 分支内连续多个 await（2026-10-25）：`y = 默认; if cond { y = await g0(args); ... }`（N≥2 走
  `async_if_multi`，单个仍走 `async_if`）。每个 await 一个 future 字段，Ready 后写 `y` 并用更新后的
  `y` 重建下一个；条件限参数，实参限参数或分支变量 `y`。用例 `if_double` / `if_triple`；
  负例 `await_if_mixed_stmt` / `await_if_mixed_type`。
- 后续：一般 CFG（任意语句、PC/状态枚举、嵌套控制流）。
