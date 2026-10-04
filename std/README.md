# Ayanami 标准库

标准库预编译为 `.lcl`，存放在编译器同目录的 `std/` 文件夹中。

使用 `import "模块名"` 自动搜索并链接。

内存模型（v0.5+）：默认所有权（非 Copy 值赋值即移动）、`[T]`/`[T; n]` 拥有堆数组、
`ref` / `ref mut` 借用；不再有 `unique` / `shared` / `weak` / 引用计数。

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
| `print(int n)` / `print(float f)` / `print(bool b)` / `print(char c)` | 输出常用类型（重载） |
| `println()` | 输出换行 |
| `println(ref String s)` | 输出字符串并换行 |
| `println(int/float/bool/char)` | 输出常用类型并换行（重载） |

```ayanami
println(42)       // 整数
println(3.5)      // 浮点
println(true)     // 布尔
println('x')      // 字符
```

## math — 数学函数

文件：`std/math.lcl`

| 函数 | 说明 |
|------|------|
| `abs(int x)` | 绝对值 |
| `min(int a, int b)` | 最小值 |
| `max(int a, int b)` | 最大值 |
| `clamp(int x, int lo, int hi)` | 限制范围 |
| `pow(int base, int exp)` | 整数幂 |
| `abs/min/max/clamp(float, ...)` | 浮点重载（同名） |
| `pow(float base, int exp)` | 浮点幂 |
| `sqrt(float x)` | 平方根 |
| `floor(float x)` / `ceil(float x)` | 向下/向上取整 |

## string — 字符串

文件：`std/string.lcl`

```ayanami
struct String {
    [char] data
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
| `s.is_empty() -> bool` | 是否空串 |
| `s.contains(ref String) -> bool` | 是否包含子串 |
| `s.index_of(ref String) -> int` | 子串位置（未找到 -1） |
| `s.starts_with(ref String) -> bool` / `s.ends_with(ref String) -> bool` | 前缀/后缀 |
| `s.substring(int start, int end) -> String` | 区间子串（越界自动夹取） |
| `s.trim() -> String` | 去首尾空白 |
| `s.to_upper() -> String` / `s.to_lower() -> String` | 大小写转换 |
| `s.repeat(int n) -> String` | 重复拼接 |
| `s.parse_int() -> int` | 十进制解析（非法输入返回 0） |
| `s.is_int() -> bool` | 是否为合法十进制整数 |

字符（`char`）工具：

| 方法 | 说明 |
|------|------|
| `c.is_digit()` / `c.is_alpha()` / `c.is_alnum()` / `c.is_space()` | 分类判断 |
| `c.is_upper()` / `c.is_lower()` | 大小写判断 |
| `c.to_upper() -> char` / `c.to_lower() -> char` | 大小写转换 |
| `c.to_digit() -> int` | 数字字符 → 数值 |

`ToString` 接口签名：`fn to_string(self) -> String`（消费）。

## std — 主入口

文件：`std/std.lcl`，等价于导入 `io`、`string`、`math`，并包含
`Error` 接口、`Result[T, E]`（`try_unwrap(self)`）与 `Option[T]`
（`unwrap_or(self, default)` / `is_some(self)`）枚举。

## list / arraylist / linkedlist — 集合

| 模块 | 说明 |
|------|------|
| `list` | `List[T]` 接口：`push(ref mut self, T)` / `index(ref self, int) -> T` / `to_string(ref self) -> String` / `len(ref self) -> int` / `iter(ref self, fn(T))` |
| `arraylist` | `ArrayList[T]` 顺序表：`[T]` 缓冲，自动扩容；另有 `set(i, v)` / `pop()` / `is_empty()` / `clear()` |
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
