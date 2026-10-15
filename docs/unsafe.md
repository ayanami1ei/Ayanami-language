# M5：unsafe / FFI 安全边界（设计）

> 状态：**设计完成；P1（`unsafe {}` / `unsafe fn` + asm/unsafe-call 门控）实施中**（2026-10-07）
> 关联：bd `Ayanami-language-576`（epic）、GitHub #6；联动 #143（static mut）、#71（指针↔int）
> 依赖说明：bd 中 M5 链接了 Phase 4（泛型推断统一化），但 P1 的语法与检查与泛型推断无耦合，
> 可先行；P2+ 的 raw pointer 泛型化再与 Phase 4 对齐。

## 1. 三层模型

```
Safe（默认）── 不得执行可导致 UB 的操作
   │ unsafe { ... } / unsafe fn
   ▼
Unsafe（受限机器操作）── raw pointer、asm、FFI、unchecked、union、volatile
   │
   ▼
machine（runtime.c / 系统调用 / 硬件）
```

原则：unsafe **不引入新类型/运行时**，只是把已有或新增的机器级操作纳入显式边界；
safe 代码的健全性不因 unsafe 存在而削弱（unsafe 块内的违规不影响外部检查）。

## 2. 需要 unsafe 的操作（分期）

| 操作 | 阶段 | 现状 |
|---|---|---|
| inline asm（`asm(...)`） | **P1** | 现无门控 → 纳入 |
| `unsafe fn` 调用 | **P1** | 新语法，调用点门控 |
| `static mut` 访问（读/写/取引用） | **P1**（联动 #143） | 已门控：读写与 `ref mut` 均需 unsafe |
| raw pointer 解引用/字段（`*p`） | **P2**（576.2） | 已实现：`*T` 类型、`*p` 读写、`(*p).field` |
| 指针 ↔ int cast | **P2**（联动 #71） | 已实现：`*T as usize` / `usize as *T`（需 unsafe） |
| unchecked indexing（`arr.get_unchecked`） | P3（576.3） | 尚无 |
| union / reinterpret / volatile | P3 | 尚无 |
| `extern "C"` 调用 | P4（迁移） | std 大量使用；需 std 侧 `unsafe` 包装迁移后再门控 |

## 3. 语法

```ayanami
unsafe fn read_reg(int addr) -> int {
    asm("mov ...")          // unsafe fn 体内允许
    return 0
}

fn main() -> int {
    unsafe {                 // unsafe 块：块内允许机器级操作
        asm("mov $1, $0", in(reg) 1, out(reg) r)
    }
    unsafe { read_reg(1) }   // 调用 unsafe fn 需 unsafe 上下文
    return 0
}
```

### 3.1 P2：裸指针 `*T` 语法

```ayanami
extern "C" fn memcpy(*void dst, *void src, usize n) -> *void

fn main() -> int {
    p = Ptr::new(41)
    raw = unsafe { p.as_ptr() }        // Ptr[T] -> *T
    unsafe {
        x = *raw                        // 解引用读
        *raw = 42                       // 解引用写
        addr = raw as usize             // 指针 -> 整数
        q = addr as *int                // 整数 -> 指针
        r = ref x as *int               // 引用 -> 裸指针
        if *q != 42 { return 1 }
    }
    return 0
}
```

- `*T` 是 **Copy** 标量（LLVM `ptr`），无 drop、无所有权；`null` 可经 `as *T` 构造空指针。
- 需要 unsafe：解引用（读/写）、`*T` 相关 `as` 转换、`Ptr[T].as_ptr()`。
- 字段访问经显式解引用：`(*p).field`（括号后接 `.`）。
- 指针算术（`offset`）留 P2b/后续，先不做。

- `unsafe { ... }` 是**语句块**（作用域/drop 语义与普通块一致）。
- `unsafe fn` 的函数体整体处于 unsafe 上下文；**调用**该函数需 unsafe 上下文。
- 暂不提供 `unsafe` 表达式形式（块即表达式语句的载体）。

## 4. 语义与检查

