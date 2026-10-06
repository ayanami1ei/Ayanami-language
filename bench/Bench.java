
public class Bench {
    // ── nbody（5 体引力，经典 Benchmarks Game 常量）──
    static final int NBODY_STEPS = 2000000;
    static final double DAYS = 365.24;

    static double[] m = new double[5];
    static double[] x = new double[5];
    static double[] y = new double[5];
    static double[] z = new double[5];
    static double[] vx = new double[5];
    static double[] vy = new double[5];
    static double[] vz = new double[5];

    static void nbody_run() {
        m[0] = 39.478;
        x[0] = 0.0;
        y[0] = 0.0;
        z[0] = 0.0;
        vx[0] = 0.0;
        vy[0] = 0.0;
        vz[0] = 0.0;
        m[1] = 0.03769367487038949;
        x[1] = 4.84143144246472090e+00;
        y[1] = -1.16032004402742839e+00;
        z[1] = -1.03622044471123109e-01;
        vx[1] = 1.66007664274403694e-03 * DAYS;
        vy[1] = 7.69901118419740425e-03 * DAYS;
        vz[1] = -6.90460016972063023e-05 * DAYS;
        m[2] = 0.011286326131968767;
        x[2] = 8.34336671824457987e+00;
        y[2] = 4.12479856412430479e+00;
        z[2] = -4.03523417114321381e-01;
        vx[2] = -2.76742510726862411e-03 * DAYS;
        vy[2] = 4.99852801234917238e-03 * DAYS;
        vz[2] = 2.30417297573763929e-05 * DAYS;
        m[3] = 0.0017237240570597112;
        x[3] = 1.28943695621391310e+01;
        y[3] = -1.51111514016986312e+01;
        z[3] = -2.23307578892655734e-01;
        vx[3] = 2.96460137564761618e-03 * DAYS;
        vy[3] = 2.37847173959480950e-03 * DAYS;
        vz[3] = -2.96589568540237556e-05 * DAYS;
        m[4] = 0.0020336868699246304;
        x[4] = 1.53796971148509165e+01;
        y[4] = -2.59193146099879641e+01;
        z[4] = 1.79258772950371181e-01;
        vx[4] = 2.68067772490389322e-03 * DAYS;
        vy[4] = 1.62824170038242295e-03 * DAYS;
        vz[4] = -9.51592254519715870e-05 * DAYS;

        double dt = 0.01;
        // 预热 1 次 + 计时 3 次
        nbody_steps();
        long t0 = System.nanoTime();
        for (int r = 0; r < 3; r++) {
            nbody_steps();
        }
        long t1 = System.nanoTime();
        long best = (t1 - t0) / 3_000_000;

        double energy = 0.0;
        for (int i = 0; i < 5; i++) {
            energy = energy + 0.5 * m[i] * (vx[i] * vx[i] + vy[i] * vy[i] + vz[i] * vz[i]);
            for (int j = i + 1; j < 5; j++) {
                double dx = x[i] - x[j];
                double dy = y[i] - y[j];
                double dz = z[i] - z[j];
                energy = energy - m[i] * m[j] / Math.sqrt(dx * dx + dy * dy + dz * dz);
            }
        }
        System.out.println("nbody " + (long)(energy * 1000.0) + " " + best);
    }

    static void nbody_steps() {
        for (int s = 0; s < NBODY_STEPS; s++) {
            for (int i = 0; i < 5; i++) {
                for (int j = i + 1; j < 5; j++) {
                    double dx = x[i] - x[j];
                    double dy = y[i] - y[j];
                    double dz = z[i] - z[j];
                    double d2 = dx * dx + dy * dy + dz * dz;
                    double mag = 0.01 / (d2 * Math.sqrt(d2));
                    vx[i] = vx[i] - dx * m[j] * mag;
                    vy[i] = vy[i] - dy * m[j] * mag;
                    vz[i] = vz[i] - dz * m[j] * mag;
                    vx[j] = vx[j] + dx * m[i] * mag;
                    vy[j] = vy[j] + dy * m[i] * mag;
                    vz[j] = vz[j] + dz * m[i] * mag;
                }
            }
            for (int i = 0; i < 5; i++) {
                x[i] = x[i] + 0.01 * vx[i];
                y[i] = y[i] + 0.01 * vy[i];
                z[i] = z[i] + 0.01 * vz[i];
            }
        }
    }

    // ── matmul（1024×1024 f64，ikj 顺序）──
    static final int MATMUL_N = 1024;
    static final int MATMUL_LEN = 1048576;

    static double[] a = new double[MATMUL_LEN];
    static double[] b = new double[MATMUL_LEN];
    static double[] c = new double[MATMUL_LEN];

