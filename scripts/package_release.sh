#!/usr/bin/env bash
# package_release.sh — 构建发布产物
#
# 产物：
#   ayanami-<version>-linux-<arch>.tar.gz   内含 install/（编译器 + bundled LLVM + runtime + std）
#   ayanami-<version>.vsix                  VSCode 插件（找到 vsce 时构建）
#
# 用法：./scripts/package_release.sh [--no-vsix]
set -euo pipefail
cd "$(dirname "$0")/.."

build_vsix=1
if [ "${1:-}" = "--no-vsix" ]; then
    build_vsix=0
fi

ver=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
arch=$(uname -m)
tar_name="ayanami-${ver}-linux-${arch}.tar.gz"
vsix_name="ayanami-${ver}.vsix"

echo "== 1/5 提交前检查 =="
./scripts/check_all.sh

echo "== 2/5 构建 release 二进制 =="
cargo build --release

echo "== 3/5 重建 std 预编译包 =="
for m in string io math list arraylist linkedlist std; do
    ./target/release/ayanami package "std/src/$m.aya" >/dev/null
done
cp std/src/*.lcl std/

echo "== 4/5 组装 install/ =="
if [ ! -x install/llc ]; then
    echo "缺少 install/llc（bundled LLVM）" >&2
    exit 1
fi
if [ ! -f install/libLLVM.so.21.1 ]; then
    echo "缺少 install/libLLVM.so.21.1" >&2
    exit 1
fi
cp target/release/ayanami install/ayanami
cp src/runtime.c install/runtime.c
rm -rf install/std
mkdir -p install/std
cp std/*.lcl install/std/

echo "== 5/5 打包 =="
tar czf "$tar_name" install/
echo "产物：$tar_name"

if [ "$build_vsix" = 1 ]; then
    vsce_bin=""
    if command -v vsce >/dev/null 2>&1; then
        vsce_bin=$(command -v vsce)
    else
        # 优先使用较新的缓存版本
        for b in $(ls -1 "$HOME"/.npm/_npx/*/node_modules/.bin/vsce 2>/dev/null | sort -r); do
            if [ -x "$b" ]; then
                vsce_bin=$b
                break
            fi
        done
    fi
    if [ -n "$vsce_bin" ]; then
        (cd vscode-ayanami && "$vsce_bin" package --allow-missing-repository \
            --out "../$vsix_name" >/dev/null)
        echo "产物：$vsix_name"
    else
        echo "未找到 vsce，跳过 vsix（可执行 npx --yes @vscode/vsce package）" >&2
    fi
fi

echo "打包完成"
