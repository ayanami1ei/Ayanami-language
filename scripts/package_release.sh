#!/usr/bin/env bash
# package_release.sh — 构建 / 发布 / 验证（子命令）
#
# 用法：
#   ./scripts/package_release.sh [package] [--no-vsix]   # 构建 tar/vsix/build.txt（默认，等价旧用法）
#   ./scripts/package_release.sh upload                  # 上传资产到 GitHub Release + 更新 notes 构建号
#   ./scripts/package_release.sh verify                  # 干净 Ubuntu 24.04 容器跑 runtime_safety 全量
#   ./scripts/package_release.sh machines [--from <tar>] # 本机 + 远端（默认联网；--from 走本地 tar）
#   ./scripts/package_release.sh all [--no-vsix]         # package → upload → verify
#
# 环境变量：
#   AYANAMI_BASELINE_IMAGE  基线容器镜像（默认 ubuntu:24.04，回退 docker.m.daocloud.io）
#   AYANAMI_REMOTE          远端 SSH 主机（默认 ayanami-lite；置空跳过远端）
#   AYANAMI_LOCAL_BIN       本机安装的编译器（默认 ~/ayanami/ayanami）
#   GITHUB_TOKEN            上传用（缺省尝试 gh auth token）
#
# 产物：
#   ayanami-<version>-linux-<arch>.tar.gz   内含 install/（编译器 + bundled LLVM + runtime + std）
#   ayanami-<version>.vsix                  VSCode 插件（找到 vsce 时构建）
#   ayanami-<version>-build.txt             构建指纹（同版本 respin 的 update --check 依据）
set -euo pipefail
cd "$(dirname "$0")/.."

ver=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
arch=$(uname -m)
tar_name="ayanami-${ver}-linux-${arch}.tar.gz"
vsix_name="ayanami-${ver}.vsix"
build_txt="ayanami-${ver}-build.txt"
tag="v${ver}"
BASELINE_GLIBC="2.39"

glibc_need() { objdump -T "$1" 2>/dev/null | grep -o 'GLIBC_[0-9]\+\.[0-9]\+' | sort -Vu | tail -1 | sed 's/GLIBC_//'; }
glibc_too_new() { [ -n "$1" ] && [ "$(printf '%s\n%s\n' "$BASELINE_GLIBC" "$1" | sort -V | tail -1)" != "$BASELINE_GLIBC" ]; }

# 基线镜像：优先 AYANAMI_BASELINE_IMAGE，其次 daocloud 镜像源；不可用时返回非零
pick_image() {
    for img in "${AYANAMI_BASELINE_IMAGE:-ubuntu:24.04}" "docker.m.daocloud.io/library/ubuntu:24.04"; do
        if docker image inspect "$img" >/dev/null 2>&1 || docker pull -q "$img" >/dev/null 2>&1; then
            echo "$img"
            return 0
        fi
    done
    return 1
}

# ----------------------------------------------------------------
#  package：构建产物（原行为）
# ----------------------------------------------------------------
cmd_package() {
    local build_vsix=1
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
    # 1) 基线库（libedit/libncurses 等）优先从 ubuntu:24.04 容器取
    fetched_baseline=0
    if command -v docker >/dev/null 2>&1; then
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
    printf '%s\n' "$build_id" > "$build_txt"
    echo "产物：$build_txt（build $build_id）"

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

    echo "打包完成（下一步：./scripts/package_release.sh upload）"
}

# ----------------------------------------------------------------
#  upload：上传资产到 GitHub Release + 更新 notes 构建号
# ----------------------------------------------------------------
cmd_upload() {
    [ -f "$tar_name" ] || { echo "缺少 $tar_name，请先运行 ./scripts/package_release.sh package" >&2; exit 1; }
    command -v gh >/dev/null 2>&1 || { echo "需要 gh CLI（GitHub CLI）" >&2; exit 1; }
    export GITHUB_TOKEN="${GITHUB_TOKEN:-$(gh auth token 2>/dev/null || true)}"
    if ! gh release view "$tag" >/dev/null 2>&1; then
        echo "Release $tag 不存在；先创建：gh release create $tag $tar_name $build_txt [--title ...]" >&2
        exit 1
    fi

    assets=("$tar_name" "$build_txt")
    [ -f "$vsix_name" ] && assets+=("$vsix_name")
    echo "== 上传资产到 $tag（--clobber 覆盖 respin）=="
    gh release upload "$tag" "${assets[@]}" --clobber

    # notes：把“构建自 `rust` 分支 `hash`”更新为当前 build id（其余内容保留）
    if [ -f "$build_txt" ]; then
        local bid tmp
        bid=$(sed 's/-dirty$//' "$build_txt" | head -1)
        tmp=$(mktemp)
        gh release view "$tag" --json body --jq .body > "$tmp"
        python3 - "$tmp" "$bid" <<'PY'
import re, sys
path, bid = sys.argv[1], sys.argv[2]
s = open(path, encoding='utf-8').read()
s2 = re.sub(r"(构建自 `rust` 分支 `)[0-9a-f]{6,40}", r"\g<1>" + bid, s, count=1)
if s2 == s:
    s2 = s.rstrip() + f"\n\n> 本资产构建自 `rust` 分支 `{bid}`（respin）。\n"
open(path, 'w', encoding='utf-8').write(s2)
PY
        gh release edit "$tag" --notes-file "$tmp"
        rm -f "$tmp"
    fi
    echo "上传完成；远端 build.txt: $(gh release download "$tag" --pattern "$build_txt" --output - 2>/dev/null | head -1)"
}

