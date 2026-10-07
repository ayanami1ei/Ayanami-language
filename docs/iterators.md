# 迭代器与 for（M3）

> 状态：**I1 完成**（2026-10-07）：`for x in iterable` + 泛型接口 `Iterator[T]` 虚调用；
> 语言侧已具备适配器表达能力（示例 `MapIter`）。std 集合的 `next` 与适配器由 std 子仓跟进（I2/I3）。
> 关联：bd `Ayanami-language-2bd`（epic）与 `.1`/`.2`/`.3`。

## 1. 目标

- `for x in iterable { ... }` 统一基于**迭代器协议**，不再依赖 `iter(fn)` 回调；
- 保留现有区间 for：`for i in (start, end[, step]) { ... }`；
- 提供足够抽象能力（闭包 + 泛型 + 迭代器）让 `map/filter/take/zip/enumerate` 可由 std/用户实现。

## 2. 协议

任何具备以下方法的类型都是可迭代的（结构化解析，无需显式实现接口）：

```ayanami
fn next(ref mut self) -> Option[T]
```

- `Option[T]` 为 std 枚举（`Some(T)` / `None`）；
- std 可提供 `interface Iterator[T] { fn next(ref mut self) -> Option[T]; }`，
  `for` 对接口类型走虚调用（现有方法解析自动处理），对具体类型走静态调用。

## 3. 语法与降级

```ayanami
for x in iterable { body }
```

等价降级（HIR 层合成，卫生名 `__for_iter_N`）：

```
__for_iter_N = iterable
while true {
    match __for_iter_N.next() {
        Some(x) => { body }
        None => break,
    }
}
```

- 迭代器值移入临时局部（拥有语义），随作用域结束释放；
- `break`/`continue` 在 `body` 内语义与普通循环一致（`continue` 进入下一轮 `next`）；
- `next` 缺失时由方法解析报错（提示实现 `next`）。

## 4. 阶段

| 阶段 | 内容 | 状态 |
|---|---|---|
| I1 | 语法 + HIR 降级 + 泛型接口虚调用 + 测试（本地迭代器/嵌套/break/continue/接口/适配器） | **完成** |
| I2 | std 集合提供 `next`（Range/ArrayList/LinkedList/…，std 子仓，用户） | 待做 |
| I3 | 适配器 map/filter/take/zip/enumerate（std/示例；语言侧已具备，`MapIter` 示例验证） | 语言侧完成 |

## 5. 实现位置

| 位置 | 内容 |
|---|---|
| `src/parser/ast/stmt.rs` | `Stmt::ForIn` |
| `src/parser/parser/decl.rs` | `parse_for`：`(` 起头 → 区间（含 `,`）或普通表达式 |
| `src/hir/lower/body/stmt_loops.rs` | `lower_for_in` 合成 AST 并降级 |
| `example/test_for_in.aya`、`tests/compile_fail/for_in_no_next.aya` | 验收与负例 |
| `src/hir/lower/body/iface_match.rs` | 泛型接口推断（`Option<T>` 等编码泛型名统一，M3 顺带修复） |
| `src/parser/parser/atom.rs` | 括号内允许结构体字面量（条件/iterable 上下文） |

## 6. 验收

- 本地 `next` 迭代器求和、嵌套 for、`break`/`continue`、接口 `Iterator[T]` 值迭代；
- 区间 for 既有用例（`test_for_usize` 等）不变；
- 缺 `next` 报错清晰；`check_all` 全绿。
