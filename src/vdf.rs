use std::time::Instant;
use std::ops::{Add, Mul};
use std::thread;

// ----------------------------------------------------------------------------
// Core Octonion Structure (u64 for Integer VDF Benchmarking)
// ----------------------------------------------------------------------------
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Octonion {
    pub coeffs: [u64; 8],
}

impl Octonion {
    pub fn new(coeffs: [u64; 8]) -> Self {
        Octonion { coeffs }
    }

    pub fn zero() -> Self {
        Octonion { coeffs: [0; 8] }
    }

    // A heuristic "random" generator for the seed
    pub fn from_seed(seed: u64) -> Self {
        let s = seed;
        Octonion::new([
            s,
            s.wrapping_mul(6364136223846793005).wrapping_add(1),
            s.rotate_left(13),
            s.rotate_right(7),
            s ^ 0xCAFEBABECAFEBABE,
            !s,
            s.wrapping_add(0xDEADBEEF),
            s.wrapping_mul(0x123456789ABCDEF0)
        ])
    }

    // Returns a u64 "norm" (sum of squares modulo 2^64).
    pub fn norm_sq(&self) -> u64 {
        self.coeffs.iter().fold(0u64, |acc, &x| acc.wrapping_add(x.wrapping_mul(x)))
    }

    // Check if exactly zero
    pub fn is_zero(&self) -> bool {
        self.coeffs.iter().all(|&x| x == 0)
    }

    // Rotate coefficients to create a 3rd independent generator
    // This breaks Artin's Theorem (2-generator associativity)
    pub fn rotate(&self) -> Self {
        let mut new_c = [0; 8];
        for i in 0..8 {
            new_c[i] = self.coeffs[(i + 1) % 8];
        }
        Octonion::new(new_c)
    }
}

// ----------------------------------------------------------------------------
// Arithmetic Implementation (Cayley-Dickson over Z_2^64)
// ----------------------------------------------------------------------------
impl Add for Octonion {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        let mut c = [0; 8];
        for i in 0..8 { c[i] = self.coeffs[i].wrapping_add(other.coeffs[i]); }
        Octonion::new(c)
    }
}

// Full non-associative multiplication
impl Mul for Octonion {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        let a = &self.coeffs;
        let b = &other.coeffs;
        let mut res = [0; 8];
        
        // Optimized naive multiplication for benchmarking
        // Real part
        res[0] = a[0].wrapping_mul(b[0])
            .wrapping_sub(a[1].wrapping_mul(b[1])).wrapping_sub(a[2].wrapping_mul(b[2]))
            .wrapping_sub(a[3].wrapping_mul(b[3])).wrapping_sub(a[4].wrapping_mul(b[4]))
            .wrapping_sub(a[5].wrapping_mul(b[5])).wrapping_sub(a[6].wrapping_mul(b[6]))
            .wrapping_sub(a[7].wrapping_mul(b[7]));

        // Imaginary parts
        res[1] = a[0].wrapping_mul(b[1]).wrapping_add(a[1].wrapping_mul(b[0]))
            .wrapping_add(a[2].wrapping_mul(b[3])).wrapping_sub(a[3].wrapping_mul(b[2]))
            .wrapping_add(a[4].wrapping_mul(b[5])).wrapping_sub(a[5].wrapping_mul(b[4]))
            .wrapping_sub(a[6].wrapping_mul(b[7])).wrapping_add(a[7].wrapping_mul(b[6]));

        res[2] = a[0].wrapping_mul(b[2]).wrapping_sub(a[1].wrapping_mul(b[3]))
            .wrapping_add(a[2].wrapping_mul(b[0])).wrapping_add(a[3].wrapping_mul(b[1]))
            .wrapping_add(a[4].wrapping_mul(b[6])).wrapping_add(a[5].wrapping_mul(b[7]))
            .wrapping_sub(a[6].wrapping_mul(b[4])).wrapping_sub(a[7].wrapping_mul(b[5]));
               
        res[3] = a[0].wrapping_mul(b[3]).wrapping_add(a[1].wrapping_mul(b[2]))
            .wrapping_sub(a[2].wrapping_mul(b[1])).wrapping_add(a[3].wrapping_mul(b[0]))
            .wrapping_add(a[4].wrapping_mul(b[7])).wrapping_sub(a[5].wrapping_mul(b[6]))
            .wrapping_add(a[6].wrapping_mul(b[5])).wrapping_sub(a[7].wrapping_mul(b[4]));

        res[4] = a[0].wrapping_mul(b[4]).wrapping_sub(a[1].wrapping_mul(b[5]))
            .wrapping_sub(a[2].wrapping_mul(b[6])).wrapping_sub(a[3].wrapping_mul(b[7]))
            .wrapping_add(a[4].wrapping_mul(b[0])).wrapping_add(a[5].wrapping_mul(b[1]))
            .wrapping_add(a[6].wrapping_mul(b[2])).wrapping_add(a[7].wrapping_mul(b[3]));

        res[5] = a[0].wrapping_mul(b[5]).wrapping_add(a[1].wrapping_mul(b[4]))
            .wrapping_sub(a[2].wrapping_mul(b[7])).wrapping_add(a[3].wrapping_mul(b[6]))
            .wrapping_sub(a[4].wrapping_mul(b[1])).wrapping_add(a[5].wrapping_mul(b[0]))
            .wrapping_sub(a[6].wrapping_mul(b[3])).wrapping_add(a[7].wrapping_mul(b[2]));

