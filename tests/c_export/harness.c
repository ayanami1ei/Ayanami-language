/* #84 ③ 回归：C 侧调用 Ayanami 导出的符号（int = int64_t，ref mut = int64_t*） */
#include <stdint.h>
int64_t aya_add(int64_t a, int64_t b);
int64_t aya_scale(int64_t v, int64_t factor);
void aya_fill(int64_t *dst, int64_t v);

int main(void) {
    if (aya_add(2, 3) != 5) return 1;
    if (aya_scale(6, 7) != 42) return 2;
    int64_t x = 0;
    aya_fill(&x, 9);
    if (x != 9) return 3;
    return 0;
}
