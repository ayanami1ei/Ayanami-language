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
#   ./scripts/selfhost.sh e2e             # 端到端：自举管线编译 tests/e2e/*.aya 并与 stage-0 对比
#   ./scripts/selfhost.sh cli             # 用 stage-0 构建自举 CLI（selfhost/build/ayanami）
#   ./scripts/selfhost.sh diff [GLOB]    # stage-0/stage-1 输出与退出码差分
set -euo pipefail
cd "$(dirname "$0")/.."
ROOT=$(pwd)
# golden 中间产物目录（build/ 已是 gitignore 的构建目录；可用 AYANAMI_SELFHOST_WORK 覆盖）
WORK="${AYANAMI_SELFHOST_WORK:-$ROOT/build}"
mkdir -p "$WORK"
export AYANAMI_SELFHOST_WORK="$WORK"
# stage-0 链接用的 runtime：仓库内 src/runtime.c（转发 std/runtime.c），
# 避免安装版 runtime.c 的 `../std/runtime.c` 相对 include 在 ~/ayanami 下失效
RUNTIME_C="$ROOT/src/runtime.c"

# 资源隔离：单次调用限时 60s、虚拟内存 4GB（防止死循环/内存暴涨拖垮系统）。
# 说明：自举发射器暂不发射堆释放（无 __drop_String），内存随总分配量增长；
# stage-2 自编译实测峰值约 2.2GB（2026-10-24），故上限取 4GB。
run_limited() {
    ( ulimit -v 4194304 2>/dev/null; timeout 60 "$@" )
}

# 本机 stage-0 沙盒：`./scripts/selfhost-stage0.sh init`（安装版布局转发到仓库 src/runtime.c）
LOCAL_STAGE0="$ROOT/selfhost/.stage0/bin/ayanami"

stage0() {
    if [[ -n "${SELFHOST_STAGE0:-}" && -x "$SELFHOST_STAGE0" ]]; then
        echo "$SELFHOST_STAGE0"; return
    fi
    # 本仓 release 构建（开发态）：driver 自行定位仓库 src/runtime.c
    if [[ -x "$ROOT/target/release/ayanami" ]]; then
        echo "$ROOT/target/release/ayanami"; return
    fi
    # 工作区本地 stage-0 沙盒（.stage0/bin/{ayanami,llc,std}）
    if [[ -e "$LOCAL_STAGE0" ]]; then
        echo "$LOCAL_STAGE0"; return
    fi
    # 安装版：runtime.c 的 `../std/runtime.c` 相对 include 需靠 AYANAMI_RUNTIME 指回仓库
    if [[ -x "$HOME/ayanami/ayanami" ]]; then
        echo "$HOME/ayanami/ayanami"; return
    fi
    echo "selfhost: stage-0 不可用（先 ./scripts/selfhost.sh sync 或 SELFHOST_STAGE0=...）" >&2
    exit 3
}

# 统一入口：解析 stage-0，并保证 runtime 与 bundled llc 可用
# 前端命令（dump/check）用的 stage-0：不导出 AYANAMI_RUNTIME（否则 dump 会尝试重编 std lcl）
stage0_fe() {
    local bin; bin=$(stage0)
    echo "$bin"
}

