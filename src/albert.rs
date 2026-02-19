// src/albert.rs
use rand::prelude::*;
use rand_distr::{Distribution, Weibull};
use std::ops::{Add, Sub, Mul};

// --- CONFIGURATION ---
// Modulus for the Lattice Cryptography (2^15)
pub const Q: u64 = 32768; 
pub type Scalar = u64;

// --- 8-DIM OCTONION ---
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Octonion {
    pub c: [Scalar; 8],
}

impl Octonion {
    pub fn new(c: [Scalar; 8]) -> Self {
        Octonion { c }
    }

    pub fn zero() -> Self {
        Octonion { c: [0; 8] }
    }
    
    // Conjugate: Reals stay same, Imaginary parts negated mod Q
    pub fn conjugate(&self) -> Self {
        let mut new_c = [0; 8];
        new_c[0] = self.c[0];
        for i in 1..8 {
            if self.c[i] == 0 {
                new_c[i] = 0;
            } else {
                new_c[i] = Q - self.c[i];
            }
        }
        Octonion::new(new_c)
    }

    /// Returns the L2 norm squared of the octonion coefficients
    pub fn norm_sq(&self) -> f64 {
        self.c.iter().map(|&x| (x as f64).powi(2)).sum()
    }
}

// --- OCTONION ARITHMETIC (Modular) ---

impl Add for Octonion {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        let mut res = [0; 8];
        for i in 0..8 {
            res[i] = (self.c[i] + other.c[i]) % Q;
        }
        Octonion::new(res)
    }
}

impl Sub for Octonion {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        let mut res = [0; 8];
        for i in 0..8 {
            // Add Q to prevent underflow before modulo
            res[i] = (self.c[i] + Q - other.c[i]) % Q;
        }
        Octonion::new(res)
    }
}

// Cayley-Dickson Multiplication
// (a, b)(c, d) = (ac - d*b_conj, a_conj*d + cb)
impl Mul for Octonion {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        let split = |o: &Octonion| -> ([Scalar; 4], [Scalar; 4]) {
            let mut a = [0; 4];
            let mut b = [0; 4];
            a.copy_from_slice(&o.c[0..4]);
            b.copy_from_slice(&o.c[4..8]);
            (a, b)
        };

        let (a, b) = split(&self);
        let (c, d) = split(&other);

        // Quaternion helpers (Mod Q)
        let qadd = |x: [Scalar;4], y: [Scalar;4]| -> [Scalar;4] {
            [ (x[0]+y[0])%Q, (x[1]+y[1])%Q, (x[2]+y[2])%Q, (x[3]+y[3])%Q ]
        };
        
        let qsub = |x: [Scalar;4], y: [Scalar;4]| -> [Scalar;4] {
            [ (x[0]+Q-y[0])%Q, (x[1]+Q-y[1])%Q, (x[2]+Q-y[2])%Q, (x[3]+Q-y[3])%Q ]
        };

        let qconj = |x: [Scalar;4]| -> [Scalar;4] {
            [ x[0], (Q-x[1])%Q, (Q-x[2])%Q, (Q-x[3])%Q ]
        };

        let qmul = |x: [Scalar;4], y: [Scalar;4]| -> [Scalar;4] {
            // r = x0y0 - x1y1 - x2y2 - x3y3
            let r = (x[0]*y[0] + Q - (x[1]*y[1])%Q + Q - (x[2]*y[2])%Q + Q - (x[3]*y[3])%Q) % Q;
            // i = x0y1 + x1y0 + x2y3 - x3y2
            let i = (x[0]*y[1] + x[1]*y[0] + x[2]*y[3] + Q - (x[3]*y[2])%Q) % Q;
            // j = x0y2 - x1y3 + x2y0 + x3y1
            let j = (x[0]*y[2] + Q - (x[1]*y[3])%Q + x[2]*y[0] + x[3]*y[1]) % Q;
            // k = x0y3 + x1y2 - x2y1 + x3y0
            let k = (x[0]*y[3] + x[1]*y[2] + Q - (x[2]*y[1])%Q + x[3]*y[0]) % Q;
            [r, i, j, k]
        };

