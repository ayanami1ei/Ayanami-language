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
- `.lcl` 只携带 LIR 与注解表；`.so` 由消费方编译器按目标平台现场构建并缓存。

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

`plugin package` 会校验源码至少包含一个插件声明，再调用正常打包流程。
产物与消费方编译器版本/ABI 需兼容；跨平台插件不随包分发 `.so`。

## 信任与限制

- 原生插件在编译期执行，信任级等同 proc-macro；不要运行不可信插件。
- WASM 沙箱、插件清单/权限模型为后续工作。
- 第三方 `#[transform]`（HIR 程序级变换）尚未开放；当前 `#[async]`/`#[sync]`
  是内建结构化变换。
- 属性插件当前仅 item 级；语句/字段/表达式/token 属性为后续。
