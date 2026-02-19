// src/jordan_sig.rs
// Jordan-Dilithium: A Post-Quantum Signature Scheme over J3(O)
// Designed for UTxO Transaction Signing in the APH Framework.

use crate::albert::{AlbertElement, Scalar};
use sha2::{Sha256, Digest};
use rand::prelude::*;

// ============================================================================
// CONFIGURATION
// ============================================================================
const GAMMA1: Scalar = 10000; // Rejection sampling bound (approx 2^13)
const GAMMA2: Scalar = 20000; // Overflow bound

// ============================================================================
// DATA STRUCTURES
// ============================================================================

#[derive(Debug, Clone)]
pub struct SecretKey {
    pub s: AlbertElement, // The secret vector (Structured Noise)
    pub pub_key: PublicKey,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PublicKey {
    pub t: AlbertElement, // t = A o s
    pub a: AlbertElement, // The Generator (Public Parameter)
}

#[derive(Debug, Clone)]
pub struct Signature {
    pub z: AlbertElement, // Response vector
    pub c: Scalar,        // Challenge (Scalar to ensure associativity)
}

// ============================================================================
// IMPLEMENTATION
// ============================================================================

pub struct JordanSchnorr;

impl JordanSchnorr {
    
    /// GENERATE KEYPAIR
    /// A: Uniformly random Albert Element (The Generator)
    /// S: Structured Noise (The Secret) - Low Norm
    pub fn keygen<R: Rng + ?Sized>(rng: &mut R) -> SecretKey {
        // 1. Sample Generator A (Public Parameter)
        // High geometric stiffness
        let a = AlbertElement::sample_uniform(rng, 1.0, 5000.0);

        // 2. Sample Secret S (Small Norm)
        // Using "Structured" sampling to hide in the bulk
        // Low values (scale ~ 10) to make "Learning" hard but "Checking" easy
        let s = AlbertElement::sample_structured(rng, 1.91, 10.0, 10.0);

        // 3. Calculate Public Key T = A o S (Jordan Product)
        let t = a.jordan_product(&s);

        SecretKey {
            s,
            pub_key: PublicKey { t, a },
        }
    }

    /// SIGN TRANSACTION
    /// Uses Fiat-Shamir with Aborts
    /// 1. y <- Random Mask
    /// 2. w = A o y
    /// 3. c = Hash(M || w)
    /// 4. z = y + c*s
    /// 5. Reject if z leaks s (norm check)
    pub fn sign<R: Rng + ?Sized>(sk: &SecretKey, msg: &[u8], rng: &mut R) -> Signature {
        loop {
            // 1. Sample Ephemeral Mask y (Random high entropy)
            let y = AlbertElement::sample_uniform(rng, 1.0, GAMMA1 as f64);

            // 2. Commitment w = A o y
            let w = sk.pub_key.a.jordan_product(&y);

            // 3. Challenge c = H(M || w)
            // We map the hash to a SCALAR. This is the distinct APH innovation.
            let c = Self::hash_to_scalar(msg, &w);

            // 4. Response z = y + c*s
            // z = y + (s * c)
            let cs = sk.s.scale(c);
            let z = y + cs;

            // 5. Rejection Sampling
            // If z is too large, it might reveal the structure of s (via subtraction z - y)
            // We want z to look like uniform noise from the range [-GAMMA2, GAMMA2]
            if z.exceeds_bound(GAMMA2) {
                continue; // Retry with new y
            }

            return Signature { z, c };
        }
    }

    /// VERIFY TRANSACTION
    /// Check: A o z == w + c*t
    ///        A o (y + cs) == A o y + c(A o s)
    ///        A o y + c(A o s) == w + c*t  <-- Valid!
    pub fn verify(pk: &PublicKey, msg: &[u8], sig: &Signature) -> bool {
        // 1. Reconstruct w' = (A o z) - (c * t)
        let a_dot_z = pk.a.jordan_product(&sig.z);
        let c_times_t = pk.t.scale(sig.c);
        
        // w_prime = a_dot_z - c_times_t
        let w_prime = a_dot_z - c_times_t;

        // 2. Reconstruct Challenge c' = H(M || w')
        let c_prime = Self::hash_to_scalar(msg, &w_prime);

        // 3. Verify Challenge Consistency
        if c_prime != sig.c {
            return false;
        }

        // 4. Bound Check
        if sig.z.exceeds_bound(GAMMA2) {
            return false;
        }

        true
    }

    // --- UTILITIES ---

    fn hash_to_scalar(msg: &[u8], w: &AlbertElement) -> Scalar {
        let mut hasher = Sha256::new();
        hasher.update(msg);
        
        // Absorb the Albert Element
        // For prototype, we hash the diagonal alpha and the first coeff of 'a'
        hasher.update(w.alpha.to_le_bytes());
        hasher.update(w.a.c[0].to_le_bytes());
        
        let result = hasher.finalize();
        
        // Fold 256 bits into a single Scalar
        let mut scalar = 0 as Scalar;
        for chunk in result.chunks(8) {
            let val = u64::from_le_bytes(chunk.try_into().unwrap_or([0; 8]));
            scalar = scalar.wrapping_add(val); 
        }
        
        // Reduce to safe challenge range (small enough to not overflow z immediately)
        // Keep it small (e.g. 10 bits) for this parameter set
        scalar % 1024 
    }
}