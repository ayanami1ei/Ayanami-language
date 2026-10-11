#!/usr/bin/env bash
# 本机 stage-0 沙盒（selfhost 分支专属）：把「安装版」或「本仓 release」编译器
# 布置成可用布局，供 scripts/selfhost.sh 的 golden 全套使用。
#
# 背景：安装版 runtime.c 是转发文件（`#include "../std/runtime.c"`），
# 若 ~/ayanami/std/runtime.c 缺失则 gcc 链接失败；本沙盒用仓库内 src/runtime.c
# （内含 `#include "../std/runtime.c"`，相对 selfhost/.stage0/ 解析到仓库 std/）。
#
#   ./scripts/selfhost-stage0.sh init [--from PATH]   # 建立 selfhost/.stage0/bin 布局
#   ./scripts/selfhost-stage0.sh show                 # 打印当前 stage-0 与 runtime 解析
#   ./scripts/selfhost-stage0.sh clean                # 删除沙盒
#
# 默认来源顺序：$SELFHOST_STAGE0 > 本仓 target/release/ayanami > ~/ayanami/ayanami
set -euo pipefail
cd "$(dirname "$0")/.."
ROOT=$(pwd)
SANDBOX="$ROOT/selfhost/.stage0"
BIN_DIR="$SANDBOX/bin"

usage() { sed -n '2,15p' "$0"; exit 1; }

resolve_source() {
    local from="${1:-}"
    if [[ -n "$from" ]]; then echo "$from"; return; fi
    if [[ -n "${SELFHOST_STAGE0:-}" && -x "$SELFHOST_STAGE0" ]]; then echo "$SELFHOST_STAGE0"; return; fi
    if [[ -x "$ROOT/target/release/ayanami" ]]; then echo "$ROOT/target/release/ayanami"; return; fi
    if [[ -x "$HOME/ayanami/ayanami" ]]; then echo "$HOME/ayanami/ayanami"; return; fi
    echo "selfhost-stage0: 找不到可用 stage-0（用 --from PATH 指定）" >&2
    exit 3
}

cmd_init() {
    local from=""
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --from) from="${2:-}"; shift 2 ;;
            *) usage ;;
        esac
    done
    local src; src=$(resolve_source "$from")
    src=$(readlink -f "$src")
    local srcdir; srcdir=$(dirname "$src")
    echo "== stage-0 来源：$src"
    mkdir -p "$BIN_DIR"
    ln -sfn "$src" "$BIN_DIR/ayanami"
    # 同目录依赖（llc / LLVM 动态库 / std 预编译包）：能链就链
    for dep in llc llvm-libs libLLVM.so.21.1 libedit.so.2 std; do
        if [[ -e "$srcdir/$dep" ]]; then ln -sfn "$srcdir/$dep" "$BIN_DIR/$dep"; fi
    done
    # runtime：生成指向仓库 std/runtime.c 的转发文件（安装版 gcc 只看该文件的相对 include）
    cat > "$SANDBOX/runtime.c" <<EOF
// 由 scripts/selfhost-stage0.sh 生成：转发到仓库 std/runtime.c
#include "$ROOT/std/runtime.c"
EOF
    ln -sfn "$ROOT/std" "$SANDBOX/std"
    echo "== 沙盒：$SANDBOX（bin/ayanami -> $src）"
    cmd_show
}

cmd_show() {
    echo "== 沙盒内容 =="
    ls -la "$BIN_DIR" 2>/dev/null || true
    [[ -f "$SANDBOX/runtime.c" ]] && head -2 "$SANDBOX/runtime.c" | tail -1
    [[ -L "$SANDBOX/std" ]] && echo "std/ -> $(readlink "$SANDBOX/std")"
    echo "== golden 前置检查 =="
    local ok=1
    for f in "$SANDBOX/runtime.c" "$ROOT/std/runtime.c" "$BIN_DIR/ayanami"; do
        if [[ -e "$f" ]]; then echo "  OK   $f"; else echo "  MISS $f"; ok=0; fi
    done
    [[ $ok -eq 1 ]] || exit 4
}

cmd_clean() { rm -rf "$SANDBOX"; echo "已删除 $SANDBOX"; }

case "${1:-}" in
    init) shift; cmd_init "$@" ;;
    show) cmd_show ;;
    clean) cmd_clean ;;
    *) usage ;;
esac
