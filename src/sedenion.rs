// src/sedenion.rs
// Implements the 16-dimensional Sedenion Algebra (S)
// Sedenions are Non-Commutative, Non-Associative, and Non-Alternative.
// They represent the "Chaos" phase of the APH vacuum (Beta -> 0).

use crate::vdf::Octonion; // Reuse the robust Octonion from VDF module
use std::ops::{Add, Mul, BitXor};

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