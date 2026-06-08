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

void __ayanami_print(const char *s) {
    printf("%s", s);
}

void __ayanami_print_ln(const char *s)
{
    printf("%s\n", s);
}
