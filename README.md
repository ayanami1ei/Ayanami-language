# Ayanami Language

## CLI

```
ayanami new <name>      创建项目
ayanami check <file>    前端检查（lex → parse → HIR → MIR → LIR → LLVM IR）
ayanami build <file>    构建可执行文件 + .lcl 包
ayanami run <file>      构建并运行
```

## 类型系统

| 类型 | 写法 | 说明 |
|------|------|------|
| 整数 | `int` | 64 位 |
| 浮点 | `float` | 64 位 |
| 字符 | `char` | 单字节 |
| 布尔 | `bool` | `true` / `false` |
| 数组 | `unique [int]` / `shared [int]` | 堆分配，必须显式写内存管理，`arr[0]` 索引 |
| 结构体 | `Point` | 自定义，值语义 |
| shared | `shared int` | 引用计数指针 |
| unique | `unique int` | 独占所有权指针 |
| weak | `weak int` | 弱引用 |

## 语法

### 注释
```
// 行注释
/* 块注释 */
```

### 函数
```
fn add(int a, int b) -> int {
    return a + b;
}
```

参数顺序：**类型 名称**（C 风格）。返回值用 `->`。

### 变量
```
x = 42;            // 类型自动推导为 int
p = Point { ... }; // 类型自动推导为 Point 结构体
arr = [1, 2, 3];   // 类型自动推导为 [int]
```

所有变量必须通过赋值声明，类型由右侧表达式推断。

### 字面量
```
42         // int
3.14       // float
'a'        // char
"hello"    // string
true       // bool
false      // bool
unique [1, 2, 3]   // array，必须带 shared/unique/weak
shared [1, 2, 3]   // array，引用计数
```

### 控制流
```
// if / elif / else
if a > b {
    return 1;
} elif a < b {
    return 2;
} else {
    return 3;
}

// while
while a < 10 {
    a = a + 1;
}

// for (desugars to while)
for i in (0, 10, 1) {
    // i from 0 to 9, step 1
}
```

### 结构体
```
struct Point {
    int x
    int y
}

p = Point { x = 10, y = 3 };
return p.x;
```

结构体默认值语义（栈分配）。函数传参和返回值也按值传递。

### 堆分配（shared / unique / weak）
```
a = 42;
b = shared a;   // 深拷贝到堆，引用计数
c = unique b;   // 深拷贝到堆，独占所有权
d = weak c;     // 深拷贝到堆，弱引用
```

`shared` / `unique` / `weak` 也可作为类型修饰符：
```
fn foo(shared Point p) -> int { ... }
fn bar(unique [int] arr) -> void { ... }
```

对于值类型（`int`, `float`, `char`, `bool`），`shared` / `unique` 在 LLVM 层面无开销（仍然是传值）。

对于堆类型（结构体、接口 fat pointer），堆分配通过 `malloc` + `memcpy` 深拷贝。

**数组必须显式带上 `shared` / `unique` / `weak`**，禁止裸数组（值语义）：

```
a = unique [1, 2, 3];    // ✓ 正确
b = shared [1, 2, 3];    // ✓ 正确
c = [1, 2, 3];            // ✗ 编译错误
```

### 接口
```
interface Drawable {
    fn draw(shared self) -> int;
    fn resize(unique self, int w, int h) -> void;
}
```

接口方法必须显式写 `shared self` 或 `unique self`。

### 实现（impl）
```
impl Point {
    fn get_x(unique self) -> int {
        return self.x;
    }
}

p = Point { x = 42, y = 0 };
return p.get_x();
// 等价于自动包装为 unique：
// return get_x(unique p);
```

### 方法调用语法糖
```
obj.method(args)
```
- 如果 obj 是接口类型 → 虚函数表动态分派
- 如果 obj 是具体类型 → 普通函数调用
- receiver 若为纯值类型而方法需要 `unique`/`shared` → 自动包装

### 可见性
```
pub fn foo() -> int { ... }
pub(crate) struct Point { int x }
pub namespace math { ... }
fn bar() -> int { ... }   // 默认 private
```

`pub` / `pub(crate)` / 默认 private，与 Rust 一致。可在 `fn`、`struct`、`namespace` 前使用。
当前可见性仅标记，不做访问控制检查。

### 命名空间
```
pub namespace math {
    fn abs(int x) -> int {
        if x < 0 { return -x; }
        return x;
    }
}
```

### 函数重载
按函数名 + 参数类型列表匹配，支持重载。

### 包（.lcl）
编译产物为 `.lcl` 格式，包含 LIR 和元数据。

## 包（.lcl）

编译时自动在可执行文件旁生成 `.lcl` 包文件，包含公开符号和 LIR。

### 格式

**二进制头（12 字节）：**
| 偏移 | 大小 | 内容 |
|------|------|------|
| 0    | 4    | 魔术字 `LCL1` |
| 4    | 4    | 版本号（u32 LE） |
| 8    | 4    | 标志位（u32 LE） |

**UTF-8 INI 正文：**

```
[pakage-info]
name="my_lib"
version="0.1.0"

[target]
type="static-lib"
type="dynamic-lib"

# 有 main 函数时为：
# type="executable"

[symbols]
fn="foo,foo(int,float)->int"
fn="bar,bar()->void"
struct="Point"
namespace="math"

[generics]
# 泛型源代码（预留）

[lir]
# LIR 文本
```

- `[symbols]`：只包含 `pub` / `pub(crate)` 的公开符号
- `[generics]`：预留，未来泛型会把全部源码放入
- `[lir]`：编译后的 LIR 文本

## 编译管线

```
源码 → Lexer → Parser(AST) → HIR → MIR → LIR → LLVM IR → .o → 可执行文件 + .lcl
```

## 示例

所有完整示例见 [`example/`](example/)。