        // 1. ac - d * b_conj
        let ac = qmul(a, c);
        let b_conj = qconj(b);
        let d_b_conj = qmul(d, b_conj);
        let first = qsub(ac, d_b_conj);

        // 2. a_conj * d + c * b
        let a_conj = qconj(a);
        let a_conj_d = qmul(a_conj, d);
        let cb = qmul(c, b);
        let second = qadd(a_conj_d, cb);

        let mut res = [0; 8];
        res[0..4].copy_from_slice(&first);
        res[4..8].copy_from_slice(&second);
        Octonion::new(res)
    }
}

// --- 27-DIM ALBERT ELEMENT ---
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AlbertElement {
    pub alpha: Scalar, 
    pub beta: Scalar, 
    pub gamma: Scalar,
    pub a: Octonion,
    pub b: Octonion,
    pub c: Octonion,
}

impl AlbertElement {
    pub fn zero() -> Self {
        AlbertElement {
            alpha: 0, beta: 0, gamma: 0,
            a: Octonion::zero(),
            b: Octonion::zero(),
            c: Octonion::zero(),
        }
    }

    /// Sample Uniform Noise (Symmetric Phase)
    pub fn sample_uniform<R: Rng + ?Sized>(rng: &mut R, shape_beta: f64, scale: f64) -> Self {
        let dist = Weibull::new(scale, shape_beta).unwrap();
        let sample = |r: &mut R| -> u64 { (dist.sample(r) as u64) % Q };
        
        let mut el = Self::zero();
        el.alpha = sample(rng);
        el.beta = sample(rng);
        el.gamma = sample(rng);

        for i in 0..8 { el.a.c[i] = sample(rng); }
        for i in 0..8 { el.b.c[i] = sample(rng); }
        for i in 0..8 { el.c.c[i] = sample(rng); }
        el
    }

    /// Sample Structured Noise (Broken Symmetry Phase)
    pub fn sample_structured<R: Rng + ?Sized>(
        rng: &mut R, 
        shape_beta: f64, 
        scale_diag: f64, 
        scale_bulk: f64 
    ) -> Self {
        let dist_diag = Weibull::new(scale_diag, shape_beta).unwrap();
        let dist_bulk = Weibull::new(scale_bulk, shape_beta).unwrap();
        
        let s_diag = |r: &mut R| -> u64 { (dist_diag.sample(r) as u64) % Q };
        let s_bulk = |r: &mut R| -> u64 { (dist_bulk.sample(r) as u64) % Q };

        let mut el = Self::zero();
        el.alpha = s_diag(rng);
        el.beta = s_diag(rng);
        el.gamma = s_diag(rng);
        
        for i in 0..8 { el.a.c[i] = s_bulk(rng); }
        for i in 0..8 { el.b.c[i] = s_bulk(rng); }
        for i in 0..8 { el.c.c[i] = s_bulk(rng); }
        el
    }
    
    // --- JORDAN ALGEBRA OPERATIONS ---

    // Scale by a scalar (Modulo Q)
    // IMPORTANT: Because 'factor' is a scalar (Real number), this operation 
    // is associative with matrix multiplication: A(s*c) = (As)c.
    pub fn scale(&self, factor: Scalar) -> Self {
        let f = factor % Q;
        let mut res = Self::zero();
        res.alpha = (self.alpha * f) % Q;
        res.beta = (self.beta * f) % Q;
        res.gamma = (self.gamma * f) % Q;
        
        let scale_oct = |o: Octonion| -> Octonion {
            let mut c = [0; 8];
            for i in 0..8 { c[i] = (o.c[i] * f) % Q; }
            Octonion::new(c)
        };
        
        res.a = scale_oct(self.a);
        res.b = scale_oct(self.b);
        res.c = scale_oct(self.c);
        res
    }

