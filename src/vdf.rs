use std::ops::{Add, Mul, Sub};

// ============================================================================
// 1. STARK-Friendly Prime Field (Goldilocks Prime)
// p = 2^64 - 2^32 + 1 = 0xFFFFFFFF00000001
// ============================================================================
const P: u64 = 0xFFFFFFFF00000001;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fp(pub u64);

impl Fp {
    #[inline(always)]
    pub fn new(val: u64) -> Self {
        // Simple modulo reduction
        Fp(if val >= P { val % P } else { val })
    }
    
    #[inline(always)]
    pub fn zero() -> Self {
        Fp(0)
    }

    // Exponentiation for the S-Box
    pub fn pow(&self, exp: u64) -> Self {
        let mut res = Fp(1);
        let mut base = *self;
        let mut e = exp;
        while e > 0 {
            if e & 1 == 1 {
                res = res * base;
            }
            base = base * base;
            e >>= 1;
        }
        res
    }
}

impl Add for Fp {
    type Output = Self;
    #[inline(always)]
    fn add(self, rhs: Self) -> Self {
        let sum = (self.0 as u128) + (rhs.0 as u128);
        Fp(if sum >= P as u128 { (sum - P as u128) as u64 } else { sum as u64 })
    }
}

impl Sub for Fp {
    type Output = Self;
    #[inline(always)]
    fn sub(self, rhs: Self) -> Self {
        let diff = if self.0 >= rhs.0 {
            self.0 - rhs.0
        } else {
            P - (rhs.0 - self.0)
        };
        Fp(diff)
    }
}

impl Mul for Fp {
    type Output = Self;
    #[inline(always)]
    fn mul(self, rhs: Self) -> Self {
        let prod = (self.0 as u128) * (rhs.0 as u128);
        Fp((prod % (P as u128)) as u64)
    }
}

// ============================================================================
// 2. Octonion Algebra over F_p
// ============================================================================
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Octonion {
    pub coeffs: [Fp; 8],
}

impl Octonion {
    pub fn new(coeffs: [Fp; 8]) -> Self {
        Octonion { coeffs }
    }

    pub fn zero() -> Self {
        Octonion { coeffs: [Fp::zero(); 8] }
    }

    pub fn is_zero(&self) -> bool {
        self.coeffs.iter().all(|&x| x.0 == 0)
    }

    // Deterministic pseudo-random initialization mapping to F_p
    pub fn from_seed(seed: u64) -> Self {
        let mut coeffs = [Fp::zero(); 8];
        let mut current = seed;
        for i in 0..8 {
            current = current.wrapping_mul(6364136223846793005).wrapping_add(1);
            coeffs[i] = Fp::new(current);
        }
        Octonion::new(coeffs)
    }
}

impl Add for Octonion {
    type Output = Self;
    #[inline(always)]
    fn add(self, other: Self) -> Self {
        let mut c = [Fp::zero(); 8];
        for i in 0..8 { c[i] = self.coeffs[i] + other.coeffs[i]; }
        Octonion::new(c)
    }
}

impl Sub for Octonion {
    type Output = Self;
    #[inline(always)]
    fn sub(self, other: Self) -> Self {
        let mut c = [Fp::zero(); 8];
        for i in 0..8 { c[i] = self.coeffs[i] - other.coeffs[i]; }
        Octonion::new(c)
    }
}

