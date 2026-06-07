# Ayanami Language

## VSCode 插件

`ayanami-0.1.0.vsix` 位于项目根目录：

```
code --install-extension ayanami-0.1.0.vsix
```

功能：语法高亮、代码补全（关键字、结构体字段、命名空间方法、变量）、保存时错误检查。

## CLI

```
ayanami new <name>         创建项目
ayanami build [file/proj]  构建（可执行文件/静态库/动态库）
ayanami run [file/proj]    构建并运行
ayanami check <file>       前端检查（lex→parse→HIR→MIR→LIR→LLVM IR）
ayanami package <file>     打包为 .lcl（不生成可执行文件）
ayanami install <lcl>      从 .lcl 构建目标产物
ayanami clean              清除 build/ 目录
```

`build` / `run` 以项目为单位时自动查找 `ayanami.toml`。

## 项目配置（ayanami.toml）

```toml
[package]
name = "my_project"
version = "0.1.0"

[build]
target = "executable"  # 项目默认类型

[build.targets]
"src/lib.aya" = "dynamic-lib"  # 单文件覆盖
"src/utils.aya" = "static-lib"
```

优先级：单文件 → 项目默认 → `main.aya`=executable / 其余=static-lib。

## 类型系统

| 类型 | 写法 | 说明 |
|------|------|------|
| 整数 | `int` | 64 位 |
| 浮点 | `float` | 64 位 |
| 字符 | `char` | 单字节 |
| 布尔 | `bool` | `true` / `false` |
| 数组 | `unique [int]` / `shared [int]` | 堆分配，必须显式内存管理 |
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
fn add(int a, int b) -> int { return a + b; }
```
参数顺序：**类型 名称**（C 风格）。返回值用 `->`。

### 变量
```
x = 42;            // 自动推导为 int
p = Point { ... }; // 自动推导为 Point
a = unique [1,2,3]; // 自动推导为 unique [int]
```
变量通过赋值声明，类型由右侧表达式自动推断。

### 字面量
```
42          // int
3.14        // float
'a'         // char
"hello"     // string
true        // bool
false       // bool
unique [1, 2, 3]   // 数组必须带 shared/unique/weak
shared [1, 2, 3]
```

### 控制流
```
if a > b { return 1; } elif a < b { return 2; } else { return 3; }

while a < 10 { a = a + 1; }

for i in (0, 10, 1) { /* i 从 0 到 9，步长 1 */ }
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
字段用换行分隔（格式：`类型 名称`），字面量用 `字段 = 值`。

### 堆分配（shared / unique / weak）
```
a = 42;
b = shared a;   // 深拷贝到堆，引用计数
c = unique b;   // 深拷贝到堆，独占所有权
d = weak c;     // 深拷贝到堆，弱引用
```

类型修饰符：
```
fn foo(shared Point p) -> int { ... }
fn bar(unique [int] arr) -> void { ... }
```

**数组必须写 `shared`/`unique`/`weak`**，禁止裸数组。

### clone 深拷贝
```
a = unique Point { x = 10, y = 3 };
b = clone a;  // 深拷贝：新分配堆内存 + memcpy
```

- `unique T` → `unique T`（新堆分配）
- `shared T` → `shared T`（新堆分配）
- 值类型 → 按位拷贝

### 接口
```
interface Drawable {
    fn draw(shared self) -> int;
}
```
方法必须显式写 `shared self` 或 `unique self`。

### impl 和方法调用
```
impl Point {
    fn get_x(unique self) -> int { return self.x; }
}

p = Point { x = 42, y = 0 };
return p.get_x();  // 自动包装为 unique
```

### 操作符重载
通过 impl 方法重载：
```
impl Point {
    fn add(shared self, shared Point other) -> Point {
        return Point { x = self.x + other.x, y = self.y + other.y };
    }
}

c = a + b;  // 调用 add(a, b)
```

| 操作符 | 方法名 |
|--------|--------|
| `+` `-` `*` `/` `%` | `add` `sub` `mul` `div` `rem` |
| `==` `!=` `<` `>` `<=` `>=` | `eq` `ne` `lt` `gt` `le` `ge` |
| `-` `!`（一元） | `neg` `not` |
| `a[i]` | `index(self, i)` |
| `a(args)` | `call(self, args...)` |
| `expr?` | `try_unwrap(expr)` |

### 泛型

```ayanami
fn identity[T](T a) -> T { return a; }
fn max[T: Ord](T a, T b) -> T { if a > b { return a; } return b; }

struct Option[T] {
    T value
    bool has_value
}

fn main() -> int {
    return identity(42);  // 单态化为 identity_int
    a = Option[int] { value = 10, has_value = true };
}
```

泛型通过单态化实现——每个具体类型生成独立专用函数/结构体。

### null / 空指针

```ayanami
struct Node {
    int data
    shared Node next
}

fn main() -> int {
    n = shared Node { data = 42, next = null };
    if n.next == null { return 1; }
    return 0;
}
```

`null` 可用于 `shared T` 和 `unique T` 类型，通过 `== null` / `!= null` 检查。

### extern "C"

```ayanami
extern "C" fn putchar(int c);

fn main() -> int {
    putchar(65);  // 调用 C 标准库函数
    return 0;
}
```

`extern "C"` 函数使用 C ABI、不 mangling 名字、可在 `.c` 文件中实现。

### 包别名

```toml
# ayanami.toml
[dependencies]
math = "lib/math.aya"
io = "../shared/io.lcl"
```

```ayanami
import "math";   // 解析为 lib/math.aya
import "io";     // 解析为 ../shared/io.lcl
```

别名在 `ayanami.toml` 的 `[dependencies]` 中配置，import 时使用短名称。

### 标准库

`install/std/` 目录下预编译好的 `.lcl` + `.o` 文件：

```
import "io";     // print, println, putchar, getchar
import "math";   // abs, min, max, clamp, pow
import "std";    // 全部导入
```

编译器自动在自身同目录的 `std/` 中搜索标准库。

### 命名空间
```
namespace math {
    fn add(int a, int b) -> int { return a + b; }
}

fn main() -> int {
    return math::add(1, 2);  // 用 :: 访问
}
```

### 可见性
```
pub fn foo() -> int { ... }
pub(crate) struct Point { ... }
pub namespace math { ... }
fn bar() -> int { ... }  // 默认 private
```

### 导入
```
import "utils.aya";   // 同项目 .aya 文件
import "package.lcl"; // 外部包
```
`.aya` 导入自动递归编译依赖，循环导入检测。

### 函数重载
按函数名 + 参数类型列表匹配。

### 包（.lcl）
编译产物包含：LIR 二进制、符号表、目标类型、元数据。支持 `install` 命令重建。

## 编译管线

```
源码 → Lexer → Parser(AST) → HIR（类型解析/接口注册）
  → MIR（内存管理策略）→ LIR（三地址码）→ LLVM IR
  → .o（llc）→ 可执行文件/静态库/动态库（gcc）
```

## 示例

所有示例见 [`example/`](example/)。项目示例见 [`files/`](files/)。
