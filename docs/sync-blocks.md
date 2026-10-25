# `#[sync]` 同步块（A5e 首个结构化注解）

> 状态：v1 已落地（2026-10-24，`rust`）。目标：把「加锁 / 临界区 / 解锁」变成一个**块**，
> 编译器在所有出口自动补 `unlock`——语言没有 `Drop`，手工配对 `lock`/`unlock` 在
> `return` / `break` / `continue` 路径上极易漏解锁。

## 1. 语法

```ayanami
#[sync(m)]
{
    d = m.data()
    if d > 0 { return d }   // 提前返回 → 编译器先 unlock 再返回
}
```

- 对象 `m` 必须是**位置表达式**（标识符或字段访问链）；它在进入块时求值一次，
  出口复用同一位置（位置表达式无副作用，重复求值安全）；
- 可选 `guard = d`：锁内绑定载荷（编译器插入 `d = obj.data()`，`d: ref mut T`）；
- `#[sync]` 只允许用于**块语句**；用于函数/其它语句报错。

## 2. 语义

1. 进入块前调用对象的 `lock()`；
2. 执行块体；
3. 正常离开块时调用 `unlock()`；
4. `return`：先求值返回值到临时局部 → 逆序 unlock 所有活动锁 → 返回；
5. `break` / `continue`：unlock 当前循环体内获取的锁（保持循环外已持有的锁）→ 跳转；
6. 嵌套 `#[sync]`：按获取的逆序解锁。

**协议识别（方法形状，无名字表）**：对象需有
`lock(ref self)` 与 `unlock(ref self)`；载荷访问为 `data(ref mut self) -> ref mut T`。
std `Mutex[T]` 已满足（`std/src/system/sync.aya`）。

## 3. 限制与约定

- **非重入**：与 std `Mutex` 一致；同一对象嵌套 `#[sync]` 会死锁（编译器暂不静态检查）；
- **禁止跨挂起持锁**：`#[async]` 内 `await` 不得出现在 `#[sync]` 块中（A1b 落地时强制）；
- **RwLock 暂不支持**：std 只提供回调式 `with_read`/`with_write`，缺少裸
  `read_lock`/`read_unlock`/`write_lock`/`write_unlock`；待 std 补齐后按同一协议接入；
- 块内可把锁对象**移动**走吗？不行——出口还要 unlock，借用/移动检查会拒绝；
- 不引入 RAII/`Drop`：本特性不改变所有权模型，只是编译器插入显式调用。

## 4. 验收

- 正例：`example/test_sync_block.aya`——正常路径 / 提前 `return` / `break` 均解锁
  （用 `Mutex.state` 观测 0）；
- 负例：对象不是位置表达式、`#[sync]` 用于非块语句；
- 回归：现有 std 并发套件与 `runtime_safety` 不回归。

## 5. 实现落点

| 部分 | 位置 |
|---|---|
| 语法 | 复用 `Stmt::Attributed` + `Stmt::Block`（零文法改动） |
| 降级 | `src/hir/lower/body/stmt_sync.rs`：`lower_sync_block` / `sync_unlock_stmts` |
| 出口清理 | `stmt_lower.rs` 的 `Return` / `Break` / `Continue`；循环入口标记 `loop_sync_marks` |
| 白名单 | `src/hir/attrs.rs`（`sync` 为编译器内置语句标注） |
| 文档 | 本文件；`docs/annotations.md` §8.6（A5e） |

## 6. 实现备注（2026-10-24）

- `guard = d`：`#[sync(m, guard = d)]` → `d = m.data()`（锁内、体前降级，局部绑定可用）；

- 出口清理：`Return` 先落 `__sync_ret_<n>` 临时局部再解锁；`Break`/`Continue`
  用 `loop_sync_marks` 只解到循环入口；正常出口在块尾补 unlock；
- **特化隔离**：泛型特化降级期间 `sync_locks`/`loop_sync_marks` 置空并恢复——
  否则特化体里的 `return` 会继承调用方的锁并误发 `unlock`（本次一并修复的既有缺陷）；
- 负例：`tests/compile_fail/sync_nonplace.aya`（非位置表达式）、
  `sync_on_nonblock.aya`（用于非块语句）；
- 回归：`check_all` 166 正例 / 96 负例 / 55 runtime，IR 快照 0 diff；std 套件通过。
