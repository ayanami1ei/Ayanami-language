// Ayanami runtime support library.
// Provides reference counting and I/O utilities.

#include <stdlib.h>
#include <stdint.h>
#include <stdio.h>
#include <math.h>
#include <string.h>
#include <unistd.h>

#define RC_HEADER(ptr)  (((int64_t *)(ptr)) - 1)

// ──────────────────────────────────────────────
//  Unique ownership (Box-like)
// ──────────────────────────────────────────────

static int64_t live_allocs = 0;

/* A6：标准库辅助 —— int→char 与数学函数 */
char __ayanami_int_to_char(long long c) {
    return (char)c;
}
double __ayanami_sqrt(double x) { return sqrt(x); }
double __ayanami_floor(double x) { return floor(x); }
double __ayanami_ceil(double x) { return ceil(x); }

/* A5d-3b：诊断通道弱符号（可执行文件里为 no-op；插件 shim 提供强定义） */
typedef struct { char* data; long len; } __ayanami_diag_buf;
__attribute__((weak)) void __ayanami_diag_emit(long long level, __ayanami_diag_buf msg) {
    (void)level;
    (void)msg;
}

void *__ayanami_unique_alloc(size_t size) {
    void *p = malloc(size);
    if (p) live_allocs++;
    return p;
}

void __ayanami_unique_free(void *ptr) {
    if (!ptr) return;
    live_allocs--;
    free(ptr);
}

int64_t __ayanami_live_allocs(void) {
    return live_allocs;
}

// ──────────────────────────────────────────────
//  A5c-2：运行时 panic（退出码 101，Rust 风格输出）
// ──────────────────────────────────────────────

typedef struct { const char* data; long len; } __ayanami_str_view;

static void __ayanami_print_view(const char* data, long len) {
    if (data && len > 0) fwrite(data, 1, (size_t)len, stderr);
}

/// 是否给 stderr 上色（TTY 且未设置 NO_COLOR）
static int __ayanami_color_stderr(void) {
    const char* no = getenv("NO_COLOR");
    if (no && *no) return 0;
    return isatty(2);
}

/// Rust 风格 panic：`thread 'main' panicked at file:line:col:`（红色）+ 消息，退出 101
static void __ayanami_panic_print(long long line, long long col,
                                  const char* file, long file_len,
                                  const char* msg, long msg_len) {
    fflush(stdout);
    int color = __ayanami_color_stderr();
    if (line > 0 && file && file_len > 0) {
        char loc[1200];
        int n = (int)(file_len < 1000 ? file_len : 1000);
        snprintf(loc, sizeof loc, "%.*s:%lld:%lld", n, file, line, col);
        if (color) {
            fprintf(stderr, "\x1b[1;31mthread 'main' panicked at %s:\x1b[0m\n", loc);
        } else {
            fprintf(stderr, "thread 'main' panicked at %s:\n", loc);
        }
    } else if (color) {
        fputs("\x1b[1;31mthread 'main' panicked:\x1b[0m\n", stderr);
    } else {
        fputs("thread 'main' panicked:\n", stderr);
    }
    __ayanami_print_view(msg, msg_len);
    fputc('\n', stderr);
    exit(101);
}

void __ayanami_panic_at(long long line, long long col, __ayanami_str_view file, __ayanami_str_view msg) {
    __ayanami_panic_print(line, col, file.data, file.len, msg.data, msg.len);
}

void __ayanami_panic_bounds_at(long long line, long long col, __ayanami_str_view file,
                               long long index, long long len) {
    char buf[160];
    snprintf(buf, sizeof buf, "index out of bounds: the len is %lld but the index is %lld", len, index);
    __ayanami_panic_print(line, col, file.data, file.len, buf, (long)strlen(buf));
}

// ──────────────────────────────────────────────
//  Overflow checks (M1.7, debug builds)
//  `a + b` 等降级为 __ayanami_ovf_{add,sub,mul}_{iN,uN}(a, b, line, col, file)
// ──────────────────────────────────────────────
#define __AYANAMI_OVF(name, T, op, msg) \
    T __ayanami_ovf_##name(T a, T b, long long line, long long col, __ayanami_str_view file) { \
        T r; \
        if (__builtin_##op##_overflow(a, b, &r)) { \
            __ayanami_panic_print(line, col, file.data, file.len, msg, (long)strlen(msg)); \
        } \
        __ayanami_unique_free((void*)file.data); \
        return r; \
    }

