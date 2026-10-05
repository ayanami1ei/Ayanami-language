#!/usr/bin/env bash
# Ayanami 自举工程辅助脚本（selfhost 分支专属）
#
#   ./scripts/selfhost.sh sync           # 合并 origin/rust + 更新 book 子仓 + 重建知识库
#   ./scripts/selfhost.sh stage0         # 确保 stage-0（本仓 release 或安装版）可用
#   ./scripts/selfhost.sh run FILE.aya   # 用 stage-0 运行单个文件
#   ./scripts/selfhost.sh test           # 运行 selfhost/tests/*_test.aya（退出码 0 = 通过）
#   ./scripts/selfhost.sh kb "QUERY"     # 知识库检索（供本地模型提示词使用）
#   ./scripts/selfhost.sh diff [GLOB]    # stage-0/stage-1 差分（stage-1 存在后生效）
set -euo pipefail
cd "$(dirname "$0")/.."
ROOT=$(pwd)

stage0() {
    if [[ -n "${SELFHOST_STAGE0:-}" && -x "$SELFHOST_STAGE0" ]]; then
        echo "$SELFHOST_STAGE0"; return
    fi
    if [[ -x "$ROOT/target/release/ayanami" ]]; then
        echo "$ROOT/target/release/ayanami"; return
    fi
    if [[ -x "$HOME/ayanami/ayanami" ]]; then
        echo "$HOME/ayanami/ayanami"; return
    fi
    echo "selfhost: stage-0 不可用（先 ./scripts/selfhost.sh sync 或 SELFHOST_STAGE0=...）" >&2
    exit 3
}

cmd_sync() {
    local before after
    before=$(git rev-parse HEAD)
    echo "== fetch origin =="
    git fetch origin
    echo "== merge origin/rust -> selfhost =="
    if git merge --ff-only origin/rust >/dev/null 2>&1; then
        echo "ff-only ok"
    else
        git merge --no-edit origin/rust
    fi
    after=$(git rev-parse HEAD)
    echo "== submodules（book/asuka 跟随远端）=="
    git submodule update --init --remote book std
    echo "== stage-0：rust 有更新则重建 release =="
    if [[ "$before" != "$after" || ! -x "$ROOT/target/release/ayanami" ]]; then
        cargo build --release
    fi
    # 注：新驱动在开发态直接找仓库 src/runtime.c（其内部 include std/runtime.c），不要再拷贝到 target/
    echo "== stage-0 std 预编译（std 子仓 build.sh -> target/{release,debug}/std）=="
    if [[ "$before" != "$after" || ! -f "$ROOT/target/release/std/string.lcl" ]]; then
        ( cd std && AYANAMI_BIN="$ROOT/target/release/ayanami" ./scripts/build.sh --install "$ROOT/target/release/std" )
        mkdir -p "$ROOT/target/debug/std"
        cp "$ROOT/target/release/std/"*.lcl "$ROOT/target/debug/std/"
    fi
    echo "== 重建知识库 =="
    python3 tools/kb/kb.py build
    echo "== sync 完成 =="
}

cmd_test() {
    local bin; bin=$(stage0)
    local fail=0 total=0
    shopt -s nullglob
    for f in "$ROOT"/selfhost/tests/*_test.aya; do
        total=$((total + 1))
        if "$bin" run --release "$f" >/tmp/opencode/selfhost_test.out 2>&1; then
            echo "PASS $(basename "$f")"
        else
            echo "FAIL $(basename "$f")"; tail -5 /tmp/opencode/selfhost_test.out || true; fail=$((fail + 1))
        fi
    done
    shopt -u nullglob
    if [[ $total -eq 0 ]]; then
        echo "selfhost: 暂无测试（selfhost/tests/*_test.aya）"
    fi
    [[ $fail -eq 0 ]]
}

cmd_diff() {
    local pattern="${1:-example/*.aya}"
    if [[ ! -x "$ROOT/selfhost/build/ayanami" ]]; then
        echo "selfhost: stage-1 未构建（selfhost/build/ayanami），M5 后端到端差分才可用"
        exit 4
    fi
    local s0 s1 fail=0
    s0=$(stage0); s1="$ROOT/selfhost/build/ayanami"
    for f in $pattern; do
        "$s0" run "$f" >/tmp/opencode/diff.s0 2>&1 && c0=$? || c0=$?
        "$s1" run "$f" >/tmp/opencode/diff.s1 2>&1 && c1=$? || c1=$?
        if [[ $c0 -ne $c1 ]] || ! diff -q <(grep -v '^\s*\(Compiling\|Finished\|Running\)' /tmp/opencode/diff.s0) \
                                     <(grep -v '^\s*\(Compiling\|Finished\|Running\)' /tmp/opencode/diff.s1) >/dev/null; then
            echo "DIFF $f (stage0=$c0 stage1=$c1)"; fail=$((fail + 1))
        fi
    done
    [[ $fail -eq 0 ]] && echo "diff: 一致" || echo "diff: $fail 处不一致"
    [[ $fail -eq 0 ]]
}

case "${1:-}" in
    sync) cmd_sync ;;
    stage0) stage0 ;;
    run) shift; bin=$(stage0); exec "$bin" run "$@" ;;
    test) cmd_test ;;
    kb) shift; python3 "$ROOT/tools/kb/kb.py" search "$@" ;;
    diff) shift; cmd_diff "$@" ;;
    *) sed -n '2,12p' "$0"; exit 1 ;;
esac
