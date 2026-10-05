# C 互操作与 runtime 选择（#84 ②③）

> 状态：②③ 已实现（2026-10）；回归见 `tests/c_export/`、`tests/runtime_custom/`。

## ③ 从 Ayanami 导出 C 符号

### 语法

```ayanami
// 定义并导出（推荐）
#[export]
fn aya_add(int a, int b) -> int {
    return a + b
}

// 等价：显式 extern "C" 定义
extern "C" fn aya_scale(int v, int factor) -> int {
    return v * factor
}

// 空体 = 外部声明（导入 C 符号，不导出）
extern "C" fn c_helper(int x) -> int;
```

- 导出函数的符号名**不加 mangle**（就是函数名），与 C 直接互通。
- `#[export]` 限制：不能用于泛型函数；不能用于结构体/枚举/接口/impl 块（编译期报错）。
- 导出即公开：定义默认全局可见（ELF `T`），无需额外可见性修饰。
- 命名由用户负责：与 libc 同名会覆盖（如导出 `getchar`）。普通 Ayanami 零参函数已加
  `_void` 后缀避免**意外**冲突（#106），导出函数是有意为之，不加后缀。

### 类型映射（当前 LLVM 后端）

| Ayanami | LLVM | C 侧建议 |
|---|---|---|
| `int` | i64 | `int64_t` / `long long` |
| `i8`…`i64` / `u8`…`u64` | iN | `intN_t` / `uintN_t` |
| `float` | double | `double` |
| `f32` | float | `float` |
| `char` | i8 | `char`（当前仅 ASCII；Unicode 扩展见 bd 67b） |
| `bool` | i1 | `_Bool`（建议跨 C 边界用 `int` 更稳） |
| `ref T` / `ref mut T` | ptr | `const T*` / `T*`（出参用 `ref mut`） |
| `void` | void | `void` |
| `FnPtr` | ptr | 函数指针 |
| `[T]` / `String` / 结构体 | Ayanami 布局 | 无稳定 ABI 承诺，不建议直接跨 C |

出参示例：

```ayanami
#[export]
extern "C" fn aya_fill(ref mut int dst, int v) -> void {
    dst = v
}
```

```c
void aya_fill(int64_t *dst, int64_t v);
```

### 实现位置

| 环节 | 位置 |
|---|---|
| 属性白名单/校验 | `src/hir/attrs.rs`（`ALLOWED`、泛型/非函数拒绝） |
| `#[export]` → extern C | `src/hir/lower/body/lower_items.rs::lower_fn` |
| 定义 vs 声明 | `src/mir/lower/functions.rs`、`src/lir/lower/fn_lower.rs`（空体 = 声明） |
| 原始符号名 | `src/lir/lower/names.rs::collect_fn_names`（`extern_c` 不加 mangle） |
| 发射 | `src/lir/emit/functions.rs::emit_fn`（普通 `define`，默认可见） |

## ② 自定义 runtime 选择

### 配置

```toml
# ayanami.toml（项目根）
[runtime]
path = "custom_runtime.c"   # .c / .a / .o，相对项目根
```

```bash
# 环境变量优先级最高（路径相对当前目录）
AYANAMI_RUNTIME=/path/to/libruntime.a ayanami run main.aya
```

优先级：`AYANAMI_RUNTIME` > `ayanami.toml [runtime] path` > 内置 `runtime.c`
（开发态仓库 `src/runtime.c`，安装态二进制同目录）。

### 语义

- 指定 runtime 后**替代**内置 `runtime.c` 链接；`.c` 由 gcc 现场编译，`.a` / `.o` 直接作为
  链接输入。
- 自定义 runtime 必须提供程序实际引用的 `__ayanami_*` 助手（分配/释放、溢出检查、panic、
  契约失败等，完整列表见 `runtime.c`）。只想扩展少量助手时，可 `#include "runtime.c"`
  或把默认实现一并打进静态库。
- 自举 runtime 用法：用 Ayanami 写 runtime → `[build] target = "static-lib"` 产出
  `libruntime.a` → 程序 `[runtime] path = "build/libruntime.a"`。

### 实现位置

| 环节 | 位置 |
|---|---|
| 配置解析 | `src/package/config.rs`（`ProjectConfig.runtime`） |
| 解析优先级 | `src/driver/runtime.rs::resolve_runtime` |
| 链接 | `src/driver/mod.rs::objects_to_exe_with_runtime` |
| 传递 | `src/compiler/build/target.rs`、`compile.rs`、`deps.rs`、`lir.rs`（递归构建共用） |

## 验收与回归

| 用例 | 位置 | 校验 |
|---|---|---|
| C harness 调用导出符号 | `tests/c_export/` | 静态库 + gcc 链接 + 运行退出码 0 |
| `[runtime] path`（.c） | `tests/runtime_custom/` | 程序退出码 42（marker 仅自定义 runtime 提供） |
| `AYANAMI_RUNTIME` 覆盖 | 同上 | 退出码 43 |
| `AYANAMI_RUNTIME` 指向 `.a` | 同上 | 退出码 42 |
| Ayanami 内部调用导出函数 | `example/test_c_export.aya` | 正例退出码 0 |

`scripts/regression.sh` 已包含以上步骤。
