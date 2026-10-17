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

# std 子仓：打包前检查（落后/未提交提示；不自动 pull，避免动用户工作区）
if git -C std rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    git -C std fetch -q origin 2>/dev/null || true
    behind=$(git -C std rev-list --count HEAD..origin/main 2>/dev/null || echo 0)
    if [ "$behind" != "0" ]; then
        echo "警告：std 子仓落后 origin/main $behind 个提交（建议先 git -C std pull --ff-only）" >&2
    fi
    if [ -n "$(git -C std status --porcelain 2>/dev/null)" ]; then
        echo "警告：std 子仓有未提交改动，请确认后再打包" >&2
    fi
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
# 直接安装新鲜构建产物到 std/：此前用 `find std/src -name '*.lcl'` 回拷会覆盖成
# 旧提交产物（std/src/** 的历史 .lcl），导致发布 std 陈旧（如缺 iter 的自由 map）
AYANAMI_BIN="$PWD/target/release/ayanami" ./std/scripts/build.sh --install "$PWD/std"

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
# 安装版 runtime：真实 runtime.c + runtime/ 子目录随包分发
# （编译器按“可执行文件同目录 runtime.c”查找；`#include "runtime/sys.c"` 相对其目录解析）
cp std/runtime.c install/runtime.c
mkdir -p install/runtime
cp std/runtime/abi.h std/runtime/sys.c install/runtime/
# bundled libLLVM 依赖 libedit.so.2；部分发行版只有 .so.0，打包兼容副本以自包含
if [ ! -e install/libedit.so.2 ]; then
    libedit_src=""
    for cand in /usr/lib/libedit.so.2 /usr/lib/x86_64-linux-gnu/libedit.so.2 \
                /usr/lib/libedit.so.0 /usr/lib/x86_64-linux-gnu/libedit.so.0; do
        if [ -e "$cand" ]; then
            libedit_src=$cand
            break
        fi
    done
    if [ -n "$libedit_src" ]; then
        cp -L "$libedit_src" install/libedit.so.2
        echo "已打包 libedit 兼容副本：$libedit_src -> install/libedit.so.2"
    else
        echo "警告：未找到 libedit，bundled llc 可能无法运行" >&2
    fi
fi
rm -rf install/std
mkdir -p install/std
cp std/*.lcl install/std/

# 运行时依赖打包（跨发行版）：llc/libLLVM 的非基线依赖按 soname 放入 install/
# 基线 glibc：2.39（Ubuntu 24.04 / Debian 13 / Arch）；高于基线的库优先从基线容器取。
BASELINE_GLIBC="2.39"
glibc_need() { objdump -T "$1" 2>/dev/null | grep -o 'GLIBC_[0-9]\+\.[0-9]\+' | sort -Vu | tail -1 | sed 's/GLIBC_//'; }
glibc_too_new() { [ -n "$1" ] && [ "$(printf '%s\n%s\n' "$BASELINE_GLIBC" "$1" | sort -V | tail -1)" != "$BASELINE_GLIBC" ]; }

# 1) 基线库（libedit/libncurses 等）优先从 ubuntu:24.04 容器取
fetched_baseline=0
if command -v docker >/dev/null 2>&1; then
    # 镜像可配（默认 ubuntu:24.04；国内镜像源可用 AYANAMI_BASELINE_IMAGE 覆盖）
    for img in "${AYANAMI_BASELINE_IMAGE:-ubuntu:24.04}" "docker.m.daocloud.io/library/ubuntu:24.04"; do
        if docker run --rm -v "$PWD/install:/out" "$img" bash -c \
            'apt-get -o Acquire::http::Dl-Limit=10240 -o Acquire::https::Dl-Limit=10240 update -qq >/dev/null 2>&1 && apt-get -o Acquire::http::Dl-Limit=10240 -o Acquire::https::Dl-Limit=10240 install -y -qq libedit2 libncurses6 libbsd0 >/dev/null 2>&1 && \
             for f in libedit.so.2 libncursesw.so.6 libtinfo.so.6 libbsd.so.0 libmd.so.0; do \
                 if [ -e "/usr/lib/x86_64-linux-gnu/$f" ]; then cp -L "/usr/lib/x86_64-linux-gnu/$f" /out/; fi; \
             done && \
             if [ -e /usr/lib/x86_64-linux-gnu/libedit.so.2 ]; then cp -L /usr/lib/x86_64-linux-gnu/libedit.so.2 /out/libedit.so.0; fi' \
            >/dev/null 2>&1; then
            echo "已从 $img 取基线库（libedit/libncurses/libbsd）"
            fetched_baseline=1
            break
        fi
    done
fi
if [ "$fetched_baseline" = 0 ]; then
    echo "警告：无法从基线容器取库（docker/网络不可用），沿用本机库（仅基线 glibc 兼容发行版可用）" >&2
fi

# 2) 收集 llc + libLLVM + 已打包库的依赖（递归），按 soname 复制（跳过 libc/libm/libstdc++ 等基线库）
for _round in 1 2 3 4; do
    added=0
    while read -r p; do
        base=$(basename "$p")
        case "$base" in
            libc.so.6|libm.so.6|libgcc_s.so.1|libstdc++.so.6) continue;;
        esac
        [ -e "install/$base" ] && continue
        cp -L "$p" "install/$base"
        added=1
        need=$(glibc_need "$p")
        if glibc_too_new "$need"; then
            if [ "$fetched_baseline" = 1 ]; then
                echo "错误: $base 需要 glibc $need > 基线 $BASELINE_GLIBC，且基线容器可用（应优先取基线库）" >&2
                exit 1
            fi
            echo "警告: $base 需要 glibc $need > 基线 $BASELINE_GLIBC（目标发行版可能不兼容）" >&2
        fi
    done < <({ ldd install/llc 2>/dev/null; ldd install/libLLVM.so.21.1 2>/dev/null; ldd install/*.so.* 2>/dev/null; } \
        | awk '{print $3}' | grep -E '^/' | sort -u)
    [ "$added" = 0 ] && break
done

echo "== 5/5 打包 =="
tar czf "$tar_name" install/
echo "产物：$tar_name"

# 构建指纹资产（#177 后续）：同版本 respin 也能被 ayanami update --check 检测
build_id=$(./install/ayanami --version | sed -n 's/.*(build \(.*\))/\1/p')
printf '%s\n' "$build_id" > "ayanami-${ver}-build.txt"
echo "产物：ayanami-${ver}-build.txt（build $build_id）"

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
echo
echo "上传/更新 Release（含构建指纹，respin 用 --clobber 覆盖）："
echo "  gh release create v${ver} ${tar_name} ayanami-${ver}-build.txt [${vsix_name}] --title v${ver} --notes-file <notes>"
echo "  gh release upload v${ver} ${tar_name} ayanami-${ver}-build.txt [${vsix_name}] --clobber"
echo "  gh release edit v${ver} --notes-file <notes>"