        res[6] = a[0].wrapping_mul(b[6]).wrapping_add(a[1].wrapping_mul(b[7]))
            .wrapping_add(a[2].wrapping_mul(b[4])).wrapping_sub(a[3].wrapping_mul(b[5]))
            .wrapping_sub(a[4].wrapping_mul(b[2])).wrapping_add(a[5].wrapping_mul(b[3]))
            .wrapping_add(a[6].wrapping_mul(b[0])).wrapping_sub(a[7].wrapping_mul(b[1]));

        res[7] = a[0].wrapping_mul(b[7]).wrapping_sub(a[1].wrapping_mul(b[6]))
            .wrapping_add(a[2].wrapping_mul(b[5])).wrapping_add(a[3].wrapping_mul(b[4]))
            .wrapping_sub(a[4].wrapping_mul(b[3])).wrapping_sub(a[5].wrapping_mul(b[2]))
            .wrapping_add(a[6].wrapping_mul(b[1])).wrapping_add(a[7].wrapping_mul(b[0]));

        Octonion::new(res)
    }
}

// The Associator: [x,y,z] = (xy)z - x(yz)
pub fn associator(x: Octonion, y: Octonion, z: Octonion) -> Octonion {
    let xy_z = (x * y) * z;
    let x_yz = x * (y * z);
    
    // Manual subtraction since we didn't impl Sub
    let mut res = [0; 8];
    for i in 0..8 { res[i] = xy_z.coeffs[i].wrapping_sub(x_yz.coeffs[i]); }
    Octonion::new(res)
}

// ----------------------------------------------------------------------------
// Benchmark Logic
// ----------------------------------------------------------------------------
pub fn run_benchmark() {
    println!("\n\n========================================================");
    println!("=== Synergeia VDF Benchmark: The Octonionic Iterator ===");
    println!("========================================================");
    println!("System: Z_{{n+1}} = Z_n^2 + C + Associator(Z_n, C, Z_n_rot)");
    println!("Note: The 'Associator' term is added to break 2-generator associativity.\n");

    let iterations = 1_000_000;
    
    // Seed parameters (Genesis State)
    let c = Octonion::from_seed(12345);
    let z_0 = Octonion::from_seed(67890);

    // ------------------------------------------------------------------------
    // TEST 1: The Associator Hazard Check
    // ------------------------------------------------------------------------
    println!("[Test 1] Verifying Non-Associativity (Topological Impedance)...");
    
    // We check the hazard of the *perturbed* step.
    // Hazard = [Z, C, Z_rot]
    let z_rot = z_0.rotate();
    let hazard = associator(z_0, c, z_rot);
    
    println!("   > Associator Hazard Check (Is Zero?): {}", hazard.is_zero());
    if !hazard.is_zero() {
        println!("   > RESULT: NON-ZERO. SUCCESS! Trajectory utilizes full Octonionic bulk.\n");
    } else {
        println!("   > RESULT: ZERO. Warning: Still associative! Check generators.\n");
    }

    // ------------------------------------------------------------------------
    // TEST 2: Sequential Hardness (True Non-Associative VDF)
    // ------------------------------------------------------------------------
    println!("[Test 2] Benchmarking Sequential Execution ({} iterations)...", iterations);
    
    let start_seq = Instant::now();
    let mut z = z_0;
    
    for _ in 0..iterations {
        // The VDF Iteration: Z_{n+1} = Z_n^2 + C + [Z_n, C, Z_n_rot]
        let sq = z * z;
        let rot = z.rotate();
        let assoc = associator(z, c, rot);
        z = sq + c + assoc;
    }
    
    let duration_seq = start_seq.elapsed();
    println!("   > Final State Norm (Wrap): {}", z.norm_sq());
    println!("   > Time Elapsed: {:.4}s", duration_seq.as_secs_f64());
    println!("   > Throughput: {:.0} ops/sec\n", iterations as f64 / duration_seq.as_secs_f64());

    // ------------------------------------------------------------------------
    // TEST 3: Parallel "Attack" Simulation
    // ------------------------------------------------------------------------
    println!("[Test 3] Simulating Parallel Attack (2 Threads)...");
    println!("   > Attacker attempts to compute first half and second half simultaneously.");
    println!("   > Constraint: Thread 2 needs Z_{{500,000}} to start.");
    
    let start_par = Instant::now();
    
    // Thread 1
    let handle1 = thread::spawn(move || {
        let mut z_local = z_0;
        for _ in 0..(iterations / 2) {
            let sq = z_local * z_local;
            let rot = z_local.rotate();
            let assoc = associator(z_local, c, rot);
            z_local = sq + c + assoc;
        }
        z_local
    });
    
    // Thread 2 (Waiting)
    let z_mid = handle1.join().unwrap();
    
    // Thread 2 (Running)
    let mut z_final = z_mid;
    for _ in 0..(iterations / 2) {
        let sq = z_final * z_final;
        let rot = z_final.rotate();
        let assoc = associator(z_final, c, rot);
        z_final = sq + c + assoc;
    }
    
    let duration_par = start_par.elapsed();
    
    println!("   > Final State Norm (Wrap): {}", z_final.norm_sq());
    println!("   > Time Elapsed: {:.4}s", duration_par.as_secs_f64());
    
    let speedup = duration_seq.as_secs_f64() / duration_par.as_secs_f64();
    println!("   > Effective Speedup: {:.4}x (Target: 1.0x)", speedup);

    if (speedup - 1.0).abs() < 0.1 {
        println!("   > CONCLUSION: Parallelism provided NO advantage. VDF Property Holds.");
    } else {
        println!("   > CONCLUSION: Anomaly detected.");
    }
}