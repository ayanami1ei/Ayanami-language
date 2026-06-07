# Ayanami 标准库

标准库预编译为 `.lcl` + `.o`，存放在编译器同目录的 `std/` 文件夹中。

使用 `import "模块名"` 自动搜索并链接。

## io — 输入输出

文件：`std/io.lcl`

```ayanami
import "io";

fn main() -> int {
    print(42);          // 输出整数
    println();          // 输出换行
    putchar(65);        // 输出字符（ASCII 码）
    c = getchar();      // 读取一个字符
    return 0;
}
```

| 函数 | 说明 |
|------|------|
| `getchar()` | 读取一个字符，返回 int |
| `putchar(int c)` | 输出一个字符（ASCII 码） |
| `print(int n)` | 输出整数 |
| `println()` | 输出换行 |

## math — 数学函数

文件：`std/math.lcl`

```ayanami
import "math";

fn main() -> int {
    print(abs(-42));     // 42
    print(min(3, 7));    // 3
    print(max(3, 7));    // 7
    print(clamp(5, 0, 3));  // 3
    print(pow(2, 10));      // 1024
    println();
    return 0;
}
```

| 函数 | 说明 |
|------|------|
| `abs(int x)` | 绝对值 |
| `min(int a, int b)` | 最小值 |
| `max(int a, int b)` | 最大值 |
| `clamp(int x, int lo, int hi)` | 限制范围 |
| `pow(int base, int exp)` | 整数幂 |

## 多个模块

```ayanami
import "io";
import "math";

fn main() -> int {
    print(abs(-42));
    println();
    return 0;
}
```