# ----------------------------------------------------------------
#  verify：干净 Ubuntu 24.04 容器端到端验证（runtime_safety 全量）
# ----------------------------------------------------------------
cmd_verify() {
    [ -f "$tar_name" ] || { echo "缺少 $tar_name，请先运行 ./scripts/package_release.sh package" >&2; exit 1; }
    command -v docker >/dev/null 2>&1 || { echo "需要 docker" >&2; exit 1; }
    local img
    img=$(pick_image) || { echo "无法获取基线镜像（docker pull 失败）" >&2; exit 1; }

    local tmp
    tmp=$(mktemp -d "${TMPDIR:-/tmp}/ayanami-verify-XXXXXX")
    cp "$tar_name" "$tmp/"
    cp tests/runtime_safety/*.aya "$tmp/" 2>/dev/null || true
    # manifest -> checks.txt（file|exit|pattern）
    python3 - "$tmp/checks.txt" <<'PY'
import sys
out = open(sys.argv[1], 'w', encoding='utf-8')
for line in open('tests/runtime_safety/manifest.txt', encoding='utf-8'):
    line = line.strip()
    if not line or line.startswith('#'):
        continue
    parts = line.split(None, 2)
    f = parts[0]
    code = parts[1] if len(parts) > 1 else '0'
    pat = parts[2] if len(parts) > 2 else ''
    out.write(f"{f}|{code}|{pat}\n")
out.close()
PY
    local n
    n=$(wc -l < "$tmp/checks.txt")
    echo "== 干净容器验证（$img，$n 个用例）=="
    local rc=0
    docker run --rm -v "$tmp:/t" "$img" bash -c '
set -u
apt-get -o Acquire::http::Dl-Limit=10240 -o Acquire::https::Dl-Limit=10240 update -qq >/dev/null 2>&1 || true
apt-get -o Acquire::http::Dl-Limit=10240 -o Acquire::https::Dl-Limit=10240 install -y -qq gcc >/dev/null 2>&1 || { echo "容器内 gcc 安装失败（网络？）"; exit 3; }
cd /t && mkdir -p x && tar xzf ayanami-*.tar.gz -C x
D=$(ls -d x/*/ | head -1)
"$D/ayanami" --version
pass=0; fail=0
while IFS="|" read -r f code pat; do
  [ -z "$f" ] && continue
  out=$(timeout 120 "$D/ayanami" run "$f" 2>&1); got=$?
  ok=1
  [ "$got" != "$code" ] && ok=0
  if [ -n "$pat" ]; then printf "%s" "$out" | grep -qF -- "$pat" || ok=0; fi
  if [ "$ok" = 1 ]; then pass=$((pass+1)); else fail=$((fail+1)); echo "FAIL $f: exit=$got want=$code pat=$pat"; fi
done < checks.txt
echo "容器验证: pass=$pass fail=$fail"
[ "$fail" = 0 ]
' || rc=$?
    # 清理（容器内以 root 写入，best-effort）
    docker run --rm -v "$tmp:/t" "$img" bash -c 'rm -rf /t/*' >/dev/null 2>&1 || true
    rm -rf "$tmp" 2>/dev/null || true
    if [ "$rc" != 0 ]; then
        echo "容器验证失败（rc=$rc）" >&2
        exit 1
    fi
    echo "容器验证通过"
}

# ----------------------------------------------------------------
#  machines：本机 + 远端更新（默认联网；--from 走本地 tar）
# ----------------------------------------------------------------
cmd_machines() {
    local from_tar=""
    if [ "${1:-}" = "--from" ]; then
        from_tar="${2:?用法: machines [--from <tar>]}"
    fi

    local local_bin="${AYANAMI_LOCAL_BIN:-$HOME/ayanami/ayanami}"
    if [ -x "$local_bin" ]; then
        echo "== 本机更新（$local_bin）=="
        if [ -n "$from_tar" ]; then
            "$local_bin" update --from "$from_tar" | tail -1
        else
            "$local_bin" update | tail -1
        fi
        "$local_bin" --version
        "$local_bin" update --check 2>&1 | tail -1 || true
    else
        echo "警告：未找到本机编译器 $local_bin（AYANAMI_LOCAL_BIN 可覆盖）" >&2
    fi

    local remote="${AYANAMI_REMOTE:-ayanami-lite}"
    if [ -n "$remote" ]; then
        echo "== 远端更新（$remote）=="
        if [ -n "$from_tar" ]; then
            if scp -q -l 80000 "$from_tar" "$remote:/tmp/"; then
                ssh -o ConnectTimeout=10 "$remote" "~/ayanami/ayanami update --from /tmp/$(basename "$from_tar")" | tail -1 \
                    || echo "警告：远端更新失败（离线？上线后重跑）" >&2
            else
                echo "警告：远端不可达（scp 失败），跳过" >&2
            fi
        else
            ssh -o ConnectTimeout=10 "$remote" '~/ayanami/ayanami update' | tail -1 \
                || echo "警告：远端更新失败（离线？上线后重跑 ./scripts/package_release.sh machines）" >&2
        fi
        ssh -o ConnectTimeout=10 "$remote" '~/ayanami/ayanami --version' 2>/dev/null || true
    fi
}

usage() {
    sed -n '2,17p' "$0" | sed 's/^# \{0,1\}//'
}

# ----------------------------------------------------------------
#  dispatch
# ----------------------------------------------------------------
case "${1:-package}" in
    package)   shift || true; cmd_package "${1:-}";;
    upload)    shift || true; cmd_upload;;
    verify)    shift || true; cmd_verify;;
    machines)  shift || true; cmd_machines "${1:-}";;
    all)       shift || true; cmd_package "${1:-}"; cmd_upload; cmd_verify;;
    help|-h|--help) usage;;
    *)         cmd_package "$@";;   # 兼容旧用法（如 --no-vsix）
esac
