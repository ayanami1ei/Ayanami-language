
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <math.h>

#define NBODY_STEPS 2000000
#define DAYS 365.24
#define MATMUL_N 1024
#define MATMUL_LEN 1048576
#define SIEVE_N 50000000
#define QSORT_N 2000000
#define MB_W 1024
#define MB_H 1024
#define MB_MAXIT 300

long long mb_total = 0;
long long mb_seed = 0;

double now_millis() {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (double)ts.tv_sec * 1000.0 + (double)ts.tv_nsec / 1000000.0;
}

void nbody_steps(int n, double m[], double x[], double y[], double z[], double vx[], double vy[], double vz[], double dt) {
    for (int s = 0; s < NBODY_STEPS; s++) {
        for (int i = 0; i < n; i++) {
            for (int j = i + 1; j < n; j++) {
                double dx = x[i] - x[j];
                double dy = y[i] - y[j];
                double dz = z[i] - z[j];
                double d2 = dx * dx + dy * dy + dz * dz;
                double mag = dt / (d2 * sqrt(d2));
                vx[i] = vx[i] - dx * m[j] * mag;
                vy[i] = vy[i] - dy * m[j] * mag;
                vz[i] = vz[i] - dz * m[j] * mag;
                vx[j] = vx[j] + dx * m[i] * mag;
                vy[j] = vy[j] + dy * m[i] * mag;
                vz[j] = vz[j] + dz * m[i] * mag;
            }
        }
        for (int i = 0; i < n; i++) {
            x[i] = x[i] + dt * vx[i];
            y[i] = y[i] + dt * vy[i];
            z[i] = z[i] + dt * vz[i];
        }
    }
}

void nbody_run() {
    int n = 5;
    double m[5] = {39.478, 0.03769367487038949, 0.011286326131968767, 0.0017237240570597112, 0.0020336868699246304};
    double x[5] = {0.0, 4.84143144246472090e+00, 8.34336671824457987e+00, 1.28943695621391310e+01, 1.53796971148509165e+01};
    double y[5] = {0.0, -1.16032004402742839e+00, 4.12479856412430479e+00, -1.51111514016986312e+01, -2.59193146099879641e+01};
    double z[5] = {0.0, -1.03622044471123109e-01, -4.03523417114321381e-01, -2.23307578892655734e-01, 1.79258772950371181e-01};
    double vx[5] = {0.0, 1.66007664274403694e-03 * DAYS, -2.76742510726862411e-03 * DAYS, 2.96460137564761618e-03 * DAYS, 2.68067772490389322e-03 * DAYS};
    double vy[5] = {0.0, 7.69901118419740425e-03 * DAYS, 4.99852801234917238e-03 * DAYS, 2.37847173959480950e-03 * DAYS, 1.62824170038242295e-03 * DAYS};
    double vz[5] = {0.0, -6.90460016972063023e-05 * DAYS, 2.30417297573763929e-05 * DAYS, -2.96589568540237556e-05 * DAYS, -9.51592254519715870e-05 * DAYS};

    double dt = 0.01;
    
    // Warmup
    nbody_steps(n, m, x, y, z, vx, vy, vz, dt);
    
    // Timing
    double t0 = now_millis();
    for (int r = 0; r < 3; r++) {
        nbody_steps(n, m, x, y, z, vx, vy, vz, dt);
    }
    double t1 = now_millis();
    int best = (int)((t1 - t0) / 3.0);

    double energy = 0.0;
    for (int i = 0; i < n; i++) {
        energy = energy + 0.5 * m[i] * (vx[i] * vx[i] + vy[i] * vy[i] + vz[i] * vz[i]);
        for (int j = i + 1; j < n; j++) {
            double dx = x[i] - x[j];
            double dy = y[i] - y[j];
            double dz = z[i] - z[j];
            energy = energy - m[i] * m[j] / sqrt(dx * dx + dy * dy + dz * dz);
        }
    }
    printf("nbody %lld %d\n", (long long)(energy * 1000.0), best);
}

void matmul_kernel(double a[], double b[], double c[]) {
    for (int i = 0; i < MATMUL_N; i++) {
        int ib = i * MATMUL_N;
        for (int k = 0; k < MATMUL_N; k++) {
            double av = a[ib + k];
            int kb = k * MATMUL_N;
            for (int j = 0; j < MATMUL_N; j++) {
                c[ib + j] = c[ib + j] + av * b[kb + j];
            }
        }
    }
}

