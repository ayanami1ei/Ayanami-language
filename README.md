# Ayanami Language

Ayanami 是一个自带 LLVM 后端、不需要系统预装 LLVM 的编译型语言。单二进制分发，开箱即用。

## 快速安装

```bash
# 1. 下载并解压
tar xzf ayanami-0.6.5-linux-x86_64.tar.gz
cd install

# 2. 运行
./ayanami run ../example/test_struct.aya
```

`install/` 目录结构：

```
install/
├── ayanami           # 编译器本体
├── llc               # LLVM 静态编译器（bundled）
├── libLLVM.so.21.1   # LLVM 共享库
├── libedit.so.2      # libLLVM 依赖（打包兼容副本）
├── runtime.c         # 运行时（libc 包装、RC 分配器）
└── std/              # 标准库（预编译 .lcl）
    ├── string.lcl
    ├── io.lcl
    └── math.lcl
```

编译器自动在同目录查找 `llc`、`std/`、`runtime.c`。

## 从源码打包

```bash
./scripts/package_release.sh            # 生成 tar.gz 与 vsix
./scripts/package_release.sh --no-vsix  # 只生成 tar.gz
```

脚本会先跑 `check_all.sh`（版本/行数/符号地图/零告警 + 语言正负回归 + IR 快照），再构建 release 二进制、
重建 `std/` 预编译包、组装 `install/`（含 bundled `llc` 与 `libLLVM.so`）并打包。

