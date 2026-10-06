
const std = @import("std");
const c_import = @cImport({
    @cInclude("stdio.h");
    @cInclude("stdlib.h");
    @cInclude("time.h");
});

const NBODY_STEPS = 2000000;
const DAYS = 365.24;

const MATMUL_N = 1024;
const MATMUL_LEN = 1048576;

const SIEVE_N = 50000000;

const QSORT_N = 2000000;

const MB_W = 1024;
const MB_H = 1024;
const MB_MAXIT = 300;

var mb_total: i64 = 0;
var mb_seed: i64 = 0;

fn now_millis() i64 {
    var ts: c_import.struct_timespec = undefined;
    _ = c_import.clock_gettime(c_import.CLOCK_MONOTONIC, &ts);
    return @as(i64, ts.tv_sec) * 1000 + @divTrunc(ts.tv_nsec, 1_000_000);
}

fn nbody_steps(n: usize, m: []f64, x: []f64, y: []f64, z: []f64, vx: []f64, vy: []f64, vz: []f64, dt: f64) void {
    var s: usize = 0;
    while (s < NBODY_STEPS) : (s += 1) {
        var i: usize = 0;
        while (i < n) : (i += 1) {
            var j: usize = i + 1;
            while (j < n) : (j += 1) {
                const dx = x[i] - x[j];
                const dy = y[i] - y[j];
                const dz = z[i] - z[j];
                const d2 = dx * dx + dy * dy + dz * dz;
                const mag = dt / (d2 * @sqrt(d2));
                vx[i] = vx[i] - dx * m[j] * mag;
                vy[i] = vy[i] - dy * m[j] * mag;
                vz[i] = vz[i] - dz * m[j] * mag;
                vx[j] = vx[j] + dx * m[i] * mag;
                vy[j] = vy[j] + dy * m[i] * mag;
                vz[j] = vz[j] + dz * m[i] * mag;
            }
        }
        var p: usize = 0;
        while (p < n) : (p += 1) {
            x[p] = x[p] + dt * vx[p];
            y[p] = y[p] + dt * vy[p];
            z[p] = z[p] + dt * vz[p];
        }
    }
}

fn nbody_run() void {
    var m = std.heap.page_allocator.alloc(f64, 5) catch unreachable;
    defer std.heap.page_allocator.free(m);
    var x = std.heap.page_allocator.alloc(f64, 5) catch unreachable;
    defer std.heap.page_allocator.free(x);
    var y = std.heap.page_allocator.alloc(f64, 5) catch unreachable;
    defer std.heap.page_allocator.free(y);
    var z = std.heap.page_allocator.alloc(f64, 5) catch unreachable;
    defer std.heap.page_allocator.free(z);
    var vx = std.heap.page_allocator.alloc(f64, 5) catch unreachable;
    defer std.heap.page_allocator.free(vx);
    var vy = std.heap.page_allocator.alloc(f64, 5) catch unreachable;
    defer std.heap.page_allocator.free(vy);
    var vz = std.heap.page_allocator.alloc(f64, 5) catch unreachable;
    defer std.heap.page_allocator.free(vz);

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

    const dt = 0.01;
    // 预热 1 次 + 计时 3 次
    nbody_steps(5, m, x, y, z, vx, vy, vz, dt);
    const t0 = now_millis();
    var r: usize = 0;
    while (r < 3) : (r += 1) {
        nbody_steps(5, m, x, y, z, vx, vy, vz, dt);
    }
    const t1 = now_millis();
    const best = @divTrunc(t1 - t0, 3);

    var energy: f64 = 0.0;
    var i: usize = 0;
    while (i < 5) : (i += 1) {
        energy = energy + 0.5 * m[i] * (vx[i] * vx[i] + vy[i] * vy[i] + vz[i] * vz[i]);
        var j: usize = i + 1;
        while (j < 5) : (j += 1) {
            const dx = x[i] - x[j];
            const dy = y[i] - y[j];
            const dz = z[i] - z[j];
            energy = energy - m[i] * m[j] / @sqrt(dx * dx + dy * dy + dz * dz);
        }
    }
    const checksum: i64 = @intFromFloat(energy * 1000.0);
    _ = c_import.printf("nbody %lld %lld\n", checksum, best);
}

fn matmul_kernel(a: []f64, b: []f64, mat_c: []f64) void {
    var i: usize = 0;
    while (i < MATMUL_N) : (i += 1) {
        const ib = i * MATMUL_N;
        var k: usize = 0;
        while (k < MATMUL_N) : (k += 1) {
            const av = a[ib + k];
            const kb = k * MATMUL_N;
            var j: usize = 0;
            while (j < MATMUL_N) : (j += 1) {
                mat_c[ib + j] = mat_c[ib + j] + av * b[kb + j];
            }
        }
    }
}

