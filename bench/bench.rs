
use std::time::Instant;

const NBODY_STEPS: i64 = 2000000;
const DAYS: f64 = 365.24;
const MATMUL_N: usize = 1024;
const MATMUL_LEN: usize = 1048576;
const SIEVE_N: usize = 50000000;
const QSORT_N: usize = 2000000;
const MB_W: usize = 1024;
const MB_H: usize = 1024;
const MB_MAXIT: i64 = 300;

static mut MB_TOTAL: i64 = 0;
static mut MB_SEED: i64 = 0;

fn nbody_run() {
    let mut m = [0.0f64; 5];
    let mut x = [0.0f64; 5];
    let mut y = [0.0f64; 5];
    let mut z = [0.0f64; 5];
    let mut vx = [0.0f64; 5];
    let mut vy = [0.0f64; 5];
    let mut vz = [0.0f64; 5];

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

    let dt = 0.01;
    
    // Warmup
    nbody_steps(&mut m, &mut x, &mut y, &mut z, &mut vx, &mut vy, &mut vz, dt);
    
    let start = Instant::now();
    for _ in 0..3 {
        nbody_steps(&mut m, &mut x, &mut y, &mut z, &mut vx, &mut vy, &mut vz, dt);
    }
    let elapsed = start.elapsed().as_millis() as i64 / 3;
    
    let mut energy = 0.0;
    for i in 0..5 {
        energy += 0.5 * m[i] * (vx[i] * vx[i] + vy[i] * vy[i] + vz[i] * vz[i]);
        for j in (i + 1)..5 {
            let dx = x[i] - x[j];
            let dy = y[i] - y[j];
            let dz = z[i] - z[j];
            energy -= m[i] * m[j] / (dx * dx + dy * dy + dz * dz).sqrt();
        }
    }
    
    println!("nbody {} {}", (energy * 1000.0) as i64, elapsed);
}

fn nbody_steps(m: &mut [f64; 5], x: &mut [f64; 5], y: &mut [f64; 5], z: &mut [f64; 5], vx: &mut [f64; 5], vy: &mut [f64; 5], vz: &mut [f64; 5], dt: f64) {
    for _ in 0..NBODY_STEPS {
        for i in 0..5 {
            for j in (i + 1)..5 {
                let dx = x[i] - x[j];
                let dy = y[i] - y[j];
                let dz = z[i] - z[j];
                let d2 = dx * dx + dy * dy + dz * dz;
                let mag = dt / (d2 * d2.sqrt());
                vx[i] -= dx * m[j] * mag;
                vy[i] -= dy * m[j] * mag;
                vz[i] -= dz * m[j] * mag;
                vx[j] += dx * m[i] * mag;
                vy[j] += dy * m[i] * mag;
                vz[j] += dz * m[i] * mag;
            }
        }
        for i in 0..5 {
            x[i] += dt * vx[i];
            y[i] += dt * vy[i];
            z[i] += dt * vz[i];
        }
    }
}

fn matmul_run() {
    let mut a = vec![0.0f64; MATMUL_LEN];
    let mut b = vec![0.0f64; MATMUL_LEN];
    let mut c = vec![0.0f64; MATMUL_LEN];
    
    for i in 0..MATMUL_LEN {
        a[i] = ((i % 1000) as f64) / 1000.0 + 0.5;
        b[i] = ((i % 997) as f64) / 997.0 + 0.25;
        c[i] = 0.0;
    }
    
    // Warmup
    matmul_kernel(&a, &b, &mut c);
    
    let start = Instant::now();
    for _ in 0..3 {
        for i in 0..MATMUL_LEN {
            c[i] = 0.0;
        }
        matmul_kernel(&a, &b, &mut c);
    }
    let elapsed = start.elapsed().as_millis() as i64 / 3;
    
    let sum: f64 = c.iter().sum();
    println!("matmul {} {}", (sum * 1000.0) as i64, elapsed);
}

