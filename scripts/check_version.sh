#!/usr/bin/env bash
# check_version.sh — 校验版本号一致性
#
# 规则（见 AGENTS.md「版本号同步」）：
#   1. Cargo.toml 的 version 是权威版本号；
#   2. std/ayanami.toml、vscode-ayanami/package.json 必须与它完全一致；
#   3. Cargo.toml 不得低于最新发布标签 vX.Y.Z（领先则提示，发布时打标签）。
#
# 用法: ./scripts/check_version.sh   # 一致退出 0，不一致退出 1
set -euo pipefail
cd "$(dirname "$0")/.."

cargo_ver=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
std_ver=$(sed -n 's/^version = "\(.*\)"/\1/p' std/ayanami.toml | head -1)
ext_ver=$(sed -n 's/.*"version": "\(.*\)".*/\1/p' vscode-ayanami/package.json | head -1)
tag_ver=$(git describe --tags --abbrev=0 --match 'v[0-9]*' 2>/dev/null | sed 's/^v//' || true)

if [[ -z "$cargo_ver" ]]; then
    echo "error: 无法从 Cargo.toml 读取版本号" >&2
    exit 1
fi

fail=0
check() { # 描述 实际值 期望值
    if [[ "$2" != "$3" ]]; then
        echo "error: $1 版本不一致: $2（应为 $3）" >&2
        fail=1
    fi
}

check "std/ayanami.toml" "$std_ver" "$cargo_ver"
check "vscode-ayanami/package.json" "$ext_ver" "$cargo_ver"

if [[ -n "$tag_ver" ]]; then
    newest=$(printf '%s\n%s\n' "$cargo_ver" "$tag_ver" | sort -V | tail -1)
    if [[ "$newest" == "$tag_ver" && "$cargo_ver" != "$tag_ver" ]]; then
        echo "error: Cargo.toml 版本 $cargo_ver 低于最新发布标签 v$tag_ver" >&2
        fail=1
    elif [[ "$cargo_ver" != "$tag_ver" ]]; then
        echo "note: Cargo.toml 版本 $cargo_ver 领先于最新标签 v$tag_ver，发布时记得打新标签"
    fi
else
    echo "note: 未找到 vX.Y.Z 标签，跳过标签校验" >&2
fi

if [[ $fail -ne 0 ]]; then
    exit 1
fi
echo "版本一致: $cargo_ver"
