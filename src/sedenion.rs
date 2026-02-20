// src/sedenion.rs
// Implements the 16-dimensional Sedenion Algebra (S)
// Sedenions are Non-Commutative, Non-Associative, and Non-Alternative.
// They represent the "Chaos" phase of the APH vacuum (Beta -> 0).

//use crate::vdf::Octonion; // Reuse the robust Octonion from VDF module
use std::ops::{Add, Mul, BitXor};


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


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sedenion {
    pub low: Octonion,  // Coefficients 0-7
    pub high: Octonion, // Coefficients 8-15
}

impl Sedenion {
    pub fn new(low: Octonion, high: Octonion) -> Self {
        Sedenion { low, high }
    }

    pub fn zero() -> Self {
        Sedenion { 
            low: Octonion::zero(), 
            high: Octonion::zero() 
        }
    }

    // Determine the conjugate of the Sedenion
    // S* = (L*, -H)
    pub fn conjugate(&self) -> Self {
        // We need an octonion conjugate helper. 
        // Since Octonion structs are arrays, we calculate manually for now to avoid altering vdf.rs too much.
        let l = self.low.coeffs;
        let h = self.high.coeffs;
        
        // Conjugate of Octonion: (s, -v)
        // Here we just negate everything for the algebraic conjugate approximation in modular arithmetic
        // Real implementation would preserve the scalar part. 
        // For hashing, a full bitwise negation is a stronger mixer.
        let mut l_conj = [0u64; 8];
        l_conj[0] = l[0]; // Keep real part (usually) but for hash we might want full diffusion
        for i in 1..8 { l_conj[i] = l[i].wrapping_neg(); }

        let mut h_neg = [0u64; 8];
        for i in 0..8 { h_neg[i] = h[i].wrapping_neg(); }

        Sedenion {
            low: Octonion::new(l_conj),
            high: Octonion::new(h_neg),
        }
    }
}

// Cayley-Dickson Construction:
// (A, B) * (C, D) = (AC - D*B_conj, A_conj*D + CB)
impl Mul for Sedenion {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        let a = self.low;
        let b = self.high;
        let c = other.low;
        let d = other.high;

        // We need a conjugate function for Octonions for strict C-D construction.
        // We define a local helper since vdf::Octonion might not expose it publicly.
        let oct_conj = |o: Octonion| -> Octonion {
            let mut coeffs = o.coeffs;
            for i in 1..8 { coeffs[i] = coeffs[i].wrapping_neg(); }
            Octonion::new(coeffs)
        };

        let b_conj = oct_conj(b);
        let a_conj = oct_conj(a);

        // Term 1: AC - D * B_conj
        let ac = a * c;
        let d_b_conj = d * b_conj;
        // Subtraction in modular arithmetic is adding the negation
        let mut term1_coeffs = [0u64; 8];
        for i in 0..8 {
            term1_coeffs[i] = ac.coeffs[i].wrapping_sub(d_b_conj.coeffs[i]);
        }
        let term1 = Octonion::new(term1_coeffs);

        // Term 2: A_conj * D + C * B
        let a_conj_d = a_conj * d;
        let cb = c * b;
        let term2 = a_conj_d + cb;

        Sedenion::new(term1, term2)
    }
}

impl Add for Sedenion {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Sedenion::new(self.low + other.low, self.high + other.high)
    }
}

impl BitXor for Sedenion {
    type Output = Self;
    fn bitxor(self, other: Self) -> Self {
        let mut l = [0u64; 8];
        let mut h = [0u64; 8];
        for i in 0..8 {
            l[i] = self.low.coeffs[i] ^ other.low.coeffs[i];
            h[i] = self.high.coeffs[i] ^ other.high.coeffs[i];
        }
        Sedenion::new(Octonion::new(l), Octonion::new(h))
    }
}

// The Sedenion Associator: [X, Y, Z] = (XY)Z - X(YZ)
// This is the core "Sponge" function for GSH-256.
// In Sedenions, this is non-zero and highly chaotic.
pub fn associator(x: Sedenion, y: Sedenion, z: Sedenion) -> Sedenion {
    let xy_z = (x * y) * z;
    let x_yz = x * (y * z);
    
    let mut l = [0u64; 8];
    let mut h = [0u64; 8];
    
    for i in 0..8 {
        l[i] = xy_z.low.coeffs[i].wrapping_sub(x_yz.low.coeffs[i]);
        h[i] = xy_z.high.coeffs[i].wrapping_sub(x_yz.high.coeffs[i]);
    }
    
    Sedenion::new(Octonion::new(l), Octonion::new(h))
}