stage0_ready() {
    local bin; bin=$(stage0)
    [[ -f "$RUNTIME_C" ]] && export AYANAMI_RUNTIME="$RUNTIME_C"
    if [[ "$(dirname "$bin")" == "$ROOT/selfhost/.stage0/bin" ]]; then
        export PATH="$(dirname "$bin"):$PATH"
    fi
    echo "$bin"
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
    local bin; bin=$(stage0_ready)
    local fail=0 total=0
    shopt -s nullglob
    for f in "$ROOT"/selfhost/tests/*_test.aya; do
        total=$((total + 1))
        if run_limited "$bin" run --release "$f" >$WORK/selfhost_test.out 2>&1; then
            echo "PASS $(basename "$f")"
        else
            echo "FAIL $(basename "$f")"; tail -5 $WORK/selfhost_test.out || true; fail=$((fail + 1))
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
    local s0; s0=$(stage0_ready)
    local rust_dump="$ROOT/tools/lexdump/target/release/lexdump"
    if [[ ! -x "$rust_dump" ]]; then
        echo "== 构建 lexdump =="
        cargo build --release --manifest-path "$ROOT/tools/lexdump/Cargo.toml" >/dev/null 2>&1 || { echo "lexdump 构建失败"; exit 5; }
    fi
    echo "== 构建 stage-0 lexer_dump（用 run 触发链接）=="
    run_limited "$s0" run --release "$ROOT/selfhost/tests/lexer_dump.aya" >/dev/null 2>&1 || true
    [[ -x "$WORK/lexer_dump" ]] || { echo "lexer_dump 构建失败"; exit 6; }
    local fail=0 total=0
    shopt -s nullglob
    for f in $pattern; do
        total=$((total + 1))
        run_limited "$rust_dump" "$f" > $WORK/tok_rust.txt 2>/dev/null || true
        run_limited env LEX_FILE="$f" "$WORK/lexer_dump" > $WORK/tok_aya.txt 2>/dev/null || true
        if /usr/bin/diff -q $WORK/tok_rust.txt $WORK/tok_aya.txt >/dev/null; then
            echo "TOKENS OK $(basename "$f")"
        else
            echo "TOKENS DIFF $(basename "$f")"
            /usr/bin/diff $WORK/tok_rust.txt $WORK/tok_aya.txt | head -6 || true
            fail=$((fail + 1))
        fi
    done
    shopt -u nullglob
    echo "tokens: $((total - fail))/$total 一致"
    [[ $fail -eq 0 ]]
}

cmd_ast() {
    local pattern="${1:-selfhost/tests/parse/*.aya}"
    local s0; s0=$(stage0_ready)
    local s0f; s0f=$(stage0_fe)
    echo "== 构建 stage-0 ast_dump（用 run 触发链接）=="
    run_limited "$s0" run --release "$ROOT/selfhost/tests/ast_dump.aya" >/dev/null 2>&1 || true
    [[ -x "$WORK/ast_dump" ]] || { echo "ast_dump 构建失败"; exit 6; }
    local fail=0 total=0
    shopt -s nullglob
    for f in $pattern; do
        total=$((total + 1))
        run_limited "$s0f" dump ast "$f" > $WORK/ast_rust.txt 2>/dev/null || true
        run_limited env PARSE_FILE="$f" "$WORK/ast_dump" > $WORK/ast_aya.txt 2>/dev/null || true
        # Import 路径依赖解析环境（相对路径/lcl 临时目录），golden 时归一化为占位符
        sed -i 's|Import { path: .* }|Import { path: <p> }|' $WORK/ast_rust.txt $WORK/ast_aya.txt
        # ConstDecl/StaticDecl 的 rust Debug 含 Span/Symbol 内部信息，归一化为占位符
        sed -i -E 's|(ConstDecl \{ name: [A-Za-z0-9_]+).*|\1 }|; s|(StaticDecl \{ name: [A-Za-z0-9_]+).*|\1 }|' $WORK/ast_rust.txt $WORK/ast_aya.txt
        # Lambda 的 rust Debug 含 Span 内部信息，归一化
        sed -i -E 's|(Lambda\().*|\1...)|' $WORK/ast_rust.txt $WORK/ast_aya.txt
        if /usr/bin/diff -q $WORK/ast_rust.txt $WORK/ast_aya.txt >/dev/null; then
            echo "AST OK $(basename "$f")"
        else
            echo "AST DIFF $(basename "$f")"
            /usr/bin/diff $WORK/ast_rust.txt $WORK/ast_aya.txt | head -8 || true
            fail=$((fail + 1))
        fi
    done
    shopt -u nullglob
    echo "ast: $((total - fail))/$total 一致"
    [[ $fail -eq 0 ]]
}

cmd_errs() {
    local s0; s0=$(stage0_ready)
    local s0f; s0f=$(stage0_fe)
    echo "== 构建 stage-0 ast_dump =="
    run_limited "$s0" run --release "$ROOT/selfhost/tests/ast_dump.aya" >/dev/null 2>&1 || true
    [[ -x "$WORK/ast_dump" ]] || { echo "ast_dump 构建失败"; exit 6; }
    local fail=0 total=0
    shopt -s nullglob
    for f in "$ROOT"/selfhost/tests/parse_bad/*.aya; do
        total=$((total + 1))
        rp=$(run_limited "$s0f" check --release "$f" 2>&1 | grep -oE ':[0-9]+:[0-9]+' | head -1 || true)
        out=$(run_limited env PARSE_FILE="$f" "$WORK/ast_dump" 2>&1 || true)
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
    local s0; s0=$(stage0_ready)
    local s0f; s0f=$(stage0_fe)
    echo "== 构建 stage-0 check_dump =="
    run_limited "$s0" run --release "$ROOT/selfhost/tests/check_dump.aya" >/dev/null 2>&1 || true
    [[ -x "$WORK/check_dump" ]] || { echo "check_dump 构建失败"; exit 6; }
    local fail=0 total=0
    shopt -s nullglob
    for f in "$ROOT"/selfhost/tests/typecheck_bad/*.aya "$ROOT"/selfhost/tests/typecheck_ok/*.aya; do
        total=$((total + 1))
        rust_sig=$(run_limited "$s0f" check --release "$f" 2>&1 | sig || true)
        ours_sig=$(run_limited env CHECK_FILE="$f" "$WORK/check_dump" 2>&1 | sig || true)
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

cmd_e2e() {
    local s0; s0=$(stage0_ready)
    echo "== 构建 stage-0 e2e_run =="
    run_limited "$s0" run --release "$ROOT/selfhost/tests/e2e_run.aya" >/dev/null 2>&1 || true
    [[ -x "$WORK/e2e_run" ]] || { echo "e2e_run 构建失败"; exit 6; }
    local fail=0 total=0
    shopt -s nullglob
    for f in "$ROOT"/selfhost/tests/e2e/*.aya; do
        total=$((total + 1))
        set +e
        s0_out=$(run_limited "$s0" run --release "$f" 2>/dev/null); s0_code=$?
        sh_out=$(run_limited env E2E_FILE="$f" "$WORK/e2e_run" 2>/dev/null); sh_code=$?
        set -e
        if [[ "$s0_out" == "$sh_out" && "$s0_code" == "$sh_code" ]]; then
            echo "E2E OK $(basename "$f") (exit=$sh_code out=${sh_out:-<empty>})"
        else
            echo "E2E DIFF $(basename "$f") stage0=$s0_code/${s0_out:-<empty>} ours=$sh_code/${sh_out:-<empty>}"
            fail=$((fail + 1))
        fi
    done
    shopt -u nullglob
    echo "e2e: $((total - fail))/$total 一致"
    [[ $fail -eq 0 ]]
}

cmd_diff() {
    local pattern="${1:-example/*.aya}"
    if [[ ! -x "$ROOT/selfhost/build/ayanami" ]]; then
        echo "selfhost: stage-1 未构建（先 ./scripts/selfhost.sh cli）"
        exit 4
    fi
    local s0 s1 fail=0 total=0
    s0=$(stage0_ready); s1="$ROOT/selfhost/build/ayanami"
    # stage-1（自举 CLI）需要 std 包做导入解析；stage-0 自带 target/release/std
    export AYANAMI_STD="${AYANAMI_STD:-$ROOT/target/release/std}"
    export AYANAMI_SELFHOST_WORK="${AYANAMI_SELFHOST_WORK:-$WORK}"
    shopt -s nullglob
    for f in $pattern; do
        total=$((total + 1))
        "$s0" run "$f" >$WORK/diff.s0 2>&1 || c0=$?
        c0=${c0:-0}
        "$s1" run "$f" >$WORK/diff.s1 2>&1 || c1=$?
        c1=${c1:-0}
        # 两边都会打印自身进度行，比较时剔除（只留程序输出）；
        # stage-0 在子进程非 0 退出时还会打印 "error: process didn't exit successfully"，一并剔除
        NOISE="^[[:space:]]*(Compiling|Checking|Finished|Running|Packaging)|^error: process didn"
        grep -v -E "$NOISE" $WORK/diff.s0 > $WORK/diff.s0.clean || true
        grep -v -E "$NOISE" $WORK/diff.s1 > $WORK/diff.s1.clean || true
        if [[ $c0 -ne $c1 ]] || ! /usr/bin/diff -q $WORK/diff.s0.clean $WORK/diff.s1.clean >/dev/null; then
            echo "DIFF $f (stage0=$c0 stage1=$c1)"
            /usr/bin/diff $WORK/diff.s0.clean $WORK/diff.s1.clean | head -6 || true
            fail=$((fail + 1))
        else
            echo "SAME $f (exit=$c0)"
        fi
        c0=""; c1=""
    done
    shopt -u nullglob
    echo "diff: $((total - fail))/$total 一致（stage-0 vs stage-1）"
    [[ $fail -eq 0 ]]
}

# 用 stage-0 构建自举 CLI 本体 → selfhost/build/ayanami
cmd_selfcompile() {
    # M7 固定点：stage-1 编译自身 -> stage-2；stage-2 再编译 -> stage-3；要求字节一致
    local s0; s0=$(stage0_ready)
    run_limited "$s0" run --release "$ROOT/selfhost/src/main.aya" >"$WORK/cli_build.log" 2>&1 || {
        echo "stage-1 构建失败："; tail -5 "$WORK/cli_build.log"; exit 5;
    }
    [[ -x "$WORK/main" ]] || { echo "stage-1 产物缺失"; exit 5; }
    local s1="$WORK/main"
    mkdir -p "$WORK/fp2" "$WORK/fp3"
    export AYANAMI_STD="${AYANAMI_STD:-$ROOT/target/release/std}"
    export AYANAMI_RUNTIME="${AYANAMI_RUNTIME:-$ROOT/src/runtime.c}"
    echo "== stage-1 编译自身 -> stage-2 =="
    AYANAMI_SELFHOST_WORK="$WORK/fp2" AYANAMI_OUT_DIR="$WORK/fp2" \
        run_limited "$s1" build "$ROOT/selfhost/src/main.aya" >"$WORK/fp2.log" 2>&1 || {
        echo "stage-2 构建失败："; tail -5 "$WORK/fp2.log"; exit 6;
    }
    [[ -x "$WORK/fp2/main" ]] || { echo "stage-2 产物缺失"; exit 6; }
    echo "== stage-2 编译自身 -> stage-3 =="
    AYANAMI_SELFHOST_WORK="$WORK/fp3" AYANAMI_OUT_DIR="$WORK/fp3" \
        run_limited "$WORK/fp2/main" build "$ROOT/selfhost/src/main.aya" >"$WORK/fp3.log" 2>&1 || {
        echo "stage-3 构建失败："; tail -5 "$WORK/fp3.log"; exit 7;
    }
    [[ -x "$WORK/fp3/main" ]] || { echo "stage-3 产物缺失"; exit 7; }
    local m2 m3
    m2=$(md5sum "$WORK/fp2/main" | cut -d' ' -f1)
    m3=$(md5sum "$WORK/fp3/main" | cut -d' ' -f1)
    if [[ "$m2" == "$m3" ]]; then
        echo "固定点达成：stage-2 == stage-3 (md5=$m2)"
        cp "$WORK/fp2/main" "$ROOT/selfhost/build/ayanami-stage2"
        return 0
    fi
    echo "固定点失败：stage-2=$m2 stage-3=$m3"
    return 1
}

cmd_cli() {
    local s0; s0=$(stage0_ready)
    run_limited "$s0" run --release "$ROOT/selfhost/src/main.aya" >"$WORK/cli_build.log" 2>&1 || {
        echo "stage-1 CLI 构建失败："; tail -5 "$WORK/cli_build.log"; exit 5;
    }
    [[ -x "$WORK/main" ]] || { echo "stage-1 CLI 产物缺失（$WORK/main）"; exit 5; }
    mkdir -p "$ROOT/selfhost/build"
    cp "$WORK/main" "$ROOT/selfhost/build/ayanami"
    echo "stage-1 CLI: $ROOT/selfhost/build/ayanami"
    "$ROOT/selfhost/build/ayanami" help || true
}

case "${1:-}" in
    sync) cmd_sync ;;
    stage0) stage0 ;;
    run) shift; bin=$(stage0_ready); exec "$bin" run "$@" ;;
    test) cmd_test ;;
    kb) shift; python3 "$ROOT/tools/kb/kb.py" search "$@" ;;
    tokens) shift; cmd_tokens "$@" ;;
    ast) shift; cmd_ast "$@" ;;
    errs) shift; cmd_errs "$@" ;;
    check) shift; cmd_check "$@" ;;
    e2e) shift; cmd_e2e "$@" ;;
    cli) cmd_cli ;;
    selfcompile) cmd_selfcompile ;;
    diff) shift; cmd_diff "$@" ;;
    *) sed -n '2,16p' "$0"; exit 1 ;;
esac
