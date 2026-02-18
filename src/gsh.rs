// src/gsh.rs
// Geometric Stiffness Hash (GSH-256)
// Reference: "Geometry in Action", Section 33.
// Mechanism: Sedenion Associator Sponge.

use crate::sedenion::{Sedenion, associator};
use crate::vdf::Octonion;

pub struct GSH256 {
    state: Sedenion,
}

impl GSH256 {
    pub fn new() -> Self {
        // Initial State (IV)
        // Derived from the first 16 primes to seed the geometric chaos
        let iv_low = Octonion::new([
            2, 3, 5, 7, 11, 13, 17, 19
        ]);
        let iv_high = Octonion::new([
            23, 29, 31, 37, 41, 43, 47, 53
        ]);
        
        GSH256 {
            state: Sedenion::new(iv_low, iv_high)
        }
    }

    // Absorb phase: Mixes message chunk M into the state S
    // Formula: S_new = S_old ^ [S_old, M, K]
    // where K is a round constant (Geometric stiffness injection)
    pub fn absorb(&mut self, chunk: &[u8]) {
        // 1. Map bytes to Sedenion (Padding with 0 if necessary)
        // A Sedenion is 16 * 64 bits = 1024 bits.
        // We absorb 64 bytes (512 bits) at a time to keep capacity high.
        let mut coeffs = [0u64; 16];
        
        for i in 0..16 {
            // Simple packing for demo purposes
            if i * 4 < chunk.len() {
                // Take up to 4 bytes for a simplistic fill, in real prod use full u64 packing
                let mut val = 0u64;
                for b in 0..4 {
                    if i*4 + b < chunk.len() {
                        val |= (chunk[i*4 + b] as u64) << (8 * b);
                    }
                }
                coeffs[i] = val;
            }
        }

        let msg_sed = Sedenion::new(
            Octonion::new(coeffs[0..8].try_into().unwrap()),
            Octonion::new(coeffs[8..16].try_into().unwrap())
        );

        // 2. Round Constant K (The "Stiffener")
        // We rotate the IV to act as a dynamic constant
        let k = Sedenion::new(
            self.state.high, // Swap halves
            self.state.low
        );

        // 3. The Associator Twist
        // This is the non-linear compression function.
        // In associative algebras (SHA-256 logic), this term is zero.
        // In Sedenions, it creates a "Geometric Vortex".
        let hazard = associator(self.state, msg_sed, k);

        // 4. Update State
        // S = S ^ Hazard
        // We XOR the hazard back into the state.
        // We also XOR the message linearly to ensure data injection.
        self.state = (self.state ^ hazard) ^ msg_sed;
    }

    pub fn digest(&self) -> String {
        // Squeeze phase: We just return the Hex of the final state's Low Octonion (512 bits)
        // or a folded version for 256 bits.
        // Here we fold High ^ Low to get 8 x u64.
        
        let mut result = String::new();
        for i in 0..8 {
            let val = self.state.low.coeffs[i] ^ self.state.high.coeffs[i];
            result.push_str(&format!("{:016x}", val));
        }
        result
    }
    
    // Process a full byte string
    pub fn hash_bytes(input: &[u8]) -> String {
        let mut hasher = GSH256::new();
        
        // Chunking (512-bit chunks)
        for chunk in input.chunks(64) {
            hasher.absorb(chunk);
        }
        
        // Final mixing rounds to resolve residual linearity
        // "Geometric Settling"
        for _ in 0..4 {
            hasher.absorb(&[0xFF; 64]);
        }
        
        hasher.digest()
    }
}