__AYANAMI_OVF(add_i8, int8_t, add, "attempt to add with overflow")
__AYANAMI_OVF(add_i16, int16_t, add, "attempt to add with overflow")
__AYANAMI_OVF(add_i32, int32_t, add, "attempt to add with overflow")
__AYANAMI_OVF(add_i64, int64_t, add, "attempt to add with overflow")
__AYANAMI_OVF(add_i128, __int128, add, "attempt to add with overflow")
__AYANAMI_OVF(add_u8, uint8_t, add, "attempt to add with overflow")
__AYANAMI_OVF(add_u16, uint16_t, add, "attempt to add with overflow")
__AYANAMI_OVF(add_u32, uint32_t, add, "attempt to add with overflow")
__AYANAMI_OVF(add_u64, uint64_t, add, "attempt to add with overflow")
__AYANAMI_OVF(add_u128, unsigned __int128, add, "attempt to add with overflow")

__AYANAMI_OVF(sub_i8, int8_t, sub, "attempt to subtract with overflow")
__AYANAMI_OVF(sub_i16, int16_t, sub, "attempt to subtract with overflow")
__AYANAMI_OVF(sub_i32, int32_t, sub, "attempt to subtract with overflow")
__AYANAMI_OVF(sub_i64, int64_t, sub, "attempt to subtract with overflow")
__AYANAMI_OVF(sub_i128, __int128, sub, "attempt to subtract with overflow")
__AYANAMI_OVF(sub_u8, uint8_t, sub, "attempt to subtract with overflow")
__AYANAMI_OVF(sub_u16, uint16_t, sub, "attempt to subtract with overflow")
__AYANAMI_OVF(sub_u32, uint32_t, sub, "attempt to subtract with overflow")
__AYANAMI_OVF(sub_u64, uint64_t, sub, "attempt to subtract with overflow")
__AYANAMI_OVF(sub_u128, unsigned __int128, sub, "attempt to subtract with overflow")

__AYANAMI_OVF(mul_i8, int8_t, mul, "attempt to multiply with overflow")
__AYANAMI_OVF(mul_i16, int16_t, mul, "attempt to multiply with overflow")
__AYANAMI_OVF(mul_i32, int32_t, mul, "attempt to multiply with overflow")
__AYANAMI_OVF(mul_i64, int64_t, mul, "attempt to multiply with overflow")
__AYANAMI_OVF(mul_i128, __int128, mul, "attempt to multiply with overflow")
__AYANAMI_OVF(mul_u8, uint8_t, mul, "attempt to multiply with overflow")
__AYANAMI_OVF(mul_u16, uint16_t, mul, "attempt to multiply with overflow")
__AYANAMI_OVF(mul_u32, uint32_t, mul, "attempt to multiply with overflow")
__AYANAMI_OVF(mul_u64, uint64_t, mul, "attempt to multiply with overflow")
__AYANAMI_OVF(mul_u128, unsigned __int128, mul, "attempt to multiply with overflow")

#undef __AYANAMI_OVF

// ──────────────────────────────────────────────
//  Contracts (A2d)
// ──────────────────────────────────────────────

void __ayanami_require_fail(int64_t line, int64_t col) {
    fprintf(stderr, "requires failed at %ld:%ld\n", (long)line, (long)col);
    abort();
}

void __ayanami_ensure_fail(int64_t line, int64_t col) {
    fprintf(stderr, "ensures failed at %ld:%ld\n", (long)line, (long)col);
    abort();
}

void __ayanami_invariant_fail(int64_t line, int64_t col) {
    fprintf(stderr, "invariant failed at %ld:%ld\n", (long)line, (long)col);
    abort();
}

// ──────────────────────────────────────────────
//  I/O
// ──────────────────────────────────────────────

int __ayanami_getchar(void) {
    return getchar();
}

void __ayanami_putchar(int c) {
    putchar(c);
}

void __ayanami_print_int(int64_t n) {
    printf("%ld", (long)n);
}

void __ayanami_print_str(const char *s, int64_t len) {
    fwrite(s, 1, len, stdout);
}

void __ayanami_print_ln(void) {
    printf("\n");
}

// Return the length needed for formatted float string.
int64_t __ayanami_float_len(double n) {
    char tmp[64];
    return (int64_t)snprintf(tmp, sizeof(tmp), "%g", n);
}

// Write formatted float into a heap buffer of exactly out_len bytes.
char *__ayanami_float_str(double n, int64_t out_len) {
    char *buf = (char *)malloc((size_t)(out_len + 1));
    if (!buf) return NULL;
    snprintf(buf, (size_t)(out_len + 1), "%g", n);
    return buf;
}
