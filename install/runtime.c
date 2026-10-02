// Ayanami runtime support library.
// Provides reference counting and I/O utilities.

#include <stdlib.h>
#include <stdint.h>
#include <stdio.h>

#define RC_HEADER(ptr)  (((int64_t *)(ptr)) - 1)

// ──────────────────────────────────────────────
//  Unique ownership (Box-like)
// ──────────────────────────────────────────────

static int64_t live_allocs = 0;

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