    static void matmul_run() {
        for (int i = 0; i < MATMUL_LEN; i++) {
            a[i] = ((i % 1000) / 1000.0) + 0.5;
            b[i] = ((i % 997) / 997.0) + 0.25;
            c[i] = 0.0;
        }
        // 预热 1 次
        matmul_kernel();
        long t0 = System.nanoTime();
        for (int r = 0; r < 3; r++) {
            for (int i = 0; i < MATMUL_LEN; i++) {
                c[i] = 0.0;
            }
            matmul_kernel();
        }
        long t1 = System.nanoTime();
        long best = (t1 - t0) / 3_000_000;
        double sum = 0.0;
        for (int i = 0; i < MATMUL_LEN; i++) {
            sum = sum + c[i];
        }
        System.out.println("matmul " + (long)(sum * 1000.0) + " " + best);
    }

    static void matmul_kernel() {
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

    // ── sieve（埃拉托斯特尼筛，u8 标记）──
    static final int SIEVE_N = 50000000;

    static byte[] comp = new byte[SIEVE_N];

    static void sieve_run() {
        for (int i = 0; i < SIEVE_N; i++) {
            comp[i] = 0;
        }
        // 预热 1 次
        sieve_kernel();
        long t0 = System.nanoTime();
        for (int r = 0; r < 3; r++) {
            for (int i = 0; i < SIEVE_N; i++) {
                comp[i] = 0;
            }
            sieve_kernel();
        }
        long t1 = System.nanoTime();
        long best = (t1 - t0) / 3_000_000;
        int count = 0;
        for (int k = 2; k < SIEVE_N; k++) {
            if (comp[k] == 0) count = count + 1;
        }
        System.out.println("sieve " + count + " " + best);
    }

    static void sieve_kernel() {
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

    // ── quicksort（2M int，LCG 数据）──
    static final int QSORT_N = 2000000;

    static int[] qarr = new int[QSORT_N];

    static void qsort_run() {
        // 预热 1 次
        qsort_fill();
        qsort(0, QSORT_N - 1);
        long t0 = System.nanoTime();
        for (int r = 0; r < 3; r++) {
            qsort_fill();
            qsort(0, QSORT_N - 1);
        }
        long t1 = System.nanoTime();
        long best = (t1 - t0) / 3_000_000;
        long sum = 0;
        for (int k = 0; k < QSORT_N; k++) {
            sum = sum + qarr[k] * (k % 97);
        }
        System.out.println("qsort " + sum + " " + best);
    }

    static void qsort(int lo, int hi) {
        if (lo >= hi) return;
        int p = qarr[(lo + hi) / 2];
        int i = lo;
        int j = hi;
        while (i <= j) {
            while (qarr[i] < p) i = i + 1;
            while (qarr[j] > p) j = j - 1;
            if (i <= j) {
                int t = qarr[i];
                qarr[i] = qarr[j];
                qarr[j] = t;
                i = i + 1;
                j = j - 1;
            }
        }
        if (lo < j) qsort(lo, j);
        if (i < hi) qsort(i, hi);
    }

    static void qsort_fill() {
        long seed = 12345;
        for (int k = 0; k < QSORT_N; k++) {
            seed = (seed * 1103515245L + 12345) % 2147483648L;
            qarr[k] = (int)(seed % 1000000);
        }
    }

    // ── mandelbrot（1024×1024，max 300 iter）──
    static final int MB_W = 1024;
    static final int MB_H = 1024;
    static final int MB_MAXIT = 300;

    static long MB_TOTAL = 0;
    static long MB_SEED = 0;

    static void mandelbrot_run() {
        // 预热 1 次（seed=0）
        MB_SEED = 0;
        mandelbrot_kernel();
        long t0 = System.nanoTime();
        for (int r = 0; r < 3; r++) {
            MB_SEED = r;
            mandelbrot_kernel();
        }
        long t1 = System.nanoTime();
        long best = (t1 - t0) / 3_000_000;
        System.out.println("mandelbrot " + MB_TOTAL + " " + best);
    }

    static void mandelbrot_kernel() {
        MB_TOTAL = 0;
        for (int py = 0; py < MB_H; py++) {
            double y0 = (py * 2.0) / MB_H - 1.0;
            for (int px = 0; px < MB_W; px++) {
                double x0 = (px * 2.5) / MB_W - 2.0 - MB_SEED * 0.001;
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
                MB_TOTAL = MB_TOTAL + it;
            }
        }
    }

    public static void main(String[] args) {
        nbody_run();
        matmul_run();
        sieve_run();
        qsort_run();
        mandelbrot_run();
    }
}
