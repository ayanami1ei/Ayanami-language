// Ayanami runtime support library.
// Provides reference counting and I/O utilities.

#include <stdlib.h>
#include <stdint.h>
#include <stdio.h>
#include <math.h>
#include <string.h>

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

/// 打印 `runtime error: <msg>` + `--> file:line:col` + 源码行与插入符，然后退出 101
static void __ayanami_panic_print(long long line, long long col,
                                  const char* file, long file_len,
                                  const char* msg, long msg_len) {
    fflush(stdout);
    fputs("runtime error: ", stderr);
    __ayanami_print_view(msg, msg_len);
    fputc('\n', stderr);
    if (line > 0 && file && file_len > 0) {
        char* path = (char*)malloc((size_t)file_len + 1);
        if (path) {
            memcpy(path, file, (size_t)file_len);
            path[file_len] = 0;
            fprintf(stderr, "  --> %s:%lld:%lld\n", path, line, col);
            // 源码片段（读不到就跳过）
            FILE* f = fopen(path, "r");
            if (f && line > 0) {
                char buf[1024];
                long long n = 0;
                while (fgets(buf, sizeof buf, f)) {
                    n++;
                    if (n == line) {
                        size_t len = strlen(buf);
                        while (len > 0 && (buf[len-1] == '\n' || buf[len-1] == '\r')) buf[--len] = 0;
                        fprintf(stderr, "   |\n%lld | %s\n   | ", line, buf);
                        for (long long i = 1; i < col; i++) fputc(' ', stderr);
                        fputs("^\n", stderr);
                        break;
                    }
                }
                fclose(f);
            }
            free(path);
        }
    }
    fputs("note: runtime panic (exit code 101)\n", stderr);
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
