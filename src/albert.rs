// src/albert.rs
use rand::prelude::*;
use rand_distr::{Distribution, Weibull};

// We use u64 to ensure sufficient "grain" for high-beta curves, 
// preventing the step-function vulnerability.
pub type Scalar = u64;

// --- 8-DIM OCTONION ---
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Octonion {
    pub c: [Scalar; 8],
}

impl Octonion {
    pub fn zero() -> Self {
        Octonion { c: [0; 8] }
    }
    
    /// Returns the L2 norm squared of the octonion coefficients
    /// Used to measure "Geometric Hazard" magnitude
    pub fn norm_sq(&self) -> f64 {
        self.c.iter().map(|&x| (x as f64).powi(2)).sum()
    }
}

// --- 27-DIM ALBERT ELEMENT ---
// 3x3 Hermitian Matrix over Octonions
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AlbertElement {
    pub alpha: Scalar, // Diagonal (1,1) - Real
    pub beta: Scalar,  // Diagonal (2,2) - Real
    pub gamma: Scalar, // Diagonal (3,3) - Real
    pub a: Octonion,   // Off-Diagonal (2,3)
    pub b: Octonion,   // Off-Diagonal (3,1)
    pub c: Octonion,   // Off-Diagonal (1,2)
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

    /// EXPERIMENT A: UNIFORM NOISE
    /// Simulates the "Strong Buffer" / Symmetric Phase.
    /// Noise is isotropic across all 27 dimensions.
    pub fn sample_uniform<R: Rng + ?Sized>(rng: &mut R, shape_beta: f64, scale: f64) -> Self {
        let dist = Weibull::new(scale, shape_beta).unwrap();
        // Closure removed here as well for consistency, though single closure was safe
        let mut el = Self::zero();
        
        // Fill Diagonals
        el.alpha = dist.sample(rng) as u64;
        el.beta = dist.sample(rng) as u64;
        el.gamma = dist.sample(rng) as u64;

        // Fill Off-Diagonals (Bulk)
        for i in 0..8 { el.a.c[i] = dist.sample(rng) as u64; }
        for i in 0..8 { el.b.c[i] = dist.sample(rng) as u64; }
        for i in 0..8 { el.c.c[i] = dist.sample(rng) as u64; }
        
        el
    }

    /// EXPERIMENT B: STRUCTURED NOISE (Associator Shielding)
    /// Simulates the "Weak Buffer" / Symmetry Broken Phase.
    /// Noise is concentrated in the non-associative bulk to create Topological Impedance.
    pub fn sample_structured<R: Rng + ?Sized>(
        rng: &mut R, 
        shape_beta: f64, 
        scale_diag: f64, // Low noise for the "Signal" (Real center)
        scale_bulk: f64  // High noise for the "Shield" (Octonions)
    ) -> Self {
        let dist_diag = Weibull::new(scale_diag, shape_beta).unwrap();
        let dist_bulk = Weibull::new(scale_bulk, shape_beta).unwrap();

        let mut el = Self::zero();
        
        // Diagonals: Low Entropy (The "Real-Secret" domain)
        // Direct calls avoid the E0524 double-borrow error
        el.alpha = dist_diag.sample(rng) as u64;
        el.beta = dist_diag.sample(rng) as u64;
        el.gamma = dist_diag.sample(rng) as u64;
        
        // Off-Diagonals: High Entropy ("Geometric Wall")
        for i in 0..8 { el.a.c[i] = dist_bulk.sample(rng) as u64; }
        for i in 0..8 { el.b.c[i] = dist_bulk.sample(rng) as u64; }
        for i in 0..8 { el.c.c[i] = dist_bulk.sample(rng) as u64; }
        
        el
    }
    
    // Metrics for Analysis
    pub fn diag_norm(&self) -> f64 {
        ((self.alpha as f64).powi(2) + (self.beta as f64).powi(2) + (self.gamma as f64).powi(2)).sqrt()
    }
    
    pub fn bulk_norm(&self) -> f64 {
        (self.a.norm_sq() + self.b.norm_sq() + self.c.norm_sq()).sqrt()
    }
}