    // Jordan Product: X o Y = XY + YX
    // Note: We use the symmetrized product without the 1/2 factor to stay in the integer ring.
    pub fn jordan_product(&self, other: &Self) -> Self {
        // Helpers for 3x3 matrix extraction
        let get_row = |m: &AlbertElement, i: usize| -> [Octonion; 3] {
            let to_oct = |s: Scalar| -> Octonion { 
                let mut c = [0; 8]; c[0] = s; Octonion::new(c) 
            };
            match i {
                0 => [to_oct(m.alpha), m.c, m.b], // Row 1: [a, c, b] (Note: c is (1,2), b is (1,3) in this notation)
                1 => [m.c.conjugate(), to_oct(m.beta), m.a], // Row 2: [c*, b, a]
                2 => [m.b.conjugate(), m.a.conjugate(), to_oct(m.gamma)], // Row 3: [b*, a*, g]
                _ => panic!("Invalid row")
            }
        };

        // Dot product of vector of octonions
        let dot = |r: [Octonion; 3], c: [Octonion; 3]| -> Octonion {
            (r[0] * c[0]) + (r[1] * c[1]) + (r[2] * c[2])
        };

        let x = self;
        let y = other;

        // Calculate Diagonal 1 (Alpha)
        // (XY)_11 + (YX)_11
        // (XY)_11 = Row1(X) . Col1(Y). Note Col1(Y) is Row1(Y)* (Conjugate transpose)
        // Since Albert elements are Hermitian, Col(i) is Row(i) conjugated.
        // let row_x_0 = get_row(x, 0);
        // let row_y_0 = get_row(y, 0);
        
        // Helper to get column j from element m
        let get_col = |m: &AlbertElement, j: usize| -> [Octonion; 3] {
            let r = get_row(m, j);
            [r[0].conjugate(), r[1].conjugate(), r[2].conjugate()]
        };

        // Diagonals (Real part of Octonion result)
        let d1 = dot(get_row(x, 0), get_col(y, 0)) + dot(get_row(y, 0), get_col(x, 0));
        let d2 = dot(get_row(x, 1), get_col(y, 1)) + dot(get_row(y, 1), get_col(x, 1));
        let d3 = dot(get_row(x, 2), get_col(y, 2)) + dot(get_row(y, 2), get_col(x, 2));

        // Off-Diagonals
        // (XY)_12 + (YX)_12
        let od_c = dot(get_row(x, 0), get_col(y, 1)) + dot(get_row(y, 0), get_col(x, 1)); // (1,2) -> c
        let od_b = dot(get_row(x, 0), get_col(y, 2)) + dot(get_row(y, 0), get_col(x, 2)); // (1,3) -> b
        let od_a = dot(get_row(x, 1), get_col(y, 2)) + dot(get_row(y, 1), get_col(x, 2)); // (2,3) -> a

        AlbertElement {
            alpha: d1.c[0], // Extract real part
            beta: d2.c[0],
            gamma: d3.c[0],
            c: od_c,
            b: od_b,
            a: od_a,
        }
    }

    // Check bounds (L-infinity norm) for rejection sampling
    pub fn exceeds_bound(&self, bound: Scalar) -> bool {
        if self.alpha > bound || self.beta > bound || self.gamma > bound { return true; }
        
        let check_oct = |o: &Octonion| -> bool {
            o.c.iter().any(|&x| x > bound)
        };
        
        check_oct(&self.a) || check_oct(&self.b) || check_oct(&self.c)
    }
}

// --- ALBERT ARITHMETIC ---

impl Add for AlbertElement {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        AlbertElement {
            alpha: (self.alpha + other.alpha) % Q,
            beta: (self.beta + other.beta) % Q,
            gamma: (self.gamma + other.gamma) % Q,
            a: self.a + other.a,
            b: self.b + other.b,
            c: self.c + other.c,
        }
    }
}

impl Sub for AlbertElement {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        AlbertElement {
            alpha: (self.alpha + Q - other.alpha) % Q,
            beta: (self.beta + Q - other.beta) % Q,
            gamma: (self.gamma + Q - other.gamma) % Q,
            a: self.a - other.a,
            b: self.b - other.b,
            c: self.c - other.c,
        }
    }
}