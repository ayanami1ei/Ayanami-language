# 并发所有权与效应模型（M7 / 038.2）

> 状态：**第一阶段已实现**（2026-11-18）。关联 Beads `038.2`、GitHub #38。
> std 并发原语接口与机器层细节见 `std/docs/dev/concurrency.md`。

## 1. 目标

在不引入垃圾回收、隐式别名或完整 Rust trait 求解器的前提下，给线程边界建立可检查、
可诊断、可审计的规则：

- 跨线程值按所有权移动；拥有型值可以安全转移。
- 栈引用不能离开创建它的调用栈。
- 裸指针只能通过显式 `unsafe` 跨线程转移。
- 同步原语通过效应声明其线程与阻塞性质。
- 原子操作提供显式内存序，默认仍为顺序一致。

本阶段覆盖 `Thread::spawn` 的闭包捕获边界。`Channel::send`、异步执行器和完整自动
`Send`/`Sync` 标记求解属于后续阶段。

## 2. 所有权模型

Ayanami 没有 `ref` 字段，也没有共享引用计数。普通值和 `Ptr[T]` 都只有一个所有者，
因此按值跨线程移动不会产生隐式别名：

| 类型形态 | 跨线程规则 |
|---|---|
| 基元、`String`、拥有数组、普通 struct/enum、`Ptr[T]` | 允许，所有权随值移动 |
| `Fn(...)` 静态闭包 | 允许 |
| 捕获闭包 | 递归检查环境字段；所有捕获值都必须满足本表 |
| `ref T` / `ref mut T` | 拒绝；借用不能存入闭包，也不能进入线程 |
| `*T` | 安全上下文拒绝；显式 `unsafe` 中允许并由调用方承担生命周期责任 |
| `FatPtr` / 无法证明的接口值 | 安全上下文拒绝；显式 `unsafe` 中允许 |

字段递归检查覆盖 struct/enum 载荷、拥有数组和 `Unique` 目标。`Ptr[T]` 不会隐式
创建第二个所有者。

## 3. Sync 等价模型

安全代码不共享普通 `ref mut T`。跨线程共享可变状态必须使用 std 提供的同步原语：

- `AtomicInt`：原子读改写；
- `Mutex[T]` / `RwLock[T]`：锁保护载荷；
- `Channel[T]`：所有权传递；
- `Barrier` / `Once` / `Condvar`：同步协议。

因此本阶段不引入独立 `Sync` 标记接口。需要共享的底层句柄若通过裸指针传递，必须进入
显式 `unsafe`。该限制同时覆盖要求“不可信插件/库不能静默创建线程别名”的目标。

## 4. 并发效应

新增四个内建效应，随 `.lcl` 的声明摘要导出并参与调用图传播：

| 效应 | 含义 |
|---|---|
| `#[cross_thread]` | 调用边界会把值或执行转移到另一线程，执行捕获安全检查 |
| `#[thread_safe]` | 实现自身允许被多个线程并发调用 |
| `#[atomic]` | 操作不可分割，并具有文档声明的内存序 |
| `#[blocking]` | 调用可能阻塞当前线程或引发调度 |

`#[cross_thread]` 是可检查边界。std 的 `Thread::spawn` 带有该效应；编译器检查其闭包
参数，发现非 Send 捕获时给出带 span 的错误。显式 `unsafe` 是唯一逃生口。

效应可以传播到调用者，从而让普通函数暴露 `cross_thread` / `blocking` 行为。它们不改变
LLVM 的 `memory(...)` 属性。

## 5. 内存序

std 增加 `Ordering`：

```ayanami
Ordering::Relaxed()
Ordering::Acquire()
Ordering::Release()
Ordering::AcqRel()
Ordering::SeqCst()
```

`AtomicInt::load` 接受 `Relaxed`、`Acquire`、`SeqCst`；`store` 接受 `Relaxed`、
`Release`、`SeqCst`。不适用的组合在运行时 panic。读改写操作继续使用 x86-64 的
`xchg` / `lock` 指令；即使请求较弱序，也允许提供更强保证。

顺序一致 store 使用带全栅栏效果的交换，而普通 release store 保持普通 `mov`。这修正了
“顺序一致”名称与 StoreLoad 语义不一致的问题。

## 6. 实现位置

- `src/hir/effects/mod.rs`：并发效应注册。
- `src/hir/lower/body/concurrency.rs`：跨线程边界检查与结构递归。
- `src/hir/lower/body/expr_call.rs`：调用边界的检查挂接。
- `src/hir/lower/ctx.rs` / `stmt_lower.rs`：闭包局部值的捕获元数据。
- `std/src/system/thread.aya`：`#[cross_thread]` 边界与效应声明。
- `std/src/system/sync.aya`、`std/src/core/arch_x86_64.aya`：内存序与同步效应。

## 7. 验收

- 安全闭包可在 `Thread::spawn` 中移动拥有值。
- 捕获裸指针的闭包在安全上下文拒绝，在 `unsafe` 上下文允许。
- 通过局部变量间接传递捕获闭包时仍执行相同检查。
- `Ordering` 的 load/store 组合运行正确，非法组合 panic。
- std 并发套件、主仓正负回归、IR 快照与零告警门禁全部通过。