// Full Non-Associative Fano Plane Multiplication over F_p
impl Mul for Octonion {
    type Output = Self;
    #[inline(always)]
    fn mul(self, other: Self) -> Self {
        let a = &self.coeffs;
        let b = &other.coeffs;
        let mut res = [Fp::zero(); 8];
        
        res[0] = a[0]*b[0] - a[1]*b[1] - a[2]*b[2] - a[3]*b[3] - a[4]*b[4] - a[5]*b[5] - a[6]*b[6] - a[7]*b[7];
        res[1] = a[0]*b[1] + a[1]*b[0] + a[2]*b[3] - a[3]*b[2] + a[4]*b[5] - a[5]*b[4] - a[6]*b[7] + a[7]*b[6];
        res[2] = a[0]*b[2] - a[1]*b[3] + a[2]*b[0] + a[3]*b[1] + a[4]*b[6] + a[5]*b[7] - a[6]*b[4] - a[7]*b[5];
        res[3] = a[0]*b[3] + a[1]*b[2] - a[2]*b[1] + a[3]*b[0] + a[4]*b[7] - a[5]*b[6] + a[6]*b[5] - a[7]*b[4];
        res[4] = a[0]*b[4] - a[1]*b[5] - a[2]*b[6] - a[3]*b[7] + a[4]*b[0] + a[5]*b[1] + a[6]*b[2] + a[7]*b[3];
        res[5] = a[0]*b[5] + a[1]*b[4] - a[2]*b[7] + a[3]*b[6] - a[4]*b[1] + a[5]*b[0] - a[6]*b[3] + a[7]*b[2];
        res[6] = a[0]*b[6] + a[1]*b[7] + a[2]*b[4] - a[3]*b[5] - a[4]*b[2] + a[5]*b[3] + a[6]*b[0] - a[7]*b[1];
        res[7] = a[0]*b[7] - a[1]*b[6] + a[2]*b[5] + a[3]*b[4] - a[4]*b[3] - a[5]*b[2] + a[6]*b[1] + a[7]*b[0];

        Octonion::new(res)
    }
}

// The Associator: [A, B, C] = (AB)C - A(BC)
pub fn associator(x: Octonion, y: Octonion, z: Octonion) -> Octonion {
    ((x * y) * z) - (x * (y * z))
}

// ============================================================================
// 3. Algebraic Hash Oracle (Poseidon-Lite Stand-in)
// Dynamically breaks Artin's Theorem by generating a strictly independent 
// 3rd element out of the current state, preventing associative trapping.
// ============================================================================
pub fn algebraic_hash_oracle(x: &Octonion) -> Octonion {
    let mut y = [Fp::zero(); 8];
    
    // Non-linear S-Box layer: x -> x^7 (7 is coprime to P-1, ensuring a true permutation)
    for i in 0..8 {
        y[i] = x.coeffs[i].pow(7);
    }
    
    // Linear Diffusion layer (MDS matrix mapping simulation)
    let mut sum = Fp::zero();
    for i in 0..8 { sum = sum + y[i]; }
    
    let mut res = [Fp::zero(); 8];
    for i in 0..8 {
        // Simple maximum diffusion: y'_i = y_i + sum(y) + round_constant
        res[i] = y[i] + sum + Fp::new((i as u64 + 1) * 0x1337CAFE_BEEFDEAD);
    }
    
    Octonion::new(res)
}

// ============================================================================
// 4. OctoSTARK VDF Evaluation
// ============================================================================
pub struct OctoStarkTrace {
    pub final_state: Octonion,
    pub trace: Vec<Octonion>, 
    // ^ The full execution trace to be passed to the STARK Prover (e.g., Plonky2)
}

pub fn evaluate_vdf(z_0: Octonion, c: Octonion, iterations: usize) -> OctoStarkTrace {
    let mut z = z_0;
    
    // Pre-allocate the trace vector to avoid reallocation overhead
    let mut trace = Vec::with_capacity(iterations + 1);
    trace.push(z);
    
    for _ in 0..iterations {
        // Z_{n+1} = Z_n^2 + C + [Z_n, C, H(Z_n)]
        let sq = z * z;
        let dynamic_generator = algebraic_hash_oracle(&z);
        let assoc = associator(z, c, dynamic_generator);
        
        z = sq + c + assoc;
        trace.push(z);
    }
    
    OctoStarkTrace {
        final_state: z,
        trace,
    }
}
