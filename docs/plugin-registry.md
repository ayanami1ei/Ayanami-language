# 插件 registry 与 lockfile（A5j-2 本地 MVP）

## 目标

让消费方按**插件名 + 版本**安装 `.lcl`，而不是只能传本地路径：

```bash
ayanami plugin install my_plugin@0.2.0
ayanami plugin install my_plugin          # 选择 registry 中最高版本
```

并把解析结果固定到项目根目录的 `ayanami.lock`，记录安装路径和 SHA-256，
支持离线重装/校验。当前切片先做**本地目录 registry**，不引入网络依赖。

## 非目标

本切片不做：

- 远程 HTTP registry、账号/发布权限；
- 完整 semver 范围求解（`^`/`~`/多依赖约束）；
- 私有 registry 认证、签名、TUF/供应链安全；
- 远程发布、推送权限；本地 `plugin publish` 已支持。

对应后续：A5j-2 后续切片、A5j-3 沙箱、A5j-4 跨平台 runtime。

## Registry 布局

registry 是一个目录：

```text
registry/
├── index.toml
├── my_plugin-0.1.0.lcl
└── my_plugin-0.2.0.lcl
```

`index.toml`：

```toml
[my_plugin]
0.1.0 = "my_plugin-0.1.0.lcl"
0.2.0 = "my_plugin-0.2.0.lcl"
```

- section 名是插件名（`[A-Za-z0-9_-]+`）。
- key 是版本（简单 semver，`major[.minor[.patch]]`）；value 是 `.lcl` 相对路径。
- 同名版本重复时加载报错；文件不存在/清单 name/version 不匹配时报错。
- registry 只存 `.lcl`，不存平台动态库。

## registry 位置

按优先级：

1. 环境变量 `AYANAMI_PLUGIN_REGISTRY`（目录路径）；
2. 项目 `ayanami.toml`：

   ```toml
   [plugins]
   registry = "vendor/registry"
   ```

3. 默认 `$AYANAMI_PLUGIN_HOME/registry`，其中
   `AYANAMI_PLUGIN_HOME` 缺省为 `~/.ayanami`。

registry 路径可以相对项目根目录；不存在时，如果 cache 有需要的版本，仍可离线安装。

## 缓存

缓存目录：`$AYANAMI_PLUGIN_HOME/cache/plugins/<name>/<version>.lcl`，
旁边有 `<version>.sha256` 校验文件。

安装顺序：

1. 已有 `ayanami.lock` 且版本满足请求 → 校验项目内文件 SHA-256；
2. 项目文件缺失/损坏 → 从 cache 恢复；
3. cache 缺失 → 从 registry 复制；
4. registry 不可用且 cache 无该版本 → 报错。

cache 同时保存 `.lcl` 和 `.sha256` 校验文件；lockfile 也记录安装时的 SHA-256。
不执行插件代码。

## `ayanami.lock`

项目根目录，INI 风格，便于当前无 TOML 依赖的解析器读取：

```ini
version=1

[plugin.my_plugin]
version="0.2.0"
path="plugins/my_plugin.lcl"
sha256="<64 hex>"
```

- `path` 是项目内相对路径（当前固定 `plugins/<alias>.lcl`）。
- `sha256` 是安装文件内容的 SHA-256；`plugin install`/`plugin verify` 会校验。
- 同一插件已锁定不同版本时，默认拒绝；显式 `--update` 才允许改锁。
- lockfile 不存在时，registry/path 安装继续工作；registry 安装成功后创建。

## 版本选择

当前只支持：

- `name@1.2.3`：精确版本；
- `name@1.2` / `name@1`：按缺省补 0 的精确版本；
- `name` 或 `name@*`：registry 中最高版本；
- 版本冲突（锁文件与请求不一致、registry 无版本）给出可用版本列表。

完整 semver 范围和约束求解后置；这样不会在没有 resolver 的情况下假装支持范围。

## 命令

```bash
# 作者发布到本地 registry（自动建目录/更新 index）
ayanami plugin publish path/to/lib.lcl --registry vendor/registry
ayanami plugin publish path/to/lib.lcl --registry vendor/registry --force

# registry 安装（alias 省略时使用插件名）
ayanami plugin install my_plugin@0.2.0
ayanami plugin install my_plugin@0.2.0 my_plugin

# 允许更新锁文件
ayanami plugin install my_plugin@0.3.0 --update

# 校验 lockfile 中每个文件和 SHA-256
ayanami plugin verify

# 原有路径模式保持不变
ayanami plugin install path/to/plugin.lcl my_plugin
```

## 错误诊断

至少覆盖：

- registry 未配置/不存在，且 cache 无可用版本；
- index 中无请求版本，列出该插件可用版本；
- `.lcl` 清单 name/version 与 index 不一致；
- 插件 ABI / 最低编译器不兼容（复用 A5j-1 校验）；
- lockfile 已锁定其他版本；
- 项目文件 SHA-256 与 lockfile 不一致；
- 缓存文件损坏。

## 安全边界

- registry 里的 `.lcl` 仍是编译期原生插件，不受信任插件不能安全执行；
- checksum 只保证传输/缓存完整性，不是发布者签名；
- 签名、权限声明和沙箱在 A5j-3 及后续处理。

## 验收

- 本地 registry 可按 name/version 安装，最新版本选择正确；
- lockfile 记录 version/path/sha256，重复安装命中锁定版本；
- 篡改安装文件后 `plugin verify` 失败，`plugin install` 能报错/恢复；
- registry 不可用时 cache 命中可离线安装；
- 错误信息包含插件名、请求版本、可用版本；
- 文档、单测、回归、`check_all` 全绿。

## 实现状态

- A5j-2 本地 MVP 已实现（2026-11-14）：`index.toml`、按名/版本安装、最高版本选择、
  `ayanami.lock`、SHA-256 校验、离线 cache、`plugin verify`。
- `ayanami plugin publish` 已实现（A5j-2b）：校验清单、复制 `.lcl`、更新 `index.toml`，
  同版本同文件幂等，不同文件需 `--force`。
- 远程 registry、完整 semver 范围/冲突求解仍未实现。