- HIR 降级期维护 `unsafe_depth`（块）+ 当前函数 `is_unsafe`；
- 违规（asm / 调用 unsafe fn / `static mut` 访问）在 safe 上下文报错，带 span；
- 错误信息：`` `asm` requires an `unsafe` block (at l:c) `` / `` call to `unsafe fn f` requires an `unsafe` block (at l:c) `` / `` read of mutable static `X` requires an `unsafe` block ``；
- `.lcl` 导出（formatter）保留 `unsafe`（generic source round-trip）；
- 不改变类型/ABI；`unsafe fn` 与普通 fn 同一调用约定。

## 5. 阶段与验收

| 阶段 | 内容 | 验收 |
|---|---|---|
| **P1（576.1）** | `unsafe {}` / `unsafe fn` 语法 + asm / unsafe-call / `static mut` 门控 | 正例：unsafe 内 asm、unsafe fn 调用、static mut 读写；负例：safe 中 asm / 调用 unsafe fn / static mut 访问报错；formatter round-trip |
| P2（576.2） | `*T` 裸指针：`Ptr.as_ptr()`、`as *T`（int/Ptr/ref ↔ `*T`）、`*p` 解引用读写、`(*p).field` | 正例：as_ptr/解引用读写/指针↔int 往返；负例：safe 中解引用或 `*T` 转换报错；`*T` 可用于签名/局部/结构体字段；#71 关闭 |
| P3（576.3） | unchecked indexing、union、reinterpret、volatile | 各自正/负例 |
| P4 | extern/asm 调用迁移（std 协作，逐步） | std 全部经 unsafe 包装 |

## 6. 实现状态

- [x] **P1（576.1）**：`unsafe` 关键字/语法（`unsafe {}`、`unsafe fn`）、HIR 检查（asm / unsafe 调用门控）、formatter、正负例
      - 例：`example/test_unsafe.aya`、`example/test_asm.aya`（已迁移 unsafe）
      - 负例：`tests/compile_fail/asm_requires_unsafe.aya`、`tests/compile_fail/unsafe_call_requires_unsafe.aya`
      - 检查点：`Ctx::unsafe_depth` / `cur_fn_unsafe`；`asm` 在 safe 上下文报错；调用 `unsafe fn` 在 safe 上下文报错
      - `static mut`（#143）：读、写、`ref mut` 取引用均在 safe 上下文报错（`Ctx::require_unsafe`）；现有 static 用例已迁移 `unsafe` 块
      - 负例：`tests/compile_fail/static_mut_read_requires_unsafe.aya`、`static_mut_write_requires_unsafe.aya`
      - 跨包：`.lcl` 符号 flags 携带 `unsafe`，导入侧调用点同样门控
- [x] **P2（576.2）**：`*T` 裸指针与指针↔int（#71）
      - 类型：`*T`（AST/HIR/LIR `ptr`；Copy、无 drop；签名/局部/字段/泛型替换/.lcl 编解码全链路）
      - 语法：`*T` 类型、`*p` 解引用（读）、`*p = v`（写）、`(*p).field` 字段读写
      - 转换（全部需 unsafe）：`int/IntN ↔ *T`（inttoptr/ptrtoint）、`ref T/Ptr[T]/fn(..)/*T → *T`（bitcast）；`ref [T]` 自动取缓冲区指针
      - `Ptr[T].as_ptr() -> *T`（拥有 Ptr 不消费；`ref Ptr[T]` 亦可）
      - 用例：`example/test_rawptr.aya`（as_ptr/解引用/指针↔int/ref→*T/libc memcpy）；负例 `raw_ptr_deref_requires_unsafe`、`raw_ptr_cast_requires_unsafe`
      - 跨包：`*T` 签名经 .lcl 符号往返（`sig_str_to_hir`/`type_to_string`）；`unsafe fn` 调用点门控
      - 未做（P2b）：指针算术 `offset`、`union`/`reinterpret`（P3）、`extern` 调用门控迁移（P4）
- [ ] P3：unchecked/union/reinterpret/volatile
- [ ] P4：extern 调用迁移
