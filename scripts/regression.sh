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
