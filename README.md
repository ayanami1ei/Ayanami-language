# Ayanami Language

Ayanami 是一个自带 LLVM 后端、不需要系统预装 LLVM 的编译型语言。单二进制分发，开箱即用。

## 快速安装

```bash
# 1. 下载并解压
tar xzf ayanami-0.1.0-linux-x86_64.tar.gz
cd install

# 2. 运行
./ayanami run ../example/hello.aya
```

`install/` 目录结构：

```
install/
├── ayanami           # 编译器本体
├── llc               # LLVM 静态编译器（bundled）
├── libLLVM.so.21.1   # LLVM 共享库
├── runtime.c         # 运行时（libc 包装、RC 分配器）
└── std/              # 标准库（预编译 .lcl）
    ├── string.lcl
    ├── io.lcl
    └── math.lcl
```

编译器自动在同目录查找 `llc`、`std/`、`runtime.c`。

## 构建要求

- **运行时依赖**：gcc（链接）、glibc
- 不需要预装 LLVM（已 bundled）
- 支持 Linux x86_64

## VSCode 插件

`ayanami-0.1.0.vsix` 位于项目根目录：

```bash
code --install-extension ayanami-0.1.0.vsix
```

功能：语法高亮、代码补全（结构体字段、方法、变量类型推断）、保存/打开时错误检查、hover 文档、Run CodeLens。

## CLI

```bash
ayanami new <name>         创建新项目
ayanami check [--watch] <file/proj>  前端检查；--watch 监听文件变更自动重检
ayanami fmt [file]         格式化代码
ayanami package <file/proj> 打包为 .lcl（不生成可执行文件）
ayanami build [file/proj]  构建可执行文件 + .lcl 包
ayanami run [file/proj]    构建并运行
ayanami install <lcl>      从 .lcl 构建目标产物
ayanami defs <file>        输出符号定义列表（JSON）
ayanami clean              清除 build/ 目录
```

项目模式下自动查找 `ayanami.toml`。

## 项目配置（ayanami.toml）

```toml
[package]
name = "my_project"
version = "0.1.0"

[build]
target = "executable"

[build.targets]
"src/lib.aya" = "dynamic-lib"
"src/utils.aya" = "static-lib"
```

## Hello World

```ayanami
import "io"

fn main() -> int {
    println("Hello, World!")
    println("value: " + 42)       // String + int via ToString
    println(1.to_string() + " + " + 2 + " = " + 3)
    return 0
}
```

## 类型系统

| 类型 | 写法 | 说明 |
|------|------|------|
| 整数 | `int` | 64 位 |
| 浮点 | `float` | 64 位 |
| 字符 | `char` | 单字节 |
| 布尔 | `bool` | `true` / `false` |
| 字符串 | `String` | 标准库结构体 `{ unique [char] data, int len }` |
| 数组 | `unique [int]` / `shared [int]` | 堆分配，必须显式内存管理 |
| 结构体 | `Point` | 自定义，值语义 |
| 枚举 | `Option[T]` | tag + union，支持方法派发 |
| shared | `shared int` | 引用计数指针 |
| unique | `unique int` | 独占所有权指针 |
| weak | `weak int` | 弱引用 |

| 枚举 | `Color` | tag + union，支持方法派发 |

## 标准库

```ayanami
import "string"   // String 结构体、ToString 接口、int/float/char/bool to_string
import "io"       // print, println, putchar, getchar
import "math"     // abs, min, max, clamp, pow
```

### String

```ayanami
s = "Hello"
t = s + " World"          // 拼接（支持 shared/unique String + 任意 ToString 类型）
println(s.len())          // 长度
println(s.copy())         // 深拷贝
if s.eq(t) { ... }        // 相等比较
s2 = 42.to_string()       // int → String
s3 = 3.14.to_string()     // float → String
println("val: " + 42)     // String + int → 自动调用 to_string
```

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
参数顺序：**类型 名称**。返回值用 `->`。

