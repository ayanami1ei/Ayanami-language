#!/usr/bin/env bash
# Ayanami 自举工程辅助脚本（selfhost 分支专属）
#
#   ./scripts/selfhost.sh sync           # 合并 origin/rust + 更新 book 子仓 + 重建知识库
#   ./scripts/selfhost.sh stage0         # 确保 stage-0（本仓 release 或安装版）可用
#   ./scripts/selfhost.sh run FILE.aya   # 用 stage-0 运行单个文件
#   ./scripts/selfhost.sh test           # 运行 selfhost/tests/*_test.aya（退出码 0 = 通过）
#   ./scripts/selfhost.sh kb "QUERY"     # 知识库检索（供本地模型提示词使用）
#   ./scripts/selfhost.sh tokens [GLOB]   # 词法 golden：与 rust lexdump 对比 token 流
#   ./scripts/selfhost.sh ast [GLOB]      # 语法 golden：与 rust `dump ast` 对比 AST 树
#   ./scripts/selfhost.sh errs            # 错误定位 golden：与 rust check 的首个错误 line:col 对比
#   ./scripts/selfhost.sh check           # 类型检查 golden：与 rust check 的诊断（消息+位置）对比
#   ./scripts/selfhost.sh diff [GLOB]    # stage-0/stage-1 差分（stage-1 存在后生效）
set -euo pipefail
cd "$(dirname "$0")/.."
ROOT=$(pwd)

# 资源隔离：单次调用限时 60s、虚拟内存 2GB（防止死循环/内存暴涨拖垮系统）
run_limited() {
    ( ulimit -v 2097152 2>/dev/null; timeout 60 "$@" )
}

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
        if run_limited "$bin" run --release "$f" >/tmp/opencode/selfhost_test.out 2>&1; then
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

cmd_tokens() {
    local pattern="${1:-example/*.aya}"
    local s0; s0=$(stage0)
    local rust_dump="$ROOT/tools/lexdump/target/release/lexdump"
    if [[ ! -x "$rust_dump" ]]; then
        echo "== 构建 lexdump =="
        cargo build --release --manifest-path "$ROOT/tools/lexdump/Cargo.toml" >/dev/null 2>&1 || { echo "lexdump 构建失败"; exit 5; }
    fi
    echo "== 构建 stage-0 lexer_dump（用 run 触发链接）=="
    run_limited "$s0" run --release "$ROOT/selfhost/tests/lexer_dump.aya" >/dev/null 2>&1 || true
    [[ -x "$ROOT/build/lexer_dump" ]] || { echo "lexer_dump 构建失败"; exit 6; }
    local fail=0 total=0
    shopt -s nullglob
    for f in $pattern; do
        total=$((total + 1))
        run_limited "$rust_dump" "$f" > /tmp/opencode/tok_rust.txt 2>/dev/null || true
        run_limited env LEX_FILE="$f" "$ROOT/build/lexer_dump" > /tmp/opencode/tok_aya.txt 2>/dev/null || true
        if diff -q /tmp/opencode/tok_rust.txt /tmp/opencode/tok_aya.txt >/dev/null; then
            echo "TOKENS OK $(basename "$f")"
        else
            echo "TOKENS DIFF $(basename "$f")"
            diff /tmp/opencode/tok_rust.txt /tmp/opencode/tok_aya.txt | head -6 || true
            fail=$((fail + 1))
        fi
    done
    shopt -u nullglob
    echo "tokens: $((total - fail))/$total 一致"
    [[ $fail -eq 0 ]]
}

