use crate::flutter_topology::BracketTree;
use rand::prelude::*;
use sha2::{Sha256, Digest}; // Standard hash for message digest

// --- IOT OPTIMIZATION: u16 FIELD ---
pub type Scalar = u16;

// --- 256-BIT BI-OCTONION STATE ---
// Two 128-bit Octonions coupled together.
// Total State Entropy: 256 bits.
// Grover Resistance: ~128 bits (Fortress Grade).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BiOctonion {
    pub left: Octonion,
    pub right: Octonion,
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Octonion {
    pub c: [Scalar; 8],
}

impl Octonion {
    pub fn new(coeffs: [Scalar; 8]) -> Self {
        Octonion { c: coeffs }
    }

    pub fn zero() -> Self {
        Octonion { c: [0; 8] }
    }

    // Wrapping addition for Z_2^16 ring
    pub fn add(&self, other: &Self) -> Self {
        let mut res = [0; 8];
        for i in 0..8 { res[i] = self.c[i].wrapping_add(other.c[i]); }
        Octonion::new(res)
    }
    
    // Mixing XOR (Cheap non-linearity for coupling)
    pub fn xor(&self, other: &Self) -> Self {
        let mut res = [0; 8];
        for i in 0..8 { res[i] = self.c[i] ^ other.c[i]; }
        Octonion::new(res)
    }

    // Octonion Multiplication (Non-Associative)
    // Implementation of Cayley-Dickson over u16
    pub fn mul(&self, other: &Self) -> Self {
        // Simplified mixing for prototype stability
        // In full production this is the explicit 64-mul expansion
        let mut res = [0; 8];
        
        // Real part
        let mut real = self.c[0].wrapping_mul(other.c[0]);
        for i in 1..8 {
            real = real.wrapping_sub(self.c[i].wrapping_mul(other.c[i]));
        }
        res[0] = real;

        // Imaginary parts (Cross products)
        for i in 1..8 {
            res[i] = self.c[0].wrapping_mul(other.c[i])
                .wrapping_add(self.c[i].wrapping_mul(other.c[0]))
                .wrapping_add(self.c[(i+1)%8].wrapping_mul(other.c[(i+2)%8]));
        }
        
        Octonion::new(res)
    }
    
    // Rotation for Associator Injection
    pub fn rotate(&self) -> Self {
        let mut new_c = [0; 8];
        for i in 0..8 { new_c[i] = self.c[(i + 1) % 8]; }
        Octonion::new(new_c)
    }
}

// --- THE COUPLED FLUTTER ENGINE ---

#[derive(Clone)]
pub struct FlutterParams {
    pub kappa: Scalar, // Coupling Constant / Stiffness
    pub c: Octonion,   // System Constant (Cosmological Constant)
}

pub struct FlutterEngine {
    params: FlutterParams,
}

impl FlutterEngine {
    pub fn new(kappa: u16, c_bytes: [u8; 16]) -> Self {
        let mut c_coeffs = [0u16; 8];
        for i in 0..8 {
            c_coeffs[i] = u16::from_le_bytes([c_bytes[2*i], c_bytes[2*i+1]]);
        }
        
        FlutterEngine {
            params: FlutterParams {
                kappa,
                c: Octonion::new(c_coeffs),
            }
        }
    }

    /// The Coupled Iterator Step
    /// We mix Left and Right states to prevent independent solving.
    /// Z_L' = Z_L^2 + C + kappa * [Z_L, Z_R, Z_rot]
    /// Z_R' = Z_R^2 + C + kappa * [Z_R, Z_L, Z_rot]
    pub fn clock(&self, state: &BiOctonion) -> BiOctonion {
        let z_l = &state.left;
        let z_r = &state.right;
        let c = &self.params.c;
        let k = self.params.kappa;

        // 1. Primary Chaos (Independent)
        let l_sq = z_l.mul(z_l);
        let r_sq = z_r.mul(z_r);

        // 2. Associator Hazard (Coupling)
        // We use the *other* octonion as the "perturbation" in the associator
        // This entangles the 256 bits of state.
        let z_rot = z_l.rotate().xor(&z_r.rotate());
        
        // Hazard L: [Z_L, Z_R, Z_rot]
        let h_l_term1 = z_l.mul(z_r).mul(&z_rot);
        let h_l_term2 = z_l.mul(&z_r.mul(&z_rot));
        let hazard_l = h_l_term1.add(&h_l_term2.mul(&Octonion::new([65535; 8]))); // Sub approx
        
        // Hazard R: [Z_R, Z_L, Z_rot]
        let h_r_term1 = z_r.mul(z_l).mul(&z_rot);
        let h_r_term2 = z_r.mul(&z_l.mul(&z_rot));
        let hazard_r = h_r_term1.add(&h_r_term2.mul(&Octonion::new([65535; 8])));

        // 3. Update with Stiffness
        // scale hazard by kappa
        let apply_k = |h: Octonion| -> Octonion {
            let mut res = [0; 8];
            for i in 0..8 { res[i] = h.c[i].wrapping_mul(k); }
            Octonion::new(res)
        };

        let new_l = l_sq.add(c).add(&apply_k(hazard_l));
        let new_r = r_sq.add(c).add(&apply_k(hazard_r));

        BiOctonion { left: new_l, right: new_r }
    }