本仓库使用子仓：`std/`（[Ayanami-std](https://github.com/ayanami1ei/Ayanami-std)）、`asuka/`、`book/`；
从源码克隆后先执行 `git submodule update --init --recursive`。

## 构建要求

- **运行时依赖**：gcc（链接）、glibc；bundled LLVM 依赖的 libedit 已随包提供
- 不需要预装 LLVM（已 bundled；安装版默认不运行 `opt` 中端优化，除非同目录提供匹配的 `opt`）
- 支持 Linux x86_64

## VSCode 插件

`ayanami-0.6.5.vsix` 位于项目根目录：

```bash
code --install-extension ayanami-0.6.5.vsix
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
version = "0.6.5"

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

| 类型   | 写法                                | 说明                                            |
| ------ | ----------------------------------- | ----------------------------------------------- |
| 整数   | `int`                             | 64 位                                           |
| 浮点   | `float`                           | 64 位                                           |
| 字符   | `char`                            | 单字节                                          |
| 布尔   | `bool`                            | `true` / `false`                            |
| 字符串 | `String`                          | 标准库结构体 `{ [char] data, int len }` |
| 数组   | `[int]`                          | 拥有堆缓冲区，离开作用域自动释放；借用写 `ref [int]` |
| 结构体 | `Point`                           | 自定义，值语义                                  |
| 枚举   | `Option[T]`                       | tag + union，支持方法派发                       |
| ref    | `ref int` / `ref mut int`         | 借用，不拥有、不可逃逸                          |

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
t = s + " World"          // 拼接（String 与任意 ToString 类型，自动借用）
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
    fn get(ref self) -> T { return self._0 }
}
// e.method() 自动按 tag 派发到对应变体的实现
println(x.get())  // 自动调用 Option_Some::get
```

枚举内存布局：`{ _tag: int, _data_V0: ..., _data_V1: ... }`，tag 决定当前活跃变体。
method 调用自动生成 `match e { V0 => e._data_V0.method(...), V1 => ... }`。

### 接口与 impl

```
interface ToString {
    fn to_string(self) -> String;
}

impl Point {
    // ref self 借用，不消费原值
    fn to_string(ref self) -> String {
        return "(" + self.x + ", " + self.y + ")"
    }
}

// 接口采用 Go 式结构匹配：方法名/形参/返回类型齐全即自动实现接口，
// 无需（也不支持）`impl Point: ToString {}` 这类显式声明；self 关键字不参与匹配。
```

方法接收者：`self` 消费、`ref self` 借用、`ref mut self` 可变借用（原语类型用 `self`，因为 Copy）。

### 所有权与借用

- **默认所有权**：非 Copy 值在赋值/传参时移动（use-after-move 会报错）；Copy 类型为
  `int` / `float` / `char` / `bool`（及函数指针）。
- **拥有堆数组**：`[T]` / `[T; n]` 为拥有堆缓冲，移动语义，离开作用域递归释放（借用写 `ref [T]`）。
- **`ref T` / `ref mut T`**：借用，不拥有；调用时对同类型左值自动借用。引用可存入局部变量，借用在其最后一次使用后失效（NLL）；可作为返回值（生命周期省略：恰好一个引用参数时，返回引用视为来自该参数）；仍不可存入字段/数组。
- 引用的**自动解引用**：值上下文（运算、比较、传值形参、返回）中 `ref T` 自动读出 `T`；`ref mut T` 目标赋值（`i = v`）穿透引用写回被借用变量；写入 `ref T`（不可变）报错。
- **接口**：`ref Shape` 是借用胖指针（不分配），`Shape` 是拥有所有权的胖指针。
- 没有 `shared` / `weak` / GC：需要共享数据时用借用，或显式 `.copy()`。

字符串拼接 `add[T:ToString](ref self, T a)` 对 String 与任意 ToString 类型生效：

```
s = "hello"
println(s + " world")     // String + String
println("val: " + 42)     // String + int（自动调用 to_string）
```

### 泛型

```ayanami
fn identity[T](T a) -> T { return a; }
fn max[T: Ord](T a, T b) -> T { if a > b { return a; } return b; }
```

泛型通过单态化实现。约束使用接口名。

函数宏用 `#name(args)` 调用（表达式级，编译期展开，可捕获调用点）：

```ayanami
import "panic";
#panic("boom");        // runtime error: boom --> file.aya:行:列，退出码 101
```

命名空间函数可用显式泛型实参调用：`ns.fn[T1, T2](args)`（普通调用仍靠形参推导；显式实参优先）。
标准库构造函数即此形式，例如 `ArrayList::new[int]()`、`ArrayList::with_capacity[String](8)`、
`LinkedList::new[int]()`、`String::empty()` / `String::new(buf, len)`。

### 操作符重载

| 操作符                                  | 方法名                                    |
| --------------------------------------- | ----------------------------------------- |
| `+` `-` `*` `/` `%`           | `add` `sub` `mul` `div` `rem`   |
| `==` `!=` `<` `>` `<=` `>=` | `eq` `ne` `lt` `gt` `le` `ge` |
| `&` `\|` `^` `<<` `>>`           | `bitand` `bitor` `bitxor` `shl` `shr` |
| `-` `!` `~`（一元）              | `neg` `not` `bitnot`              |
| `a[i]`                                | `index(self, i)`                        |
| `expr?`                               | `try_unwrap(expr)`                      |

## 编译管线

```
源码 → Lexer → Parser(AST) → HIR → MIR → LIR → LLVM IR → .o → 可执行文件
```

## 未来方向

### 标注式编程（Attributes）

用简短的标注向编译器声明意图与契约，驱动代码生成、优化、条件判定与效应检查。
设计文档：[`docs/annotations.md`](docs/annotations.md)。示例：

```ayanami
#[inline]
fn add(int a, int b) -> int { return a + b }

#[pure]
extern "C" fn strlen(unique [char] s) -> int;
```

### 效应系统

引入代数效应（algebraic effects），将副作用（IO、可变状态、异常、非确定性等）纳入类型系统。函数通过 `eff`/`#[throws]` 声明其可能产生的效应，编译器静态确保效应处理（见标注系统 A3）。

### 代码设计平台

将编译器从离线工具转变为交互式开发平台的核心：

| 阶段             | 能力                                                                                                             |
| ---------------- | ---------------------------------------------------------------------------------------------------------------- |
| **开发期** | 解释执行某个 IR（待定：HIR / MIR / LIR），无需完整编译即可运行代码片段，获得类似 Python 的即时反馈与动态调试体验 |
| **发布期** | 一次性编译为优化机器码，产物不含解释器或动态能力，用户侧零运行时开销                                             |

解释 IR 时可保留完整的类型信息与源码映射，支持断点、求值、热重载等 IDE 特性。发布路径则走现有 LLVM 后端生成原生二进制。

### 核心原则

> 开发体验向动态语言看齐，部署产物向静态语言看齐。

## 教程

《Ayanami 语言教程》（Rust 圣经风格，17 章）位于子仓 [`book/`](book/)：

- 仓库：<https://github.com/ayanami1ei/ayanami-language-book>
- 内容：安装、猜数字、基础语法、结构体、枚举、接口、泛型、所有权、数组、
  标准库、错误处理、标注系统、生命周期、FFI / 汇编 / 宏、工具链、综合项目
- 本地构建：`cd book && mdbook build`（或 `mdbook serve --open`）

## 示例

见 [`example/`](example/)（26 个端到端用例）。

## CLI 输出（cargo 风格）

```
   Compiling main
    Finished in 0.12s
     Running `build/main`
```

- 状态词绿色加粗（TTY 下），`NO_COLOR` 关闭颜色
- 退出码 0 不打印；非 0 时红色 `error: process didn't exit successfully: ... (exit code: N)` 并以该码退出
- 运行时 panic（红色，Rust 风格）：

```
thread 'main' panicked at main.aya:4:5:
boom
```