cmd_ast() {
    local pattern="${1:-selfhost/tests/parse/*.aya}"
    local s0; s0=$(stage0)
    echo "== 构建 stage-0 ast_dump（用 run 触发链接）=="
    run_limited "$s0" run --release "$ROOT/selfhost/tests/ast_dump.aya" >/dev/null 2>&1 || true
    [[ -x "$ROOT/build/ast_dump" ]] || { echo "ast_dump 构建失败"; exit 6; }
    local fail=0 total=0
    shopt -s nullglob
    for f in $pattern; do
        total=$((total + 1))
        run_limited "$s0" dump ast "$f" > /tmp/opencode/ast_rust.txt 2>/dev/null || true
        run_limited env PARSE_FILE="$f" "$ROOT/build/ast_dump" > /tmp/opencode/ast_aya.txt 2>/dev/null || true
        # Import 路径依赖解析环境（相对路径/lcl 临时目录），golden 时归一化为占位符
        sed -i 's|Import { path: .* }|Import { path: <p> }|' /tmp/opencode/ast_rust.txt /tmp/opencode/ast_aya.txt
        # ConstDecl/StaticDecl 的 rust Debug 含 Span/Symbol 内部信息，归一化为占位符
        sed -i -E 's|(ConstDecl \{ name: [A-Za-z0-9_]+).*|\1 }|; s|(StaticDecl \{ name: [A-Za-z0-9_]+).*|\1 }|' /tmp/opencode/ast_rust.txt /tmp/opencode/ast_aya.txt
        # Lambda 的 rust Debug 含 Span 内部信息，归一化
        sed -i -E 's|(Lambda\().*|\1...)|' /tmp/opencode/ast_rust.txt /tmp/opencode/ast_aya.txt
        if diff -q /tmp/opencode/ast_rust.txt /tmp/opencode/ast_aya.txt >/dev/null; then
            echo "AST OK $(basename "$f")"
        else
            echo "AST DIFF $(basename "$f")"
            diff /tmp/opencode/ast_rust.txt /tmp/opencode/ast_aya.txt | head -8 || true
            fail=$((fail + 1))
        fi
    done
    shopt -u nullglob
    echo "ast: $((total - fail))/$total 一致"
    [[ $fail -eq 0 ]]
}

cmd_errs() {
    local s0; s0=$(stage0)
    echo "== 构建 stage-0 ast_dump =="
    run_limited "$s0" run --release "$ROOT/selfhost/tests/ast_dump.aya" >/dev/null 2>&1 || true
    [[ -x "$ROOT/build/ast_dump" ]] || { echo "ast_dump 构建失败"; exit 6; }
    local fail=0 total=0
    shopt -s nullglob
    for f in "$ROOT"/selfhost/tests/parse_bad/*.aya; do
        total=$((total + 1))
        rp=$(run_limited "$s0" check --release "$f" 2>&1 | grep -oE ':[0-9]+:[0-9]+' | head -1 || true)
        out=$(run_limited env PARSE_FILE="$f" "$ROOT/build/ast_dump" 2>&1 || true)
        ln=$(printf '%s\n' "$out" | sed -n '3p')
        co=$(printf '%s\n' "$out" | sed -n '4p')
        op=":$ln:$co"
        if [[ -z "$rp" ]]; then
            echo "ERR OK $(basename "$f") rust=<none>（EOF 错误无位置）"
        elif [[ "$rp" == "$op" ]]; then
            echo "ERR OK $(basename "$f") rust=$rp"
        else
            echo "ERR DIFF $(basename "$f") rust=${rp:-<none>} ours=$op"
            fail=$((fail + 1))
        fi
    done
    shopt -u nullglob
    echo "errs: $((total - fail))/$total 一致"
    [[ $fail -eq 0 ]]
}

# 诊断签名：error 行 + 随后的 --> 位置，合并为 "error: msg @ line:col"
sig() {
    awk '/^error: /{msg=$0} /^  --> /{n=split($0,a,":"); print msg" @"a[n-1]":"a[n]}'
}

cmd_check() {
    local s0; s0=$(stage0)
    echo "== 构建 stage-0 check_dump =="
    run_limited "$s0" run --release "$ROOT/selfhost/tests/check_dump.aya" >/dev/null 2>&1 || true
    [[ -x "$ROOT/build/check_dump" ]] || { echo "check_dump 构建失败"; exit 6; }
    local fail=0 total=0
    shopt -s nullglob
    for f in "$ROOT"/selfhost/tests/typecheck_bad/*.aya "$ROOT"/selfhost/tests/typecheck_ok/*.aya; do
        total=$((total + 1))
        rust_sig=$(run_limited "$s0" check --release "$f" 2>&1 | sig || true)
        ours_sig=$(run_limited env CHECK_FILE="$f" "$ROOT/build/check_dump" 2>&1 | sig || true)
        if [[ "$rust_sig" == "$ours_sig" ]]; then
            echo "CHECK OK $(basename "$f")"
        else
            echo "CHECK DIFF $(basename "$f")"
            printf '  rust: %s\n' "$rust_sig"
            printf '  ours: %s\n' "$ours_sig"
            fail=$((fail + 1))
        fi
    done
    shopt -u nullglob
    echo "check: $((total - fail))/$total 一致"
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
    tokens) shift; cmd_tokens "$@" ;;
    ast) shift; cmd_ast "$@" ;;
    errs) shift; cmd_errs "$@" ;;
    check) shift; cmd_check "$@" ;;
    diff) shift; cmd_diff "$@" ;;
    *) sed -n '2,12p' "$0"; exit 1 ;;
esac
