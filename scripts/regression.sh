#!/usr/bin/env bash
# regression.sh — 语言 feature 正/负回归
#
#   正例：example/*.aya，按 tests/positive_exit.txt 校验退出码（含 panic 101 等）
#   负例：tests/compile_fail/*.aya，按同名 .expected（每行一个子串）校验报错内容
#
# 用法：./scripts/regression.sh [--binary target/debug/ayanami]
set -uo pipefail
cd "$(dirname "$0")/.."

BIN="target/debug/ayanami"
if [ "${1:-}" = "--binary" ] && [ -n "${2:-}" ]; then BIN="$2"; fi
if [ ! -x "$BIN" ] || [ "$BIN" = "target/debug/ayanami" ]; then cargo build -q; fi

fail=0
pos=0
while read -r name code; do
    [ -z "${name:-}" ] && continue
    case "$name" in \#*) continue ;; esac
    pos=$((pos + 1))
    timeout 180 "$BIN" run "example/$name" >/dev/null 2>&1
    got=$?
    if [ "$got" != "$code" ]; then
        echo "FAIL example/$name: exit $got, want $code"
        fail=$((fail + 1))
    fi
done < tests/positive_exit.txt

neg=0
for f in tests/compile_fail/*.aya; do
    [ -e "$f" ] || continue
    neg=$((neg + 1))
    base="${f%.aya}"
    if [ ! -f "$base.expected" ]; then
        echo "FAIL $f: missing $base.expected"
        fail=$((fail + 1))
        continue
    fi
    out=$(timeout 60 "$BIN" check "$f" 2>&1)
    # .expected 每行是一个候选子串；命中任意一个即通过
    # （借用冲突等诊断可能有多种等价报法，顺序不确定）
    hit=0
    while IFS= read -r pat; do
        [ -z "$pat" ] && continue
        if grep -qF -- "$pat" <<<"$out"; then
            hit=1
            break
        fi
    done < "$base.expected"
    if [ "$hit" != "1" ]; then
        echo "FAIL $f: none of the expected substrings found:"
        sed 's/^/  - /' "$base.expected"
        fail=$((fail + 1))
    fi
done

rt=0
while read -r name code pat; do
    [ -z "${name:-}" ] && continue
    case "$name" in \#*) continue ;; esac
    rt=$((rt + 1))
    out=$(timeout 60 "$BIN" run "tests/runtime_safety/$name" 2>&1)
    got=$?
    if [ "$got" != "$code" ]; then
        echo "FAIL tests/runtime_safety/$name: exit $got, want $code"
        fail=$((fail + 1))
    fi
    if [ -n "${pat:-}" ] && ! grep -qF -- "$pat" <<<"$out"; then
        echo "FAIL tests/runtime_safety/$name: output missing: $pat"
        fail=$((fail + 1))
    fi
done < tests/runtime_safety/manifest.txt

# 交叉/裸机：freestanding 只产出 aarch64 .o（不链接 runtime/libc/PIE）
if [ -f tests/baremetal/ayanami.toml ]; then
    rm -f build/baremetal.o build/baremetal.ll build/baremetal.lcl build/baremetal
    "$BIN" build tests/baremetal/baremetal.aya >/dev/null 2>&1
    got=$?
    if [ "$got" != 0 ]; then
        echo "FAIL tests/baremetal: freestanding build failed"
        fail=$((fail + 1))
    elif [ ! -f build/baremetal.o ]; then
        echo "FAIL tests/baremetal: missing build/baremetal.o"
        fail=$((fail + 1))
    elif [ -e build/baremetal ] || [ -e build/baremetal.lcl ] || [ -e build/baremetal.ll ]; then
        echo "FAIL tests/baremetal: freestanding produced non-object artifact"
        fail=$((fail + 1))
    elif ! LC_ALL=C readelf -h build/baremetal.o 2>/dev/null | grep -q "AArch64"; then
        echo "FAIL tests/baremetal: build/baremetal.o is not AArch64"
        fail=$((fail + 1))
    elif LC_ALL=C readelf -r build/baremetal.o 2>/dev/null | grep -q "GOT"; then
        echo "FAIL tests/baremetal: GOT/PIE relocation present"
        fail=$((fail + 1))
    fi
    rm -f build/baremetal.o build/baremetal.ll build/baremetal.lcl
fi

# 交叉 cfg：check 必须按 ayanami.toml [build] arch 解析 arch/target
if [ -d tests/cfg_arch ]; then
    if ! "$BIN" check tests/cfg_arch >/dev/null 2>&1; then
        echo "FAIL tests/cfg_arch: check did not apply [build] arch/target to cfg"
        fail=$((fail + 1))
    fi
fi

# #213：static-lib 粗粒度增量缓存（源码不变复用；源码变化失效）
if [ -f tests/incremental/incremental.aya ]; then
    inc_tmp=$(mktemp /tmp/ayanami-incr-XXXXXX.aya)
    cp tests/incremental/incremental.aya "$inc_tmp"
    inc_stem=$(basename "$inc_tmp" .aya)
    inc_lib="build/lib${inc_stem}.a"
    inc_stamp="build/${inc_stem}.static-lib.stamp"
    rm -f "$inc_lib" "$inc_stamp"
    if ! "$BIN" build "$inc_tmp" >/dev/null 2>&1; then
        echo "FAIL #213: first static-lib build failed"
        fail=$((fail + 1))
    else
        t1=$(stat -c %Y "$inc_lib" 2>/dev/null || echo "")
        sleep 1
        "$BIN" build "$inc_tmp" >/dev/null 2>&1
        t2=$(stat -c %Y "$inc_lib" 2>/dev/null || echo "")
        if [ -z "$t1" ] || [ "$t1" != "$t2" ]; then
            echo "FAIL #213: unchanged source did not reuse static-lib"
            fail=$((fail + 1))
        fi
        printf '\n// cache bust\n' >> "$inc_tmp"
        "$BIN" build "$inc_tmp" >/dev/null 2>&1
        t3=$(stat -c %Y "$inc_lib" 2>/dev/null || echo "")
        if [ -n "$t2" ] && [ "$t2" = "$t3" ]; then
            echo "FAIL #213: changed source did not invalidate cache"
            fail=$((fail + 1))
        fi
    fi
    rm -f "$inc_tmp" "$inc_lib" "$inc_stamp"
fi

# #214：并发宏插件缓存不得读到半写 .so
if [ -f example/test_macro_fn.aya ]; then
    mrace=$(mktemp -d)
    for _ in 1 2 3; do
        rm -rf "$mrace/ayanami-macros"
        TMPDIR="$mrace" "$BIN" check example/test_macro_fn.aya >/dev/null 2>&1 & p1=$!
        TMPDIR="$mrace" "$BIN" check example/test_macro_fn.aya >/dev/null 2>&1 & p2=$!
        wait $p1; r1=$?
        wait $p2; r2=$?
        if [ "$r1" -ne 0 ] || [ "$r2" -ne 0 ]; then
            echo "FAIL #214: concurrent macro plugin build ($r1/$r2)"
            fail=$((fail + 1))
            break
        fi
    done
    rm -rf "$mrace"
fi

# A5h：第三方插件作者流程（plugin new → package → install → consumer）
pbase=$(mktemp -d)
pdir="$pbase/demo_plugin"
cproj="$pbase/consumer"
bin_abs=$(cd "$(dirname "$BIN")" && pwd)/$(basename "$BIN")
if ! "$BIN" plugin new "$pdir" >/dev/null 2>&1; then
    echo "FAIL A5h: plugin new failed"
    fail=$((fail + 1))
elif ! "$BIN" plugin package "$pdir" >/dev/null 2>&1; then
    echo "FAIL A5h: plugin package failed"
    fail=$((fail + 1))
elif [ ! -f "$pdir/src/lib.lcl" ]; then
    echo "FAIL A5h: plugin package did not produce src/lib.lcl"
    fail=$((fail + 1))
elif ! grep -aq '^\[plugin\]' "$pdir/src/lib.lcl"; then
    echo "FAIL A5j-1: plugin package did not write [plugin] manifest"
    fail=$((fail + 1))
elif ! "$BIN" new "$cproj" >/dev/null 2>&1; then
    echo "FAIL A5h: consumer project creation failed"
    fail=$((fail + 1))
elif ! (cd "$cproj" && "$bin_abs" plugin install "$pdir/src/lib.lcl" demo >/dev/null 2>&1); then
    echo "FAIL A5h: plugin install failed"
    fail=$((fail + 1))
elif [ ! -f "$cproj/plugins/demo.lcl" ]; then
    echo "FAIL A5h: plugin install did not place plugins/demo.lcl"
    fail=$((fail + 1))
else
    # A5j-1：不兼容 ABI 必须在 install 阶段被拒绝
    { head -c 512 "$pdir/src/lib.lcl" | sed 's/abi=8/abi=999/'; tail -c +513 "$pdir/src/lib.lcl"; } > "$pbase/bad_abi.lcl"
    if (cd "$cproj" && "$bin_abs" plugin install "$pbase/bad_abi.lcl" bad >/dev/null 2>&1); then
        echo "FAIL A5j-1: incompatible plugin ABI was installed"
        fail=$((fail + 1))
    fi
    cat > "$cproj/src/main.aya" <<'EOF'
import "demo"

#[demo::identity]
fn marked() -> int {
    return 7
}

fn main() -> int {
    if marked() != 7 { return 1 }
    return 0
}
EOF
    if ! "$BIN" run "$cproj" >/dev/null 2>&1; then
        echo "FAIL A5h: consumer could not run installed third-party plugin"
        fail=$((fail + 1))
    fi

    # A5e/B1：第三方 #[transform] identity（JSON HIR 插件通路）
    tdir="$pbase/transform_lib"
    tconsumer="$pbase/transform_consumer.aya"
    if ! "$BIN" plugin new "$tdir" >/dev/null 2>&1; then
        echo "FAIL A5e/B1: transform plugin new failed"
        fail=$((fail + 1))
    else
        cat > "$tdir/src/lib.aya" <<'EOF'
#[transform]
pub fn identity(String input) -> String {
    return input
}
EOF
        if ! "$BIN" plugin package "$tdir" >/dev/null 2>&1; then
            echo "FAIL A5e/B1: transform plugin package failed"
            fail=$((fail + 1))
        elif ! grep -aq '^transform="identity"' "$tdir/src/lib.lcl"; then
            echo "FAIL A5e/B1: transform table missing"
            fail=$((fail + 1))
        else
            cat > "$tconsumer" <<EOF
import "$tdir/src/lib.lcl"

#[lib::identity]
fn marked() -> int {
    return 7
}

fn main() -> int {
    if marked() != 7 { return 1 }
    return 0
}
EOF
            if ! "$BIN" run "$tconsumer" >/dev/null 2>&1; then
                echo "FAIL A5e/B1: transform consumer failed"
                fail=$((fail + 1))
            fi
            # B2：插件实际修改 HIR JSON（把字面量 7 改成 8）
            cat > "$tdir/src/lib.aya" <<'EOF'
import "hir"

#[transform]
#[alloc]
#[state]
pub fn bump(String input) -> String {
    doc = parse_hir(ref input).unwrap()
    body = single_body(make_return(make_int(8)))
    result = replace_function_body(ref doc, ref "main", body).unwrap()
    return render_hir(ref result)
}
EOF
            if ! "$BIN" plugin package "$tdir" >/dev/null 2>&1; then
                echo "FAIL A5e/B2: mutating transform package failed"
                fail=$((fail + 1))
            else
                cat > "$tconsumer" <<EOF
import "$tdir/src/lib.lcl"

#[lib::bump]
fn main() -> int {
    return 7
}
EOF
                "$BIN" run "$tconsumer" >/dev/null 2>&1
                got=$?
                if [ "$got" != 8 ]; then
                    echo "FAIL A5e/B2: mutated HIR exit $got, want 8"
                    fail=$((fail + 1))
                fi
                # B2c：clone + append 一个真实函数 item，验证 FnId/结构校验
                cat > "$tdir/src/lib.aya" <<'EOF'
import "hir"

#[transform]
#[alloc]
#[state]
pub fn clone_main(String input) -> String {
    doc = parse_hir(ref input).unwrap()
    item = find_function(ref doc, ref "main").unwrap()
    next = max_fn_id(ref doc) + 1
    extra = clone_function(ref item, ref "unused", next).unwrap()
    result = append_item(ref doc, extra)
    return render_hir(ref result)
}
EOF
                if ! "$BIN" plugin package "$tdir" >/dev/null 2>&1; then
                    echo "FAIL A5e/B2c: clone transform package failed"
                    fail=$((fail + 1))
                else
                    cat > "$tconsumer" <<EOF
import "$tdir/src/lib.lcl"

#[lib::clone_main]
fn main() -> int {
    return 7
}
EOF
                    "$BIN" run "$tconsumer" >/dev/null 2>&1
                    got=$?
                    if [ "$got" != 7 ]; then
                        echo "FAIL A5e/B2c: appended function exit $got, want 7"
                        fail=$((fail + 1))
                    fi
                fi
                # B2d：改调用点 add2 → sub2
                cat > "$tdir/src/lib.aya" <<'EOF'
import "hir"

#[transform]
#[alloc]
#[state]
pub fn redirect(String input) -> String {
    doc = parse_hir(ref input).unwrap()
    from = function_id(ref doc, ref "add2").unwrap()
    to = function_id(ref doc, ref "sub2").unwrap()
    result = replace_call_targets(ref doc, from, to)
    return render_hir(ref result)
}
EOF
                if ! "$BIN" plugin package "$tdir" >/dev/null 2>&1; then
                    echo "FAIL A5e/B2d: redirect transform package failed"
                    fail=$((fail + 1))
                else
                    cat > "$tconsumer" <<EOF
import "$tdir/src/lib.lcl"

#[lib::redirect]
fn main() -> int {
    return add2(4, 2)
}

fn add2(int a, int b) -> int { return a + b }
fn sub2(int a, int b) -> int { return a - b }
EOF
                    "$BIN" run "$tconsumer" >/dev/null 2>&1
                    got=$?
                    if [ "$got" != 2 ]; then
                        echo "FAIL A5e/B2d: redirect call exit $got, want 2"
                        fail=$((fail + 1))
                    fi
                    # B2e：transform 只能标注函数，标注 struct 必须有诊断
                    bad="$pbase/transform_bad.aya"
                    cat > "$bad" <<EOF
import "$tdir/src/lib.lcl"

#[lib::redirect]
enum Bad { A, B }

fn main() -> int { return 0 }
EOF
                    if "$BIN" check "$bad" >/tmp/transform_bad.out 2>&1; then
                        echo "FAIL A5e/B2e: transform on struct was accepted"
                        fail=$((fail + 1))
                    elif ! grep -q "only supported on functions and non-generic structs" /tmp/transform_bad.out; then
                        echo "FAIL A5e/B2e: missing unsupported-target diagnostic"
                        fail=$((fail + 1))
                    fi
                fi
                # B2f：非泛型 struct 目标可触发 transform
                cat > "$tdir/src/lib.aya" <<'EOF'
import "hir"

#[transform]
#[alloc]
#[state]
pub fn struct_bump(String input) -> String {
    doc = parse_hir(ref input).unwrap()
    body = single_body(make_return(make_int(8)))
    result = replace_function_body(ref doc, ref "main", body).unwrap()
    return render_hir(ref result)
}
EOF
                if ! "$BIN" plugin package "$tdir" >/dev/null 2>&1; then
                    echo "FAIL A5e/B2f: struct transform package failed"
                    fail=$((fail + 1))
                else
                    cat > "$tconsumer" <<EOF
import "$tdir/src/lib.lcl"

#[lib::struct_bump]
struct Point {
    int x
}

fn main() -> int { return 7 }
EOF
                    "$BIN" run "$tconsumer" >/dev/null 2>&1
                    got=$?
                    if [ "$got" != 8 ]; then
                        echo "FAIL A5e/B2f: struct transform exit $got, want 8"
                        fail=$((fail + 1))
                    fi
                fi
                # B2g：泛型函数只在特化 HirFn 上触发（per-specialization）
                cat > "$tdir/src/lib.aya" <<'EOF'
import "hir"

#[transform]
#[alloc]
#[state]
pub fn bump(String input) -> String {
    doc = parse_hir(ref input).unwrap()
    body = single_body(make_return(make_int(8)))
    result = replace_function_body(ref doc, ref "main", body).unwrap()
    return render_hir(ref result)
}
EOF
                if ! "$BIN" plugin package "$tdir" >/dev/null 2>&1; then
                    echo "FAIL A5e/B2g: generic transform package failed"
                    fail=$((fail + 1))
                else
                    cat > "$tconsumer" <<EOF
import "$tdir/src/lib.lcl"

#[lib::bump]
fn identity[T](T x) -> T { return x }

fn main() -> int { return identity(7) }
EOF
                    "$BIN" run "$tconsumer" >/dev/null 2>&1
                    got=$?
                    if [ "$got" != 8 ]; then
                        echo "FAIL A5e/B2g: generic specialization exit $got, want 8"
                        fail=$((fail + 1))
                    fi
                fi
            fi
        fi
    fi

    # A5j-2：本地 registry + lockfile + 离线 cache
    reg="$pbase/registry"
    v2="$pbase/demo_plugin_v2"
    rproj="$pbase/registry_consumer"
    phome="$pbase/plugin-home"
    cp -r "$pdir" "$v2"
    sed -i 's/version = "0.1.0"/version = "0.2.0"/' "$v2/ayanami.toml"
    "$BIN" plugin package "$v2" >/dev/null 2>&1 || true
    if ! "$BIN" plugin publish "$pdir/src/lib.lcl" --registry "$reg" >/dev/null 2>&1; then
        echo "FAIL A5j-2b: plugin publish 0.1.0 failed"
        fail=$((fail + 1))
    fi
    if ! "$BIN" plugin publish "$v2/src/lib.lcl" --registry "$reg" >/dev/null 2>&1; then
        echo "FAIL A5j-2b: plugin publish 0.2.0 failed"
        fail=$((fail + 1))
    fi
    if ! grep -q '0.2.0 = "demo_plugin-0.2.0.lcl"' "$reg/index.toml"; then
        echo "FAIL A5j-2b: registry index missing published version"
        fail=$((fail + 1))
    fi
    if ! "$BIN" new "$rproj" >/dev/null 2>&1; then
        echo "FAIL A5j-2: registry consumer creation failed"
        fail=$((fail + 1))
    elif ! printf '\n[plugins]\nregistry = "%s"\n' "$reg" >> "$rproj/ayanami.toml"; then
        echo "FAIL A5j-2: could not configure registry"
        fail=$((fail + 1))
    elif ! (cd "$rproj" && env AYANAMI_PLUGIN_HOME="$phome" "$bin_abs" plugin install demo_plugin@0.1.0 >/dev/null 2>&1); then
        echo "FAIL A5j-2: registry install failed"
        fail=$((fail + 1))
    elif ! grep -q 'version="0.1.0"' "$rproj/ayanami.lock"; then
        echo "FAIL A5j-2: lockfile did not record 0.1.0"
        fail=$((fail + 1))
    elif (cd "$rproj" && env AYANAMI_PLUGIN_HOME="$phome" "$bin_abs" plugin install demo_plugin@0.2.0 >/dev/null 2>&1); then
        echo "FAIL A5j-2: locked 0.1.0 was silently replaced by 0.2.0"
        fail=$((fail + 1))
    elif ! (cd "$rproj" && env AYANAMI_PLUGIN_HOME="$phome" "$bin_abs" plugin install demo_plugin@0.2.0 --update >/dev/null 2>&1); then
        echo "FAIL A5j-2: --update install failed"
        fail=$((fail + 1))
    elif ! grep -q 'version="0.2.0"' "$rproj/ayanami.lock"; then
        echo "FAIL A5j-2: lockfile did not record 0.2.0 after --update"
        fail=$((fail + 1))
    else
        cat > "$rproj/src/main.aya" <<'EOF'
import "demo_plugin"

#[demo_plugin::identity]
fn marked() -> int { return 7 }

fn main() -> int {
    if marked() != 7 { return 1 }
    return 0
}
EOF
        if ! "$BIN" run "$rproj" >/dev/null 2>&1; then
            echo "FAIL A5j-2: registry-installed plugin could not run"
            fail=$((fail + 1))
        fi
        if ! (cd "$rproj" && env AYANAMI_PLUGIN_HOME="$phome" "$bin_abs" plugin verify >/dev/null 2>&1); then
            echo "FAIL A5j-2: plugin verify failed"
            fail=$((fail + 1))
        fi
        printf 'x' >> "$rproj/plugins/demo_plugin.lcl"
        if (cd "$rproj" && env AYANAMI_PLUGIN_HOME="$phome" "$bin_abs" plugin verify >/dev/null 2>&1); then
            echo "FAIL A5j-2: plugin verify accepted a tampered .lcl"
            fail=$((fail + 1))
        fi
        mv "$reg" "$pbase/registry.offline"
        rm -f "$rproj/plugins/demo_plugin.lcl"
        if ! (cd "$rproj" && env AYANAMI_PLUGIN_HOME="$phome" AYANAMI_PLUGIN_REGISTRY="$reg" "$bin_abs" plugin install demo_plugin >/dev/null 2>&1); then
            echo "FAIL A5j-2: offline cache restore failed"
            fail=$((fail + 1))
        fi
        if ! (cd "$rproj" && env AYANAMI_PLUGIN_HOME="$phome" "$bin_abs" plugin verify >/dev/null 2>&1); then
            echo "FAIL A5j-2: restored plugin verify failed"
            fail=$((fail + 1))
        fi
    fi
fi
rm -rf "$pbase"

# A5e/B0：HIR JSON schema identity roundtrip（代表性节点覆盖）
for f in test_struct test_enum_boxed test_async_await test_pass test_iface_box test_match_destructure test_asm; do
    if [ -f "example/$f.aya" ]; then
        if ! AYANAMI_HIR_SCHEMA=1 "$BIN" check "example/$f.aya" >/dev/null 2>&1; then
            echo "FAIL A5e/B0: HIR schema roundtrip failed for example/$f.aya"
            fail=$((fail + 1))
        fi
    fi
done

# #106：零参函数不得发射裸符号（避免与 libc 冲突）
if [ -f build/test_zero_arg_fn ]; then
    if nm build/test_zero_arg_fn 2>/dev/null | grep -qE ' T time$'; then
        echo "FAIL #106: zero-arg fn emitted bare symbol 'time'"
        fail=$((fail + 1))
    fi
fi

# #117：溢出检查合成位置串不计 alloc；合成标记不算未知 extern
if [ -d tests/effects ]; then
    "$BIN" check --verify-effects tests/effects/pure_arith.aya >/dev/null 2>&1
    got=$?
    if [ "$got" != 0 ]; then
        echo "FAIL tests/effects/pure_arith.aya: exit $got, want 0（#117 纯函数算术误报）"
        fail=$((fail + 1))
    fi
    "$BIN" check --verify-effects tests/effects/pure_real_alloc.aya >/dev/null 2>&1
    got=$?
    if [ "$got" = 0 ]; then
        echo "FAIL tests/effects/pure_real_alloc.aya: want verify failure（#117 真 alloc 应仍报错）"
        fail=$((fail + 1))
    fi
    # #176：边界检查合成调用不应破坏 #[pure]
    "$BIN" check --verify-effects tests/effects/pure_bounds.aya >/dev/null 2>&1
    got=$?
    if [ "$got" != 0 ]; then
        echo "FAIL tests/effects/pure_bounds.aya: exit $got, want 0（#176 越界检查纯函数误报）"
        fail=$((fail + 1))
    fi
fi

# #179：fmt 保留注释（文件头/文档/行尾/字段/变体）+ 幂等 + 格式化后可编译
if [ -d tests/fmt ]; then
    for f in tests/fmt/*.aya; do
        [ -e "$f" ] || continue
        fdir=$(mktemp -d)
        cp "$f" "$fdir/t.aya"
        "$BIN" fmt "$fdir/t.aya" >/dev/null 2>&1
        before=$(grep -c '//\|/\*' "$f" || true)
        after=$(grep -c '//\|/\*' "$fdir/t.aya" || true)
        if [ "$after" -lt "$before" ]; then
            echo "FAIL $f: fmt 丢失注释（$before → $after）"
            fail=$((fail + 1))
        fi
        cp "$fdir/t.aya" "$fdir/t2.aya"
        "$BIN" fmt "$fdir/t2.aya" >/dev/null 2>&1
        if ! cmp -s "$fdir/t.aya" "$fdir/t2.aya" >/dev/null; then
            echo "FAIL $f: fmt 非幂等"
            fail=$((fail + 1))
        fi
        if ! "$BIN" check "$fdir/t.aya" >/dev/null 2>&1; then
            echo "FAIL $f: 格式化结果不可编译"
            fail=$((fail + 1))
        fi
        rm -rf "$fdir"
    done
fi

# #84 ③：导出 C 符号 + C harness 互操作
if [ -d tests/c_export ]; then
    ce_dir="tests/c_export"
    root="$PWD"
    (cd "$ce_dir" && "$root/$BIN" build mylib.aya >/dev/null 2>&1)
    if gcc -no-pie "$ce_dir/harness.c" "$ce_dir/build/libmylib.a" "$root/src/runtime.c" -lm -o "$ce_dir/harness" >/dev/null 2>&1; then
        "$ce_dir/harness"
        got=$?
        if [ "$got" != 0 ]; then
            echo "FAIL $ce_dir/harness.c: exit $got, want 0（#84 C 导出互操作）"
            fail=$((fail + 1))
        fi
    else
        echo "FAIL $ce_dir: gcc harness link failed（#84 C 导出互操作）"
        fail=$((fail + 1))
    fi
    rm -rf "$ce_dir/build" "$ce_dir/harness"
fi

# #84 ②：自定义 runtime（[runtime] path / AYANAMI_RUNTIME，.c/.a）
if [ -d tests/runtime_custom ]; then
    rc_dir="tests/runtime_custom"
    root="$PWD"
    bin_abs="$root/$BIN"
    (cd "$rc_dir" && "$bin_abs" run main.aya >/dev/null 2>&1); got=$?
    if [ "$got" != 42 ]; then
        echo "FAIL $rc_dir/main.aya: exit $got, want 42（#84 [runtime] path .c）"
        fail=$((fail + 1))
    fi
    (cd "$rc_dir" && AYANAMI_RUNTIME="$root/$rc_dir/alt_runtime.c" "$bin_abs" run main.aya >/dev/null 2>&1); got=$?
    if [ "$got" != 43 ]; then
        echo "FAIL $rc_dir/main.aya: exit $got, want 43（#84 AYANAMI_RUNTIME 覆盖）"
        fail=$((fail + 1))
    fi
    (cd "$rc_dir" && gcc -c custom_runtime.c -o custom_runtime.o && ar rcs libcustom.a custom_runtime.o) >/dev/null 2>&1
    (cd "$rc_dir" && AYANAMI_RUNTIME="$root/$rc_dir/libcustom.a" "$bin_abs" run main.aya >/dev/null 2>&1); got=$?
    if [ "$got" != 42 ]; then
        echo "FAIL $rc_dir/main.aya: exit $got, want 42（#84 AYANAMI_RUNTIME .a）"
        fail=$((fail + 1))
    fi
    rm -rf "$rc_dir/build" "$rc_dir/custom_runtime.o" "$rc_dir/libcustom.a"
fi

# #129：分别打包的 .lcl 各自实例化同一泛型函数不得重复定义（弱链接去重）
if [ -d tests/lcl_weak ]; then
    wdir=tests/lcl_weak
    root="$PWD"
    "$BIN" package "$wdir/a.aya" >/dev/null 2>&1 || true
    "$BIN" package "$wdir/b.aya" >/dev/null 2>&1 || true
    if [ -f "$wdir/a.lcl" ] && [ -f "$wdir/b.lcl" ]; then
        cp "$wdir/a.lcl" "$wdir/b.lcl" "$(dirname "$BIN")/std/" 2>/dev/null || true
        timeout 120 "$BIN" run "$wdir/entry/main.aya" >/dev/null 2>&1
        got=$?
        if [ "$got" != 0 ]; then
            echo "FAIL $wdir/entry/main.aya: exit $got, want 0（#129 .lcl 泛型实例化弱链接）"
            fail=$((fail + 1))
        fi
        rm -f "$(dirname "$BIN")/std/a.lcl" "$(dirname "$BIN")/std/b.lcl"
    else
        echo "FAIL $wdir: package failed（#129 夹具）"
        fail=$((fail + 1))
    fi
    rm -rf "$root/build" "$wdir/a.lcl" "$wdir/b.lcl"
fi

# #157：extern C 声明实参借用（ref 形参不标 readonly），release 运行读回值正确
if [ -f example/test_extern_out_param.aya ]; then
    "$BIN" run --release example/test_extern_out_param.aya >/dev/null 2>&1
    got=$?
    if [ "$got" != 0 ]; then
        echo "FAIL #157: release extern out-param 读回错误（exit $got）"
        fail=$((fail + 1))
    fi
fi

# #159/#149：参数化接口约束（I: Iterator[T]）与多约束（T: A + B）打包后方法可用
for pd in tests/lcl_param_constraint tests/lcl_multi_constraint tests/lcl_extern_arr; do
    [ -d "$pd" ] || continue
    root="$PWD"
    "$BIN" package "$pd/lib.aya" >/dev/null 2>&1 || true
    if [ -f "$pd/lib.lcl" ]; then
        cp "$pd/lib.lcl" "$(dirname "$BIN")/std/" 2>/dev/null || true
        timeout 120 "$BIN" run "$pd/main.aya" >/dev/null 2>&1
        got=$?
        if [ "$got" != 0 ]; then
            echo "FAIL $pd/main.aya: exit $got, want 0（泛型约束 .lcl 夹具）"
            fail=$((fail + 1))
        fi
        rm -f "$(dirname "$BIN")/std/lib.lcl"
    else
        echo "FAIL $pd: package failed（泛型约束夹具）"
        fail=$((fail + 1))
    fi
    rm -rf "$root/build" "$pd/lib.lcl"
done

# M-opt.1：debug/release 模式差异（溢出检查 / 推断 noalias、nounwind）
if [ -f example/test_release_opt.aya ]; then
    "$BIN" build example/test_release_opt.aya >/dev/null 2>&1
    if ! grep -q '__ayanami_ovf_' build/test_release_opt.ll; then
        echo "FAIL M-opt.1: debug 构建缺少溢出检查"
        fail=$((fail + 1))
    fi
    "$BIN" build --release example/test_release_opt.aya >/dev/null 2>&1
    if grep -q '__ayanami_ovf_' build/test_release_opt.ll; then
        echo "FAIL M-opt.1: release 构建仍含溢出检查"
        fail=$((fail + 1))
    fi
    if ! grep -q 'noalias' build/test_release_opt.ll; then
        echo "FAIL M-opt.1: release 构建缺少推断 noalias"
        fail=$((fail + 1))
    fi
    if ! grep -q 'nounwind' build/test_release_opt.ll; then
        echo "FAIL M-opt.1: release 构建缺少 nounwind"
        fail=$((fail + 1))
    fi
    # M-opt.2：release 非导出函数 internal；pub 函数保持外部
    if ! grep -q 'define internal' build/test_release_opt.ll; then
        echo "FAIL M-opt.2: release 构建缺少 internal 非导出函数"
        fail=$((fail + 1))
    fi
    if grep 'define.*@public_double_int' build/test_release_opt.ll | grep -q 'internal'; then
        echo "FAIL M-opt.2: release 下 pub 函数被错误内部化"
        fail=$((fail + 1))
    fi
    # M-opt.6：release 推断 ref 非空/只读与拥有返回值 noalias
    if ! grep -q 'nonnull' build/test_release_opt.ll; then
        echo "FAIL M-opt.6: release 构建缺少推断 nonnull"
        fail=$((fail + 1))
    fi
    if ! grep -q 'readonly' build/test_release_opt.ll; then
        echo "FAIL M-opt.6: release 构建缺少推断 readonly"
        fail=$((fail + 1))
    fi
    if ! grep -q 'define internal noalias ptr @make_int' build/test_release_opt.ll; then
        echo "FAIL M-opt.6: release 拥有返回值缺少 noalias"
        fail=$((fail + 1))
    fi
    "$BIN" build example/test_release_opt.aya >/dev/null 2>&1
    if grep -q 'define internal' build/test_release_opt.ll; then
        echo "FAIL M-opt.2: debug 构建不应有 internal"
        fail=$((fail + 1))
    fi
    if grep 'define' build/test_release_opt.ll | grep -qE 'nonnull|readonly'; then
        echo "FAIL M-opt.6: debug 构建不应有推断 nonnull/readonly"
        fail=$((fail + 1))
    fi
    timeout 60 "$BIN" run --release example/test_release_opt.aya >/dev/null 2>&1
    got=$?
    if [ "$got" != 0 ]; then
        echo "FAIL M-opt.1: release run exit $got, want 0"
        fail=$((fail + 1))
    fi
fi

# #100：源模块调用 .lcl 泛型方法不得重复单态化（multiple definition）
if [ -d tests/lcl_mono ]; then
    mono_dir=tests/lcl_mono
    "$BIN" package "$mono_dir/mlib.aya" >/dev/null 2>&1 || true
    if [ -f "$mono_dir/mlib.lcl" ]; then
        cp "$mono_dir/mlib.lcl" "$(dirname "$BIN")/std/" 2>/dev/null || true
        timeout 120 "$BIN" run "$mono_dir/entry.aya" >/dev/null 2>&1
        got=$?
        if [ "$got" != 0 ]; then
            echo "FAIL $mono_dir/entry.aya: exit $got, want 0（#100 .lcl 泛型特化重复定义）"
            fail=$((fail + 1))
        fi
        rm -f "$(dirname "$BIN")/std/mlib.lcl"
    else
        echo "FAIL $mono_dir/mlib.aya: package failed"
        fail=$((fail + 1))
    fi
fi

# #104：传递 .lcl 依赖（libb → liba）的符号/泛型 impl 注册
if [ -d tests/lcl_transitive ]; then
    tdir=tests/lcl_transitive
    "$BIN" package "$tdir/liba.aya" >/dev/null 2>&1 || true
    cp "$tdir/liba.lcl" "$(dirname "$BIN")/std/" 2>/dev/null || true
    "$BIN" package "$tdir/libb.aya" >/dev/null 2>&1 || true
    cp "$tdir/libb.lcl" "$(dirname "$BIN")/std/" 2>/dev/null || true
    timeout 120 "$BIN" run "$tdir/entry.aya" >/dev/null 2>&1
    got=$?
    if [ "$got" != 0 ]; then
        echo "FAIL $tdir/entry.aya: exit $got, want 0（#104 传递 .lcl 依赖注册）"
        fail=$((fail + 1))
    fi
    rm -f "$(dirname "$BIN")/std/liba.lcl" "$(dirname "$BIN")/std/libb.lcl"
fi

# nsr：跨模块同名私有函数（模块私有符号不进入导入方符号表）
if [ -d tests/lcl_nsr ]; then
    ndir=tests/lcl_nsr
    "$BIN" package "$ndir/lib.aya" >/dev/null 2>&1 || true
    cp "$ndir/lib.lcl" "$(dirname "$BIN")/std/" 2>/dev/null || true
    timeout 120 "$BIN" run "$ndir/entry.aya" >/dev/null 2>&1
    got=$?
    if [ "$got" != 0 ]; then
        echo "FAIL $ndir/entry.aya: exit $got, want 0（nsr 跨模块同名私有函数）"
        fail=$((fail + 1))
    fi

# M8：缺失依赖 .lcl 清晰诊断（此前静默跳过 → 裸链接错误 undefined reference）
if [ -d tests/lcl_missing_dep ]; then
    mdir=tests/lcl_missing_dep
    "$BIN" package "$mdir/src/libb.aya" >/dev/null 2>&1 || true
    cp "$mdir/src/libb.lcl" "$(dirname "$BIN")/std/" 2>/dev/null || true
    out=$(timeout 120 "$BIN" run "$mdir/entry/entry.aya" 2>&1)
    got=$?
    if [ "$got" = 0 ] || ! printf '%s' "$out" | grep -q "depends on 'liba'"; then
        echo "FAIL $mdir/entry/entry.aya: 期望缺失依赖诊断（got=$got）"
        fail=$((fail + 1))
    fi

# M8：版本漂移告警（陈旧 .lcl；篡改版本行后导入应告警且不阻断）
if [ -d tests/lcl_version_drift ]; then
    vdir=tests/lcl_version_drift
    "$BIN" package "$vdir/lib.aya" >/dev/null 2>&1 || true
    sed -i 's/version="[^"]*"/version="0.0.0"/' "$vdir/lib.lcl" 2>/dev/null || true
    cp "$vdir/lib.lcl" "$(dirname "$BIN")/std/" 2>/dev/null || true
    out=$(timeout 120 "$BIN" run "$vdir/entry/entry.aya" 2>&1)
    got=$?
    if [ "$got" != 0 ] || ! printf '%s' "$out" | grep -q "was built with ayanami 0.0.0"; then
        echo "FAIL $vdir/entry/entry.aya: 期望版本漂移告警（got=$got）"
        fail=$((fail + 1))
    fi
    rm -f "$vdir/lib.lcl" "$(dirname "$BIN")/std/lib.lcl"
fi

# M8：feature flags（#[cfg(feature = "x")]；AYANAMI_FEATURES 与 ayanami.toml features）
if [ -d tests/cfg_feature ]; then
    fdir=tests/cfg_feature
    timeout 120 "$BIN" run "$fdir/entry.aya" >/dev/null 2>&1
    got_off=$?
    AYANAMI_FEATURES=extra timeout 120 "$BIN" run "$fdir/entry.aya" >/dev/null 2>&1
    got_on=$?
    if [ "$got_off" != 1 ] || [ "$got_on" != 42 ]; then
        echo "FAIL $fdir/entry.aya: feature off=$got_off（want 1）/ on=$got_on（want 42）"
        fail=$((fail + 1))
    fi
fi
    rm -f "$mdir/src/libb.lcl" "$(dirname "$BIN")/std/libb.lcl"
fi
    rm -f "$(dirname "$BIN")/std/lib.lcl"
fi

# M6.1b：pub const 跨 .lcl 导出/导入 + 命名空间 const
if [ -d tests/lcl_const ]; then
    tdir=tests/lcl_const
    "$BIN" package "$tdir/lib.aya" >/dev/null 2>&1 || true
    cp "$tdir/lib.lcl" "$(dirname "$BIN")/std/" 2>/dev/null || true
    timeout 120 "$BIN" run "$tdir/main.aya" >/dev/null 2>&1
    got=$?
    if [ "$got" != 0 ]; then
        echo "FAIL $tdir/main.aya: exit $got, want 0（M6.1b pub const 导入）"
        fail=$((fail + 1))
    fi
    rm -f "$(dirname "$BIN")/std/lib.lcl"
fi

# #144：跨 .lcl 嵌套泛型枚举（Option[ArrayList[String]]）签名/ABI
if [ -d tests/lcl_nested ]; then
    tdir=tests/lcl_nested
    "$BIN" package "$tdir/lib.aya" >/dev/null 2>&1 || true
    cp "$tdir/lib.lcl" "$(dirname "$BIN")/std/" 2>/dev/null || true
    timeout 120 "$BIN" run "$tdir/main.aya" >/dev/null 2>&1
    got=$?
    if [ "$got" != 0 ]; then
        echo "FAIL $tdir/main.aya: exit $got, want 0（#144 嵌套泛型 .lcl 签名）"
        fail=$((fail + 1))
    fi
    rm -f "$(dirname "$BIN")/std/lib.lcl"
fi

# #145：顶层泛型自由函数跨 .lcl（无约束 + 带约束）
if [ -d tests/lcl_generic_fn ]; then
    tdir=tests/lcl_generic_fn
    "$BIN" package "$tdir/lib.aya" >/dev/null 2>&1 || true
    cp "$tdir/lib.lcl" "$(dirname "$BIN")/std/" 2>/dev/null || true
    timeout 120 "$BIN" run "$tdir/main.aya" >/dev/null 2>&1
    got=$?
    if [ "$got" != 0 ]; then
        echo "FAIL $tdir/main.aya: exit $got, want 0（#145 顶层泛型自由函数导入）"
        fail=$((fail + 1))
    fi
    rm -f "$(dirname "$BIN")/std/lib.lcl"
fi

# M6.2b：pub static 跨 .lcl 导出/导入（含 static mut 与命名空间）
if [ -d tests/lcl_static ]; then
    tdir=tests/lcl_static
    "$BIN" package "$tdir/lib.aya" >/dev/null 2>&1 || true
    cp "$tdir/lib.lcl" "$(dirname "$BIN")/std/" 2>/dev/null || true
    timeout 120 "$BIN" run "$tdir/main.aya" >/dev/null 2>&1
    got=$?
    if [ "$got" != 0 ]; then
        echo "FAIL $tdir/main.aya: exit $got, want 0（M6.2b pub static 导入）"
        fail=$((fail + 1))
    fi
    rm -f "$(dirname "$BIN")/std/lib.lcl"
fi

echo "regression: positive=$pos negative=$neg runtime=$rt failures=$fail"
[ "$fail" -eq 0 ]