void matmul_run() {
    double *a = malloc(MATMUL_LEN * sizeof(double));
    double *b = malloc(MATMUL_LEN * sizeof(double));
    double *c = malloc(MATMUL_LEN * sizeof(double));
    
    for (int i = 0; i < MATMUL_LEN; i++) {
        a[i] = ((i % 1000) / 1000.0) + 0.5;
        b[i] = ((i % 997) / 997.0) + 0.25;
        c[i] = 0.0;
    }
    
    // Warmup
    matmul_kernel(a, b, c);
    
    // Timing
    double t0 = now_millis();
    for (int r = 0; r < 3; r++) {
        for (int i = 0; i < MATMUL_LEN; i++) {
            c[i] = 0.0;
        }
        matmul_kernel(a, b, c);
    }
    double t1 = now_millis();
    int best = (int)((t1 - t0) / 3.0);
    
    double sum = 0.0;
    for (int i = 0; i < MATMUL_LEN; i++) {
        sum = sum + c[i];
    }
    printf("matmul %lld %d\n", (long long)(sum * 1000.0), best);
    
    free(a);
    free(b);
    free(c);
}

void sieve_kernel(unsigned char comp[]) {
    int i = 2;
    while (i * i < SIEVE_N) {
        if (comp[i] == 0) {
            int j = i * i;
            while (j < SIEVE_N) {
                comp[j] = 1;
                j = j + i;
            }
        }
        i = i + 1;
    }
}

void sieve_run() {
    unsigned char *comp = malloc(SIEVE_N * sizeof(unsigned char));
    for (int i = 0; i < SIEVE_N; i++) {
        comp[i] = 0;
    }
    
    // Warmup
    sieve_kernel(comp);
    
    // Timing
    double t0 = now_millis();
    for (int r = 0; r < 3; r++) {
        for (int i = 0; i < SIEVE_N; i++) {
            comp[i] = 0;
        }
        sieve_kernel(comp);
    }
    double t1 = now_millis();
    int best = (int)((t1 - t0) / 3.0);
    
    int count = 0;
    for (int k = 2; k < SIEVE_N; k++) {
        if (comp[k] == 0) count = count + 1;
    }
    printf("sieve %d %d\n", count, best);
    
    free(comp);
}

void my_qsort(int a[], int lo, int hi) {
    if (lo >= hi) return;
    int p = a[(lo + hi) / 2];
    int i = lo;
    int j = hi;
    while (i <= j) {
        while (a[i] < p) i = i + 1;
        while (a[j] > p) j = j - 1;
        if (i <= j) {
            int t = a[i];
            a[i] = a[j];
            a[j] = t;
            i = i + 1;
            j = j - 1;
        }
    }
    if (lo < j) my_qsort(a, lo, j);
    if (i < hi) my_qsort(a, i, hi);
}

void qsort_fill(int a[]) {
    long long seed = 12345;
    for (int k = 0; k < QSORT_N; k++) {
        seed = (seed * 1103515245 + 12345) % 2147483648;
        a[k] = seed % 1000000;
    }
}

void qsort_run() {
    int *a = malloc(QSORT_N * sizeof(int));
    
    // Warmup
    qsort_fill(a);
    my_qsort(a, 0, QSORT_N - 1);
    
    // Timing
    double t0 = now_millis();
    for (int r = 0; r < 3; r++) {
        qsort_fill(a);
        my_qsort(a, 0, QSORT_N - 1);
    }
    double t1 = now_millis();
    int best = (int)((t1 - t0) / 3.0);
    
    long long sum = 0;
    for (int k = 0; k < QSORT_N; k++) {
        sum = sum + a[k] * (k % 97);
    }
    printf("qsort %lld %d\n", sum, best);
    
    free(a);
}

void mandelbrot_kernel() {
    mb_total = 0;
    for (int py = 0; py < MB_H; py++) {
        double y0 = (py * 2.0) / MB_H - 1.0;
        for (int px = 0; px < MB_W; px++) {
            double x0 = (px * 2.5) / MB_W - 2.0 - mb_seed * 0.001;
            double x = 0.0;
            double y = 0.0;
            int it = 0;
            while (it < MB_MAXIT) {
                double x2 = x * x;
                double y2 = y * y;
                if (x2 + y2 > 4.0) break;
                y = 2.0 * x * y + y0;
                x = x2 - y2 + x0;
                it = it + 1;
            }
            mb_total = mb_total + it;
        }
    }
}

void mandelbrot_run() {
    // Warmup
    mb_seed = 0;
    mandelbrot_kernel();
    
    // Timing
    double t0 = now_millis();
    for (int r = 0; r < 3; r++) {
        mb_seed = r;
        mandelbrot_kernel();
    }
    double t1 = now_millis();
    int best = (int)((t1 - t0) / 3.0);
    
    printf("mandelbrot %lld %d\n", mb_total, best);
}

int main() {
    nbody_run();
    matmul_run();
    sieve_run();
    qsort_run();
    mandelbrot_run();
    return 0;
}