fn matmul_run() void {
    var a = std.heap.page_allocator.alloc(f64, MATMUL_LEN) catch unreachable;
    defer std.heap.page_allocator.free(a);
    var b = std.heap.page_allocator.alloc(f64, MATMUL_LEN) catch unreachable;
    defer std.heap.page_allocator.free(b);
    var mc = std.heap.page_allocator.alloc(f64, MATMUL_LEN) catch unreachable;
    defer std.heap.page_allocator.free(mc);

    var i: usize = 0;
    while (i < MATMUL_LEN) : (i += 1) {
        a[i] = (@as(f64, @floatFromInt(i % 1000)) / 1000.0) + 0.5;
        b[i] = (@as(f64, @floatFromInt(i % 997)) / 997.0) + 0.25;
        mc[i] = 0.0;
    }
    // 预热 1 次
    matmul_kernel(a, b, mc);
    const t0 = now_millis();
    var r: usize = 0;
    while (r < 3) : (r += 1) {
        i = 0;
        while (i < MATMUL_LEN) : (i += 1) {
            mc[i] = 0.0;
        }
        matmul_kernel(a, b, mc);
    }
    const t1 = now_millis();
    const best = @divTrunc(t1 - t0, 3);
    var sum: f64 = 0.0;
    i = 0;
    while (i < MATMUL_LEN) : (i += 1) {
        sum = sum + mc[i];
    }
    const checksum: i64 = @intFromFloat(sum * 1000.0);
    _ = c_import.printf("matmul %lld %lld\n", checksum, best);
}

fn sieve_kernel(comp: []u8) void {
    var i: usize = 2;
    while (i * i < SIEVE_N) : (i += 1) {
        if (comp[i] == 0) {
            var j: usize = i * i;
            while (j < SIEVE_N) : (j += i) {
                comp[j] = 1;
            }
        }
    }
}

fn sieve_run() void {
    var comp = std.heap.page_allocator.alloc(u8, SIEVE_N) catch unreachable;
    defer std.heap.page_allocator.free(comp);

    var i: usize = 0;
    while (i < SIEVE_N) : (i += 1) {
        comp[i] = 0;
    }
    // 预热 1 次
    sieve_kernel(comp);
    const t0 = now_millis();
    var r: usize = 0;
    while (r < 3) : (r += 1) {
        i = 0;
        while (i < SIEVE_N) : (i += 1) {
            comp[i] = 0;
        }
        sieve_kernel(comp);
    }
    const t1 = now_millis();
    const best = @divTrunc(t1 - t0, 3);
    var count: i32 = 0;
    i = 2;
    while (i < SIEVE_N) : (i += 1) {
        if (comp[i] == 0) {
            count += 1;
        }
    }
    _ = c_import.printf("sieve %lld %lld\n", @as(i64, count), best);
}

fn qsort(a: []i32, lo: usize, hi: usize) void {
    if (lo >= hi) return;
    const p = a[(lo + hi) / 2];
    var i = lo;
    var j = hi;
    while (i <= j) {
        while (a[i] < p) i += 1;
        while (a[j] > p) j -= 1;
        if (i <= j) {
            const t = a[i];
            a[i] = a[j];
            a[j] = t;
            i += 1;
            j -= 1;
        }
    }
    if (lo < j) qsort(a, lo, j);
    if (i < hi) qsort(a, i, hi);
}

fn qsort_fill(a: []i32) void {
    var seed: u32 = 12345;
    var k: usize = 0;
    while (k < QSORT_N) : (k += 1) {
        seed = (seed * 1103515245 + 12345) % 2147483648;
        a[k] = @as(i32, @intCast(seed % 1000000));
    }
}

fn qsort_run() void {
    const a = std.heap.page_allocator.alloc(i32, QSORT_N) catch unreachable;
    defer std.heap.page_allocator.free(a);
    // 预热 1 次
    qsort_fill(a);
    qsort(a, 0, QSORT_N - 1);
    const t0 = now_millis();
    var r: usize = 0;
    while (r < 3) : (r += 1) {
        qsort_fill(a);
        qsort(a, 0, QSORT_N - 1);
    }
    const t1 = now_millis();
    const best = @divTrunc(t1 - t0, 3);
    var sum: i64 = 0;
    var k: usize = 0;
    while (k < QSORT_N) : (k += 1) {
        sum += @as(i64, a[k]) * @as(i64, @intCast(k % 97));
    }
    _ = c_import.printf("qsort %lld %lld\n", sum, best);
}

fn mandelbrot_kernel() void {
    mb_total = 0;
    var py: usize = 0;
    while (py < MB_H) : (py += 1) {
        const y0 = (@as(f64, @floatFromInt(py)) * 2.0) / (@as(f64, @floatFromInt(MB_H))) - 1.0;
        var px: usize = 0;
        while (px < MB_W) : (px += 1) {
            const x0 = (@as(f64, @floatFromInt(px)) * 2.5) / (@as(f64, @floatFromInt(MB_W))) - 2.0 - (@as(f64, @floatFromInt(mb_seed)) * 0.001);
            var x: f64 = 0.0;
            var y: f64 = 0.0;
            var it: i32 = 0;
            while (it < MB_MAXIT) : (it += 1) {
                const x2 = x * x;
                const y2 = y * y;
                if (x2 + y2 > 4.0) break;
                y = 2.0 * x * y + y0;
                x = x2 - y2 + x0;
            }
            mb_total += it;
        }
    }
}

fn mandelbrot_run() void {
    // 预热 1 次（seed=0）
    mb_seed = 0;
    mandelbrot_kernel();
    const t0 = now_millis();
    var r: usize = 0;
    while (r < 3) : (r += 1) {
        mb_seed = @intCast(r);
        mandelbrot_kernel();
    }
    const t1 = now_millis();
    const best = @divTrunc(t1 - t0, 3);
    _ = c_import.printf("mandelbrot %lld %lld\n", mb_total, best);
}

pub fn main() !void {
    nbody_run();
    matmul_run();
    sieve_run();
    qsort_run();
    mandelbrot_run();
}