fn matmul_kernel(a: &[f64], b: &[f64], c: &mut [f64]) {
    for i in 0..MATMUL_N {
        let ib = i * MATMUL_N;
        for k in 0..MATMUL_N {
            let av = a[ib + k];
            let kb = k * MATMUL_N;
            for j in 0..MATMUL_N {
                c[ib + j] += av * b[kb + j];
            }
        }
    }
}

fn sieve_run() {
    let mut comp = vec![0u8; SIEVE_N];
    
    // Warmup
    sieve_kernel(&mut comp);
    
    let start = Instant::now();
    for _ in 0..3 {
        for i in 0..SIEVE_N {
            comp[i] = 0;
        }
        sieve_kernel(&mut comp);
    }
    let elapsed = start.elapsed().as_millis() as i64 / 3;
    
    let mut count = 0;
    for k in 2..SIEVE_N {
        if comp[k] == 0 { count += 1 }
    }
    println!("sieve {} {}", count, elapsed);
}

fn sieve_kernel(comp: &mut [u8]) {
    let mut i = 2;
    while i * i < SIEVE_N {
        if comp[i] == 0 {
            let mut j = i * i;
            while j < SIEVE_N {
                comp[j] = 1;
                j += i;
            }
        }
        i += 1;
    }
}

fn qsort_run() {
    let mut a = vec![0i64; QSORT_N];
    
    // Warmup
    qsort_fill(&mut a);
    qsort(&mut a, 0, (QSORT_N - 1) as i64);
    
    let start = Instant::now();
    for _ in 0..3 {
        qsort_fill(&mut a);
        qsort(&mut a, 0, (QSORT_N - 1) as i64);
    }
    let elapsed = start.elapsed().as_millis() as i64 / 3;
    
    let mut sum = 0i64;
    for k in 0..QSORT_N {
        sum += a[k] * (k % 97) as i64;
    }
    println!("qsort {} {}", sum, elapsed);
}

fn qsort(a: &mut [i64], lo: i64, hi: i64) {
    if lo >= hi { return }
    let p = a[((lo + hi) / 2) as usize];
    let mut i = lo;
    let mut j = hi;
    loop {
        while a[i as usize] < p { i += 1 }
        while a[j as usize] > p { j -= 1 }
        if i <= j {
            let t = a[i as usize];
            a[i as usize] = a[j as usize];
            a[j as usize] = t;
            i += 1;
            j -= 1;
        } else {
            break;
        }
    }
    if lo < j { qsort(a, lo, j); }
    if i < hi { qsort(a, i, hi); }
}

fn qsort_fill(a: &mut [i64]) {
    let mut seed = 12345i64;
    for k in 0..QSORT_N {
        seed = (seed * 1103515245 + 12345) % 2147483648;
        a[k] = seed % 1000000;
    }
}

fn mandelbrot_run() {
    // Warmup (seed=0)
    unsafe {
        MB_SEED = 0;
    }
    mandelbrot_kernel();
    
    let start = Instant::now();
    for r in 0..3 {
        unsafe {
            MB_SEED = r as i64;
        }
        mandelbrot_kernel();
    }
    let elapsed = start.elapsed().as_millis() as i64 / 3;
    
    let total = unsafe { MB_TOTAL };
    println!("mandelbrot {} {}", total, elapsed);
}

fn mandelbrot_kernel() {
    unsafe {
        MB_TOTAL = 0;
    }
    for py in 0..MB_H {
        let y0 = (py as f64) * 2.0 / (MB_H as f64) - 1.0;
        for px in 0..MB_W {
            let x0 = (px as f64) * 2.5 / (MB_W as f64) - 2.0 - (unsafe { MB_SEED } as f64) * 0.001;
            let mut x = 0.0f64;
            let mut y = 0.0f64;
            let mut it = 0i64;
            while it < MB_MAXIT {
                let x2 = x * x;
                let y2 = y * y;
                if x2 + y2 > 4.0 { break }
                y = 2.0 * x * y + y0;
                x = x2 - y2 + x0;
                it += 1;
            }
            unsafe {
                MB_TOTAL += it;
            }
        }
    }
}

fn main() {
    nbody_run();
    matmul_run();
    sieve_run();
    qsort_run();
    mandelbrot_run();
}