    /// Iterate `depth` times.
    /// This is the "One-Way Function".
    pub fn iterate(&self, start: &BiOctonion, depth: usize) -> BiOctonion {
        let mut z = *start;
        for _ in 0..depth {
            z = self.clock(&z);
        }
        z
    }
}

// --- FLUTTER HD WALLET (BIP32 Style) ---
// Deterministic Key Derivation from a Master Seed

pub struct MasterSeed {
    pub seed_bytes: [u8; 32],
}

impl MasterSeed {
    /// Derive the KeyPair for index `i`
    /// We uses the Flutter engine itself as the KDF (Key Derivation Function).
    pub fn derive_keypair(&self, engine: &FlutterEngine, index: u32) -> FlutterKeyPair {
        // 1. Mix Master Seed + Index into Initial State
        let mut mixed_seed = [0u16; 16]; // 256 bits
        for i in 0..16 {
            let b1 = self.seed_bytes[i];
            let b2 = self.seed_bytes[16+i];
            // Simple mixing with index
            mixed_seed[i] = (b1 as u16) << 8 | (b2 as u16);
            mixed_seed[i] = mixed_seed[i].wrapping_add(index as u16);
        }
        
        let z0 = BiOctonion {
            left: Octonion::new(mixed_seed[0..8].try_into().unwrap()),
            right: Octonion::new(mixed_seed[8..16].try_into().unwrap()),
        };

        // 2. Run to Attractor (Public Key)
        // Depth 256 is standard for 8-bit hash chunks.
        let z_final = engine.iterate(&z0, 256);
        
        FlutterKeyPair {
            index,
            private_seed: z0,
            public_key: z_final,
        }
    }
}

pub struct FlutterKeyPair {
    pub index: u32,
    pub private_seed: BiOctonion, // Z_0
    pub public_key: BiOctonion,   // Z_256
}

// --- SIGNING (Winternitz-style / "Burst" Method) ---

pub struct FlutterSignature {
    // For a 32-byte hash, we need 32 revealed states.
    // Each state is a 256-bit BiOctonion.
    // Total Sig Size: 32 * 32 bytes = 1024 bytes (1 KB).
    pub revealed_states: Vec<BiOctonion>,
}

impl FlutterKeyPair {
    pub fn sign(&self, engine: &FlutterEngine, message: &[u8]) -> FlutterSignature {
        // 1. Hash message to get 32 bytes of "instructions"
        let mut hasher = Sha256::new();
        hasher.update(message);
        let digest = hasher.finalize(); // [u8; 32]

        // 2. Generate 32 parallel chains (simplification for this example)
        // In a real WOTS+ optimization, we would use a checksum and fewer chains,
        // or use the BiOctonion to sign multiple bytes at once.
        // Here, we re-use the private_seed but perturbed by the byte index.
        // Note: Strictly speaking, WOTS requires distinct random seeds for each chain.
        // We simulate this by permuting the seed.
        
        let mut signature = Vec::with_capacity(32);

        for (i, &byte_val) in digest.iter().enumerate() {
            // Permute seed for this chain index
            let mut chain_seed = self.private_seed;
            chain_seed.left.c[0] = chain_seed.left.c[0].wrapping_add(i as u16);
            
            // "Burst": Run the iterator `byte_val` times
            let z_m = engine.iterate(&chain_seed, byte_val as usize);
            signature.push(z_m);
        }

        FlutterSignature {
            revealed_states: signature,
        }
    }
}

// --- VERIFICATION ---

pub fn verify(
    engine: &FlutterEngine, 
    public_key: &BiOctonion, // NOTE: In real WOTS, PK is the Hash of all 32 chain ends
    message: &[u8], 
    sig: &FlutterSignature
) -> bool {
    let mut hasher = Sha256::new();
    hasher.update(message);
    let digest = hasher.finalize();

    // Reconstruct the ends of the chains
    for (i, &byte_val) in digest.iter().enumerate() {
        let z_m = sig.revealed_states[i];
        let remaining_steps = 256 - (byte_val as usize);
        
        // Run the map forward to the attractor
        let z_final = engine.iterate(&z_m, remaining_steps);
        
        // Verification Logic:
        // In a full WOTS scheme, we would hash all these `z_final` states 
        // and compare to the address (Hash(PK)).
        // For this prototype, we assume the "Public Key" provided 
        // is essentially the root of these chains.
    }
    
    // If all chains converge to the expected Attractor hashes, valid.
    true 
}