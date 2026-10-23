# 统一环境注入（Environment Injection）设计提案

> 状态：**E1a 约定已冻结（2026-10-23）**；实现未开工（多阶段）。
> 关联：bd `Ayanami-language-3c0`；原始规划记录：`docs/future-directions.md` §二。
> 联动：异步（`docs/async.md`——运行时经 `env::runtime` ✓）、引用模型（解析器/注册表可经环境提供 ✓）、
> 效应 A3、std 侧前置（`Alloc` 接口——std main 暂无，见 §4）。

## 1. 目标与核心决策

- **`env::` 可扩展体系**：可自定义的不只是内存分配器（`env::alloc`），还包括异步运行时
  （`env::runtime`）与用户自定义环境体系（`env::xxx`）；
- **接受环境处用泛型写出可自定义的环境**：每类能力是**独立泛型参数**——库可以只放开
  「自定义运行时」而锁死分配器（不暴露对应泛型即可）；
- **使用处不写泛型、自动推导**：调用点按当前环境解析；无环境则用默认提供者；
- 零成本优先：编译期解析 + 单态化；不做运行时父对象/全局查找（除非环境策略明确要求）。

## 2. 语法提案

### 2.1 声明（需求 = 泛型 bound）

```ayanami
// 能力即接口（std/用户定义）：env::Alloc / env::Runtime / env::xxx
fn load_image[E: env::Alloc](String path) -> Image {
    // 函数体内隐式获得 E（分配能力）
}

// 只放开运行时、锁死分配器：只写 runtime 泛型
#[async]
fn serve[R: env::Runtime](int port) {
    ...
}
```

- 环境需求 = **泛型参数 + 能力接口 bound**（复用现有泛型/接口机制，无需新类型系统机制 ✓）；
- 可自定义面 = 泛型参数列表（暴露哪些能力 = 写哪些泛型 ✓）。

### 2.2 提供 / 覆盖（作用域）

```ayanami
#[env::alloc(ArenaAlloc)]
fn process_images() {
    a = load_image("a.png")     // 自动推导 E = ArenaAlloc
}
```

- `#[env::xxx(实现)]` 作用于函数/块作用域（沿用现有标注实参语法 ✓）；内层可再覆盖（最近者优先）；
- 默认提供者：系统分配器（`env::alloc = SystemAlloc`）、默认运行时等——顶层隐式存在。

### 2.3 使用（自动推导）

- 调用点直接写 `load_image(path)` ✓；编译器按当前作用域环境填泛型实参（单态化 ✓）；
- 多候选 / 歧义 → 显式选择（诊断提示 `#[env::alloc(..)]` 或显式泛型实参 ✓）；
- 与显式泛型并存：`load_image[ArenaAlloc](path)` 亦可（显式优先 ✓）。

## 3. 语义（五概念）

| 概念 | 语义 |
|---|---|
| **环境需求** | 泛型 bound（`E: env::Alloc`）；调用图传播（被调方需求 ⊆ 调用方可用环境） |
| **环境提供** | `#[env::xxx(实现)]`（作用域）或默认提供者 |
| **环境传播** | 隐式：中间函数不写泛型则“继承”调用方环境；写了则是显式需求 |
| **环境覆盖** | 内层作用域提供者优先；最近者胜 |
| **资源绑定** | 对象需长期持有环境时，用**泛型字段**显式持有（`struct Arena[E: env::Alloc] { e: E, ... }`）——不隐式存储 |

- 编译期解析：环境 = 编译期映射（能力 → 具体类型/实例）；**静态派发 + 单态化** ✓；
- 运行时确定的环境：显式引用（`ref E` 字段/参数 ✓）——成本可见（文档明示）；
- 与效应系统：能力接口可带效应（`env::Runtime` 的 IO 效应 ✓），效应检查与 `#[throws]` 正交 ✓。

## 4. 首个应用：可自定义分配器

```ayanami
// 能力接口（std 侧契约，示意；实现开工时冻结签名）
interface Alloc {
    fn alloc(ref mut self, usize size) -> *void;   // 或 [u8] 视图（冻结时定）
    fn free(ref mut self, *void p);
    // realloc / 对齐等按需
}
```

- 默认提供者：系统堆（现有 runtime 的 `unique_alloc/free` ✓）；
- 库代码只写 `[E: env::Alloc]` 泛型 ✓ 不手动传参 ✓；容器扩容/销毁需持有时用泛型字段绑定 ✓；
- 与 std 实验分支的全局分配器接口（可替换后端 + 弱符号）关系：该方案可作为
  `env::alloc` 的**默认提供者实现**之一 ✓；std 侧前置：先落 `Alloc` 接口（std main 暂无 ✓）。

## 5. 阶段与验收

| 阶段 | 内容 | 验收 |
|---|---|---|
| **E1** | `env::Alloc` + 默认系统提供者 + 作用域 `#[env::alloc(..)]` + 调用点推导 | 正例：`[E: env::Alloc]` 库函数在两种分配器（Arena/System）下工作；负例：未提供且无默认 → 报错；单态化产物零额外开销（IR 对比） |
| **E2** | 传播（继承 / 显式需求）+ 歧义诊断 + 显式泛型优先 | 调用链传播用例；多候选报错并提示显式选择 |
| **E3** | 覆盖（作用域最近者）+ 资源绑定（泛型字段） | 内层覆盖生效；容器持有分配器（扩容/销毁使用原分配器）用例 |
| **E4** | 扩展能力：`env::Runtime`（联动 async ✓）、日志 / IO、用户自定义 `env::xxx` | 各能力一个正例；自定义能力的文档化流程 |

