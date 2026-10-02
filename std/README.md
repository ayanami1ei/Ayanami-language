# Ayanami 标准库

标准库预编译为 `.lcl`，存放在编译器同目录的 `std/` 文件夹中。

使用 `import "模块名"` 自动搜索并链接。

内存模型（v0.5+）：默认所有权（非 Copy 值赋值即移动）、`unique` 独占堆指针、
`ref` / `ref mut` 借用；不再有 `shared` / `weak` / 引用计数。

## io — 输入输出

文件：`std/io.lcl`

```ayanami
import "io";

fn main() -> int {
    println("hello")            // 打印字符串（自动借用）
    println(42.to_string())     // 打印整数
    putchar(65);                // 输出字符（ASCII 码）
    c = getchar();              // 读取一个字符
    return 0
}
```

| 函数 | 说明 |
|------|------|
| `getchar() -> int` | 读取一个字符 |
| `putchar(int c)` | 输出一个字符（ASCII 码） |
| `print(ref String n)` | 输出字符串（不消费） |
| `println()` | 输出换行 |
| `println(ref String s)` | 输出字符串并换行 |

## math — 数学函数

文件：`std/math.lcl`

| 函数 | 说明 |
|------|------|
| `abs(int x)` | 绝对值 |
| `min(int a, int b)` | 最小值 |
| `max(int a, int b)` | 最大值 |
| `clamp(int x, int lo, int hi)` | 限制范围 |
| `pow(int base, int exp)` | 整数幂 |

## string — 字符串

文件：`std/string.lcl`

```ayanami
struct String {
    unique [char] data
    int len
}
```

| 方法 | 说明 |
|------|------|
| `s.len() -> int` | 长度 |
| `s.index(int i) -> char` | 按索引访问字符（也可写 `s.data[i]`） |
| `s.add(ref String other) -> String` | 拼接（`+` 运算符） |
| `s.eq(ref String other) -> bool` | 相等比较（`==`） |
| `s.copy() -> String` | 深拷贝 |
| `s.to_string() -> String` | 消费 self 并返回（ToString 接口） |
| `42.to_string()` | int/float/char/bool → String |

`ToString` 接口签名：`fn to_string(self) -> String`（消费）。

## std — 主入口

文件：`std/std.lcl`，等价于导入 `io`、`string`、`math`，并包含
`Error` 接口与 `Result[T, E]` 枚举（`try_unwrap(self)`）。

## list / arraylist / linkedlist — 集合

| 模块 | 说明 |
|------|------|
| `list` | `List[T]` 接口：`push(ref mut self, T)` / `index(ref self, int) -> T` / `to_string(ref self) -> String` / `len(ref self) -> int` / `iter(ref self, fn(T))` |
| `arraylist` | `ArrayList[T]` 顺序表：`unique [T]` 缓冲，自动扩容 |
| `linkedlist` | `LinkedList[T]`：新所有权模型下改为数组缓冲实现，接口与 `List` 一致 |

```ayanami
import "arraylist";

fn main() -> int {
    a = ArrayList[int] { data = null, len = 0, capability = 0 }
    a.push(1)
    a.push(2)
    return a.index(1)     // 2
}
```