### 函数指针
```
fn apply(int x, int y, fn(int,int)->int f) -> int {
    return f(x, y)
}

fn add(int a, int b) -> int { return a + b; }

fn main() -> int {
    f = add           // 函数名自动转为函数指针
    return apply(3, 4, f)
}
```
函数指针类型 `fn(T) -> U`，函数名可直接赋值给函数指针变量。

### 变量
```
x = 42;                   // int
f = 3.14;                 // float
c = 'X';                  // char
b = true;                 // bool
s = "hello";              // String
p = Point { x = 1, y = 2 };
```

### 控制流
```
if a > b { return 1; } elif a < b { return 2; } else { return 3; }
while a < 10 { a = a + 1; }
for i in (0, 10) { /* i: 0..9 */ }
match x { V(v) => expr, W => expr }   // 枚举模式匹配
break / continue                      // 循环控制
```

### 结构体
```
struct Point {
    int x
    int y
}
p = Point { x = 10, y = 3 };
```

### 枚举
```
enum Option[T] {
    Some(T),
    None,
}

x = Option::Some(42)

// `_tag` 字段访问判别值
println(x._tag)               // 0

// `_data_V` 访问变体数据
println(x._data_Some._0)     // 42

// `match` 按 tag 分支
match x {
    Some(v) => println("" + v),
    None => println("none"),
}

// 变体方法
impl Option_Some[T] {
    fn get(shared self) -> T { return self._0 }
}
// e.method() 自动按 tag 派发到对应变体的实现
println(x.get())  // 自动调用 Option_Some::get
```

枚举内存布局：`{ _tag: int, _data_V0: ..., _data_V1: ... }`，tag 决定当前活跃变体。
method 调用自动生成 `match e { V0 => e._data_V0.method(...), V1 => ... }`。

### 接口与 impl
```
interface ToString {
    fn to_string(unique self) -> unique String;
}

impl Point {
    // shared self 适合 to_string（不消费原值）
    fn to_string(shared self) -> unique String {
        return "(" + self.x + ", " + self.y + ")"
    }
}

impl Point: ToString {}  // 结构匹配：有 to_string 方法即自动实现接口
```

方法必须写 `shared self` / `unique self`。`shared self` 借用，`unique self` 消费。

### shared / unique / weak

| 所有权 | 说明 |
|--------|------|
| `shared T` | 引用计数指针，可共享，自动释放 |
| `unique T` | 独占所有权指针，移动语义，离开作用域自动释放 |
| `weak T` | 弱引用，不增加引用计数，用于遍历 |

`shared T` 可传入接受 `T` 或 `shared T` 参数的函数。  
`unique T` 可传入接受 `T` 或 `unique T` 参数的函数。

字符串拼接 `add[T:ToString](T a)` 支持 `unique T` 和 `shared T`：

```
s = shared String { data = "hello", len = 5 }  // 共享字符串
println(s + " world")                           // 可用在拼接中
```

### 泛型
```ayanami
fn identity[T](T a) -> T { return a; }
fn max[T: Ord](T a, T b) -> T { if a > b { return a; } return b; }
```
泛型通过单态化实现。约束使用接口名。

### 操作符重载

| 操作符 | 方法名 |
|--------|--------|
| `+` `-` `*` `/` `%` | `add` `sub` `mul` `div` `rem` |
| `==` `!=` `<` `>` `<=` `>=` | `eq` `ne` `lt` `gt` `le` `ge` |
| `-` `!`（一元） | `neg` `not` |
| `a[i]` | `index(self, i)` |
| `expr?` | `try_unwrap(expr)` |

## 编译管线

```
源码 → Lexer → Parser(AST) → HIR → MIR → LIR → LLVM IR → .o → 可执行文件
```

## 示例

见 [`example/`](example/) 和 [`files/`](files/)。
