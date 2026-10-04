# 运行时安全行为（Phase 0.3）

本文件是 Ayanami 运行时错误 / UB 行为的权威清单。可确定复现的部分由
`tests/runtime_safety/`（manifest + 用例）覆盖，跑 `./scripts/regression.sh` 一并校验。

## 已检查：runtime panic（退出码 101）

输出为 Rust 风格（stderr，TTY 下红色）：

```
thread 'main' panicked at <file>:<line>:<col>:
<message>
```

| 场景 | 行为 | 位置 |
|---|---|---|
| `String.index` 越界 | panic：`index out of bounds: the len is N but the index is M` | 用户调用行（track_caller） |
| `ArrayList.index` / `set` 越界 | 同上 | 用户调用行 |
| `ArrayList.pop` 空表 | panic：`pop from empty ArrayList` | 用户调用行 |
| `LinkedList.index` 越界 | 同上（len/index） | 用户调用行 |
| 用户 `#panic("msg")` | panic：`msg` | 调用点（宏展开） |
| 整数溢出（debug 构建） | panic：`attempt to add/subtract/multiply with overflow` | 表达式位置（`a + b`） |

## 未检查：UB / 平台相关

| 场景 | 现状 | 计划 |
|---|---|---|
| 裸数组 `[T]` / `[T; n]` 越界 | 不检查（运行时无长度信息） | M5 之后：fat pointer 或编译器插检查 |
| `int / 0`、`int % 0` | 无诊断（SIGFPE / UB；实测退出码非 101） | M1 数值语义 |
| `null` 解引用 | SIGSEGV（无诊断） | M5 unsafe 边界 |
| 分配失败（malloc 返回 null） | 继续使用（UB） | 待定（M5） |
| unique 双重释放 / 悬垂 | 不保证（无 RC/GC）；借用检查阻止常见情形 | 明确为 unsafe 能力 |

整数溢出在 `--release` 下静默回绕（LLVM `add`/`sub`/`mul` 语义）；debug 下按上表 panic。
`checked_*` / `wrapping_*` / `saturating_*` / `overflowing_*` 方法由 std 子仓提供（0i6.7）。

## 编译期拒绝（不是运行时错误）

- use-after-move：`use of moved value ...`
- 借用冲突：`cannot borrow ...`（诊断顺序暂不确定，见 Ayanami-language-ppe）
- 引用存入字段/数组：`references cannot be stored ...`
- 向不可变引用赋值：`cannot assign through an immutable reference`
- 非 void 函数 `return;`：`returns \`T\` but \`return;\` has no value`

## 测试

- `tests/runtime_safety/manifest.txt`：文件、期望退出码、可选输出子串
- 覆盖：String/ArrayList 越界、空表 pop、用户 panic、整数溢出（debug panic）
- 未覆盖（平台相关，仅文档化）：除零、null、裸数组越界
