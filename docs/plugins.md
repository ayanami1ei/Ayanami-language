# 第三方插件作者指南（A5h）

Ayanami 插件是普通的 Ayanami 源码库：用 `#[macro]` / `#[pass]` / `#[check]` /
`#[attr]` 声明能力，打包为 `.lcl` 供其他项目导入。插件代码在**使用方编译期**执行，
不在运行时加载。

## 快速开始

```bash
ayanami plugin new my_plugin
cd my_plugin
ayanami plugin package .
# -> src/lib.lcl
```

脚手架生成：

```text
my_plugin/
├── ayanami.toml       # [build] target = "static-lib"
├── README.md
└── src/
    └── lib.aya        # #[attr] identity(String input) -> String
```

`src/lib.aya` 是插件项目入口；`ayanami check .` / `ayanami build .` / `ayanami package .`
都会自动识别它（无 `src/main.aya` 时回退到 `src/lib.aya`）。

## 消费插件

```ayanami
import "path/to/my_plugin/src/lib.lcl"

#[lib::identity]
fn marked() -> int { return 7 }

fn main() -> int {
    if marked() != 7 { return 1 }
    return 0
}
```

- 包名取 `.lcl` 文件 stem（`lib.lcl` → `lib`）；裸名注解也可在 import 后使用。
- 同一包内注解名跨 `macro/pass/check/attr` 唯一；跨包同名须全限定 `pkg::name`。
- `.lcl` 只携带 LIR 与注解表；本机动态库由消费方编译器按目标平台现场构建并缓存。

## 安装与分发

**发布 `.lcl`，不要发布 `.so`。** `.lcl` 是平台无关的交付物；`.so` / `.dylib` / `.dll`
只是消费方编译期的本机缓存，换平台后重新生成。

推荐在消费方项目根目录执行：

```bash
ayanami plugin install path/to/my_plugin/src/lib.lcl my_plugin
# installed plugin: <project>/plugins/my_plugin.lcl
```

安装前会校验 `.lcl` 的插件清单（ABI、最低编译器版本）；不兼容时直接拒绝。
命令会把插件复制到项目的 `plugins/`，并自动在 `ayanami.toml` 写入：

```toml
[dependencies]
my_plugin = "plugins/my_plugin.lcl"
```

之后即可 `import "my_plugin"` 并使用 `#[my_plugin::identity]`。也支持手动把 `.lcl`
放到任意位置并在 `[dependencies]` 写相对/绝对路径；`import` 使用依赖别名即可。
同名依赖已存在但路径不同时，安装会报错而不是覆盖。

## 跨平台

| 宿主 | 本机缓存 | 构建与加载 |
|---|---|---|
| Linux | `.so` | `gcc -shared -fPIC`；`dlopen`（libdl） |
| macOS | `.dylib` | `gcc`/`clang -dynamiclib -fPIC`；`dlopen`（libSystem） |
| Windows | `.dll` | MinGW `gcc -shared -Wl,--export-all-symbols`；`LoadLibraryW` |

- 链接器优先使用环境变量 `CC`，默认 `gcc`（macOS 上通常是 clang 的别名）。
- 插件作者只需发布 `.lcl`；不要按平台分别打包动态库。
- 原生插件仍需要消费方有 C 工具链。当前 `runtime.c` 以 Linux 为主；macOS/Windows 的
  动态库命名、链接参数与加载器已就绪，但 runtime 端口（POSIX/Win32）仍属编译器整体
  平台化工作，尚未验证。

## 插件清单与兼容性

`ayanami plugin package` 会在 `.lcl` 里写入 `[plugin]` 清单：

```ini
[plugin]
manifest=1
name="my_plugin"
version="0.1.0"
abi=8
min-compiler="0.6.8"
built-with="0.6.8"
```

- `abi` 必须与消费方编译器的插件 ABI 完全一致；不一致会给出“重建或安装兼容版本”
  的错误。
- `min-compiler` 使用简单 semver 比较；低于要求会拒绝加载/安装。
- 旧版 `.lcl` 没有清单时仍可加载，但每个包只提示一次重建。
- `ayanami plugin install` 在复制前执行同样校验；`import`/编译阶段也会再校验一次。

详见 [`docs/plugin-manifest.md`](plugin-manifest.md)。

## 四类插件

### 属性插件 `#[attr]`

```ayanami
#[attr]
pub fn add_pure(String input) -> String {
    return "#[pure]\n" + input
}
```

- item 级源码进出；按 item 上书写的属性顺序叠加。
- 输入是去掉当前属性后的 item 源码，其余属性保留；返回源码重新解析。
- 适合“叠加一个内建标注”“生成/改写声明”的轻量场景。

### 宏 `#[macro]`

```ayanami
#[macro]
pub fn answer() -> String {
    return "fn answer() -> int { return 42 }"
}
```

支持 item / 语句 / 表达式级；详见仓库 `docs/annotations.md` §8.2–8.5。

### MIR pass `#[pass]` / check `#[check]`

- `#[pass] fn(ref mut MirFunction) -> void`：函数内 MIR 优化，可改效应；禁止改签名。
- `#[check] fn(ref MirFunction) -> void`：只读检查，通过 `warn` / `error` 诊断。
- MIR 视图 API 见 `std/mir.aya`；示例见仓库 `example/pass_lib.aya`、`check_lib.aya`。

## 项目配置

```toml
[package]
name = "my_plugin"
version = "0.1.0"

[build]
target = "static-lib"
```

`plugin package` 会校验源码至少包含一个插件声明，再调用正常打包流程；
插件名/版本来自 `[package]`（缺失时回退到入口 stem / `0.1.0`）。
产物与消费方编译器版本/ABI 通过 `[plugin]` 清单校验；跨平台插件不随包分发 `.so`。

## 信任与限制

- 原生插件在编译期执行，信任级等同 proc-macro；不要运行不可信插件。
- 插件清单与 ABI/最低编译器校验已实现（A5j-1）；WASM 沙箱、权限模型为后续工作。
- 第三方 `#[transform]`（HIR 程序级变换）尚未开放；当前 `#[async]`/`#[sync]`
  是内建结构化变换。
- 属性插件当前仅 item 级；语句/字段/表达式/token 属性为后续。