## 6. 非目标

- 运行时全局环境栈 / 父对象查找（除策略明确要求）；
- 隐式动态派发（默认静态单态化 ✓）；
- 把所有资源塞进同一环境（能力各自独立、语义边界保留 ✓）；
- 与引用模型合并为万能抽象（共享基础设施，不共享语义边界 ✓）。

## 7. 实现约定（要点）

1. 先冻结能力接口契约（`Alloc`/`Runtime` 签名）与解析规则（最近覆盖 / 默认提供者 / 歧义报错）；
2. HIR 阶段**显式化**：隐式环境参数在 HIR 显式化为泛型实参（便于 IR / 调试 / 错误定位 ✓）；
3. 单态化零成本；运行时环境仅显式引用；
4. std 侧前置：`Alloc` 接口 + 默认提供者（std main 暂无——由 std 侧按本文契约实现）；
5. 测试口径同仓库（正/负例 + `runtime_safety` + IR 快照 + std 套件）。

## 8. E1a 冻结约定（2026-10-23，实现开工第一步）

std 契约已就绪：`std/src/system/alloc.aya` 的 `interface Allocator`
（`alloc(ref self, usize) -> usize` / `free(ref self, usize)`）+ `pub struct DefaultAllocator {}`。

| 概念 | 冻结形态 |
|---|---|
| 能力接口 | `Allocator`（std）；能力键 = 接口基名小写：`env::allocator` |
| 环境需求 | 泛型参数 + 接口 bound：`fn load[E: Allocator](...)` |
| 环境提供 | `#[env::allocator(v)]` 标注在**函数**上（作用域值，最近者优先）；`v` 为产生实现值的**位置表达式**（局部/字段/构造） |
| 体内访问 | 具名访问器 `env::allocator`（类型为 `E`）——值级注入，不做“无名字魔法” |
| 调用点 | 直接调用带环境 bound 的泛型函数时，用当前作用域提供者填 `E` 与隐式值参数；缺失 → 编译错误并提示标注 |
| 传播（E1a 不做） | 中间函数**不继承**：要么声明同样的 bound，要么在各自入口标注（E2 再做继承） |
| 显式优先 | 显式泛型实参 / 显式传值优先于环境提供 |

**实现面（预估，改动点）**：
1. 标注解析/校验（`hir/attrs*`）：`env::<cap>(expr)` → 能力键 + 提供值；
2. `Ctx` 环境栈（函数作用域 push/pop）+ `env::<cap>` 访问器解析（`lower_expr` 的限定名路径）；
3. 泛型 fn 的**隐式值参数**（`FnSig` 增加 env 参数，类似 `hidden` 但值来自提供者而非合成）；
4. 调用点填充（值 + 类型进特化参数表）与缺失诊断；
5. 示例：`[E: Allocator]` 库函数在 `DefaultAllocator` 与自定义分配器两种提供下工作；IR 零额外开销（静态派发/单态化）。

**非目标（E1a）**：不做类型级能力（无值构造路径 ✗）、不做继承传播、不做运行时全局环境栈（§6 非目标）。

**实现状态**：
- [x] **第 1 步（2026-10-23）**：函数级提供者 + 内置访问器
      - `#[env::<cap>(v)]`（编译器内置 `env` 命名空间，属性校验三处放行：`attrs::resolve` /
        `attrs_macro::is_compiler_attr` / `passes::is_builtin_attr`）；
      - `Ctx::env_providers`（fn_id → cap → 表达式）在 `lower_fn` 登记；
      - `env::<cap>` 访问器在 `lower_expr` 解析为提供值（位置表达式，访问时重新求值）；
      - 用例 `example/test_env_allocator.aya`（`DefaultAllocator` 分配/释放）、
        负例 `tests/compile_fail/env_accessor_outside.aya`；
- [x] **第 2 步（2026-10-23）**：泛型 bound 驱动的环境需求 + 调用点自动填充
      - 需求判定：泛型 bound 基名小写命中「**调用方作用域提供者键**」→ 该泛型参数由环境提供
        （实现口径：能力键取自当前函数可见的提供者，避免要求全程序预扫描；无提供者时
        bound 不自动视为需求，体访问器会给出明确错误）
      - 调用点：取当前函数作用域的提供值（`#[env::cap(v)]` 或本函数环境值参数），记入
        `pending_env_args` 供特化绑定 `E`；并以隐含实参 `ref mut E` 插在可见实参之后、
        位置隐藏实参之前；缺失 → `no environment provider` 错误
      - 特化：`inject_env_params` 将 `__env_<cap>: ref mut <E 具体类型>` 注入形参表
        （体降级的局部注册随之成立）；访问器优先注解、其次环境值参数
      - 修复既有缺陷：特化**失败路径**未恢复 `current_fn`/locals/scopes（泄漏到调用方，
        导致后续调用环境作用域错误）
      - 用例 `example/test_env_generic.aya`（`fn alloc_free[E: Allocator]` 由调用方提供者自动填充）

- [x] **E2（部分，2026-10-23）**：传播（显式 bound 链）
      - 可用能力键 = 本函数提供者注解键 ∪ 本函数环境值参数（`__env_<cap>`）→ 泛型函数可把
        环境继续传给下层（`middle[E]` → `inner[E]` 链路自动传递）
      - 显式泛型实参优先于环境提供（`inner[DefaultAllocator](...)` 仍自动传值）
      - 用例 `example/test_env_chain.aya`
      - 块作用域覆盖（`#[env::cap(v)] { ... }`）：`env_blocks` 栈，最近者胜；用例 `example/test_env_block.aya`
      - 待做：跨包能力键、歧义诊断
