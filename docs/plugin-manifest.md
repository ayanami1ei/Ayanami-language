# 插件清单与兼容性校验（A5j-1）

## 目标

给第三方插件 `.lcl` 增加一个随包携带的清单，让消费方在**加载/安装时**就能判断：

- 这是不是插件，插件名和版本是什么；
- 插件需要哪个插件 ABI；
- 插件要求的最低编译器版本；
- 插件由哪个编译器版本打出。

目标是替代当前只会在导入时打印 version drift warning、ABI 不兼容则等到构建或
`dlopen` 才失败的行为。

## 非目标

本切片不做：

- 中心化 registry、版本解析、lockfile；
- 权限声明、沙箱、WASM 后端；
- 第三方 `#[transform]` 的 HIR schema。

这些分别由 bd `A5j-2`、`A5j-3`、`A5j-4` 和 A5e 跟踪。

## `.lcl` 清单格式

沿用 `.lcl` 的 INI 头，在 `[pakage-info]` 后增加 `[plugin]`：

```ini
[pakage-info]
name="my_plugin"
version="0.6.8"

[plugin]
manifest=1
name="my_plugin"
version="0.1.0"
abi=8
min-compiler="0.6.8"
built-with="0.6.8"
```

字段：

| 字段 | 含义 |
|---|---|
| `manifest` | 清单 schema 版本；当前只支持 `1` |
| `name` | 插件名（展示/诊断用） |
| `version` | 插件自身版本 |
| `abi` | 插件 ABI；必须与消费方编译器的插件 ABI 完全一致 |
| `min-compiler` | 最低编译器版本，简单 semver（`major.minor.patch`，缺省段按 0） |
| `built-with` | 打出该 `.lcl` 的编译器版本，仅用于诊断 |

`[pakage-info] name/version` 保持 `.lcl` 现有语义：`version` 仍记录打包该包的编译器
版本，避免破坏 `warn_version_drift`；插件自身版本以 `[plugin]` 为准。

## 兼容规则

加载任何 `.lcl` 时：

1. 有 `[plugin]`：
   - `manifest` 不等于 1 → 报错；
   - `abi` 不等于当前插件 ABI → 报错，并提示重建/安装兼容版本；
   - `min-compiler` 高于当前编译器 → 报错，并给出插件名、插件版本、要求版本、实际版本；
   - 其他字段缺失或格式非法 → 报错。
2. 没有 `[plugin]`，但包导出了 `macro/pass/check/attr` 注解：
   - 视为 A5j 之前的旧插件，保持可加载；
   - 每个包路径只打印一次 warning，提示用新版 `ayanami plugin package` 重建。
3. 没有 `[plugin]` 且没有插件注解：
   - 普通 `.lcl`，不检查、不告警。

`ayanami plugin install <lcl>` 在复制/写配置前先做同样的清单校验，避免把不兼容插件
装进项目。

## 打包

`ayanami plugin package <project>`：

- 从项目 `ayanami.toml` 的 `[package] name/version` 读取插件名/版本；
- 缺少时回退到入口文件 stem 和 `0.1.0`；
- 自动写入当前 `manifest=1`、插件 ABI、`min-compiler=当前编译器版本`、
  `built-with=当前编译器版本`；
- 普通 `ayanami package` 不写 `[plugin]`，行为不变。

## 错误示例

```text
plugin 'my_plugin' 0.1.0 requires plugin ABI 7, compiler provides 8;
rebuild the plugin with this compiler or install a compatible version
```

```text
plugin 'my_plugin' 0.1.0 requires ayanami >= 0.7.0, current 0.6.8
```

## 实现位置

- `src/package/plugin.rs`：清单解析、校验、版本比较、旧插件告警；
- `src/package/bytes.rs`：序列化 `[plugin]`；
- `src/package/load.rs`：加载时校验；
- `src/compiler/build/package.rs` / `src/cli/plugin.rs`：打包与安装入口；
- `src/compiler/macro_expand/plugin_abi.rs`：ABI 常量单一来源。

## 验收

- 新打的插件 `.lcl` 含 `[plugin]`，ABI/最低版本校验生效；
- ABI 或 `min-compiler` 不兼容时给出含名称/版本/要求的错误；
- 旧插件仍能加载，只告警一次；
- `plugin install` 提前拒绝不兼容插件；
- 回归、单测和 `./scripts/check_all.sh` 全绿。

## 实现状态

- A5j-1 已实现（2026-11-14）：`.lcl` `[plugin]` 序列化、加载/安装校验、旧插件一次性
  warning、`plugin package` 自动写入清单；单测与回归见 `scripts/regression.sh`。
- A5j-2 registry / A5j-3 沙箱 / A5j-4 runtime 端口仍未实现。
