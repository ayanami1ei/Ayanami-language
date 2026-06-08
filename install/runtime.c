// Ayanami runtime support library.
// Provides reference counting and I/O utilities.

#include <stdlib.h>
#include <stdint.h>
#include <stdio.h>

#define RC_HEADER(ptr)  (((int64_t *)(ptr)) - 1)

// ──────────────────────────────────────────────
//  Allocation
// ──────────────────────────────────────────────

void *__ayanami_shared_alloc(size_t size) {
    int64_t *header = (int64_t *)malloc(sizeof(int64_t) + size);
    if (!header) return NULL;
    *header = 1;
    return header + 1;
}

// ──────────────────────────────────────────────
//  Retain / Release
// ──────────────────────────────────────────────

void __ayanami_shared_retain(void *ptr) {
    if (!ptr) return;
    __sync_fetch_and_add(RC_HEADER(ptr), 1);
}

void __ayanami_shared_release(void *ptr) {
    if (!ptr) return;
    int64_t *rc = RC_HEADER(ptr);
    if (__sync_fetch_and_sub(rc, 1) == 1) {
        free(rc);
    }
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

void __ayanami_print_str(const char *s) {
    printf("%s", s);
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
