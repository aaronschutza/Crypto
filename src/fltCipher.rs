// ============================================================================
// FLUTTER: APH-Based Lightweight Stream Cipher for IoT
// ============================================================================
//
// Based on the "Vacuum Flutter Epoch" described in "Flavor from Geometry".
// This cipher simulates a chaotic octonionic vacuum state to generate
// a pseudo-random keystream.
//
// Target Architecture: 16-bit / 32-bit Microcontrollers (IoT)
// State Size: 128 bits (1 Octonion over u16)
// Key Size: 128 bits
// ============================================================================

use std::ops::{Add, Mul, BitXor};

// Use u16 for lightweight IoT compatibility
type Scalar = u16;

// ----------------------------------------------------------------------------
// Core Structure: Discrete Octonion (Z_2^16)
// ----------------------------------------------------------------------------

// REMOVED `Copy` to allow `Drop`. Added `Clone` for explicit duplication.
#[derive(Clone, Debug, PartialEq, Eq)]
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
}

// Secure Zeroization: Wipes memory when the variable goes out of scope.
impl Drop for Octonion {
    fn drop(&mut self) {
        // In a real no_std crate, we would use `ptr::write_volatile`.
        // Since we are simulating in a hosted environment (for output check),
        // we'll use a simple loop.
        for i in 0..8 {
            unsafe {
                let ptr = self.c.as_mut_ptr().add(i);
                std::ptr::write_volatile(ptr, 0);
            }
        }
    }
}

// ----------------------------------------------------------------------------
// Arithmetic Implementations (IoT Optimized)
// ----------------------------------------------------------------------------

// Using references to avoid ownership issues with non-Copy types
impl<'a, 'b> Add<&'b Octonion> for &'a Octonion {
    type Output = Octonion;
    fn add(self, other: &'b Octonion) -> Octonion {
        let mut res = [0; 8];
        for i in 0..8 {
            res[i] = self.c[i].wrapping_add(other.c[i]);
        }
        Octonion::new(res)
    }
}

// Helper for value + reference
impl Add<&Octonion> for Octonion {
    type Output = Octonion;
    fn add(self, other: &Octonion) -> Octonion {
        let mut res = [0; 8];
        for i in 0..8 {
            res[i] = self.c[i].wrapping_add(other.c[i]);
        }
        Octonion::new(res)
    }
}

impl<'a, 'b> Mul<&'b Octonion> for &'a Octonion {
    type Output = Octonion;
    fn mul(self, other: &'b Octonion) -> Octonion {
        // Standard Cayley-Dickson doubling logic optimized for arrays
        // x = (a, b), y = (c, d) -> xy = (ac - d*b_conj, a_conj*d + c*b)
        
        let a = &self.c[0..4];
        let b = &self.c[4..8];
        let c = &other.c[0..4];
        let d = &other.c[4..8];

        // Quaternion Multiply Helper
        fn qmul(x: &[Scalar], y: &[Scalar]) -> [Scalar; 4] {
            let r = x[0].wrapping_mul(y[0]).wrapping_sub(x[1].wrapping_mul(y[1]))
                    .wrapping_sub(x[2].wrapping_mul(y[2])).wrapping_sub(x[3].wrapping_mul(y[3]));
            let i = x[0].wrapping_mul(y[1]).wrapping_add(x[1].wrapping_mul(y[0]))
                    .wrapping_add(x[2].wrapping_mul(y[3])).wrapping_sub(x[3].wrapping_mul(y[2]));
            let j = x[0].wrapping_mul(y[2]).wrapping_sub(x[1].wrapping_mul(y[3]))
                    .wrapping_add(x[2].wrapping_mul(y[0])).wrapping_add(x[3].wrapping_mul(y[1]));
            let k = x[0].wrapping_mul(y[3]).wrapping_add(x[1].wrapping_mul(y[2]))
                    .wrapping_sub(x[2].wrapping_mul(y[1])).wrapping_add(x[3].wrapping_mul(y[0]));
            [r, i, j, k]
        }

        // Quaternion Conjugate Helper
        fn qconj(x: &[Scalar]) -> [Scalar; 4] {
            [x[0], (0 as Scalar).wrapping_sub(x[1]), 
                   (0 as Scalar).wrapping_sub(x[2]), 
                   (0 as Scalar).wrapping_sub(x[3])]
        }

        // First part: ac - d * b_conj
        let ac = qmul(a, c);
        let b_conj = qconj(b);
        let d_b_conj = qmul(d, &b_conj);
        let mut first = [0; 4];
        for k in 0..4 { first[k] = ac[k].wrapping_sub(d_b_conj[k]); }

        // Second part: a_conj * d + c * b
        let a_conj = qconj(a);
        let a_conj_d = qmul(&a_conj, d);
        let cb = qmul(c, b);
        let mut second = [0; 4];
        for k in 0..4 { second[k] = a_conj_d[k].wrapping_add(cb[k]); }

        let mut res = [0; 8];
        res[0..4].copy_from_slice(&first);
        res[4..8].copy_from_slice(&second);
        Octonion::new(res)
    }
}

// ----------------------------------------------------------------------------
// The Flutter Cipher (Vacuum Iterator)
// ----------------------------------------------------------------------------

pub struct FlutterCipher {
    state: Octonion,
    key_c: Octonion,
    // "Kappa" - The Geometric Stiffness / Feedback Strength
    // In physics kappa ~ 0.1. Here we map it to integer space.
    kappa: Scalar, 
}

impl FlutterCipher {
    /// Initialize with a 128-bit key (represented as 8 u16s)
    /// and a 128-bit nonce (IV).
    pub fn new(key: [u16; 8], nonce: [u16; 8]) -> Self {
        let k = Octonion::new(key);
        let n = Octonion::new(nonce);
        
        let mut cipher = FlutterCipher {
            state: n,
            key_c: k,
            // A heuristic constant derived from the "Golden Ratio" of the octonions 
            // to ensure maximum mixing (related to 1/8 phase transition).
            kappa: 0x1910, // ~1.910 scaled (Beta from paper)
        };

        // "Warm up" the vacuum - Iterate 16 times to mix Key and IV
        // This corresponds to the "Inflationary Search Phase".
        for _ in 0..16 {
            cipher.clock();
        }
        
        cipher
    }

    /// The "Octonionic Iterator" Step
    /// Z_{n+1} = Z_n^2 + C + Associator_Feedback
    fn clock(&mut self) {
        let z = &self.state;
        let c = &self.key_c;

        // 1. Primary Chaotic Map: Z^2 + C
        let z_sq = z * z;
        let map_res = z_sq + c; // Note: Using reference arithmetic

        // 2. Associator Injection (The "Hard" Part)
        // APH Physics: [Z, C, Z_conjugate]
        // This term vanishes if Z and C associate. We force non-associativity
        // by mixing in a rotated version of the state.
        
        // Simple rotation for efficiency: Swap halves
        let z_rot_coeffs = [z.c[4], z.c[5], z.c[6], z.c[7], z.c[0], z.c[1], z.c[2], z.c[3]];
        let z_rot = Octonion::new(z_rot_coeffs);

        // Calculate Associator: (Z * C) * Z_rot - Z * (C * Z_rot)
        // This is the "Topological Impedance" term.
        let term1 = &(z * c) * &z_rot;
        let term2 = z * &(c * &z_rot);
        
        // Associator Hazard
        let mut hazard_c = [0; 8];
        for i in 0..8 {
            hazard_c[i] = term1.c[i].wrapping_sub(term2.c[i]);
        }

        // Feedback: Apply stiffness
        // State += Map + Kappa * Hazard
        let mut final_c = [0; 8];
        for i in 0..8 {
            let stiff = hazard_c[i].wrapping_mul(self.kappa);
            final_c[i] = map_res.c[i].wrapping_add(stiff);
        }

        self.state = Octonion::new(final_c);
    }

    /// Generate the next byte of the keystream
    pub fn next_byte(&mut self) -> u8 {
        self.clock();
        // Extract entropy from the "Vacuum Fluctuations"
        // Mix the coefficients to get a single byte
        let s = self.state.c;
        let b = s[0] ^ s[1] ^ s[2] ^ s[3] ^ s[4] ^ s[5] ^ s[6] ^ s[7];
        (b & 0xFF) as u8
    }

    /// Encrypt/Decrypt a buffer in place (XOR stream)
    pub fn process(&mut self, data: &mut [u8]) {
        for byte in data.iter_mut() {
            *byte ^= self.next_byte();
        }
    }
}

// ----------------------------------------------------------------------------
// Test Harness
// ----------------------------------------------------------------------------
fn main() {
    println!("=== FLUTTER: IoT Vacuum Cipher ===");
    
    // 1. Define Key and Nonce (128-bit each)
    let key = [0x1337, 0xC0DE, 0xDEAD, 0xBEEF, 0xCAFE, 0xBABE, 0x8080, 0xFFFF];
    let nonce = [0, 1, 2, 3, 4, 5, 6, 7];

    println!("Key: {:X?}", key);
    println!("Nonce: {:X?}", nonce);

    // 2. Initialize Cipher
    let mut flutter = FlutterCipher::new(key, nonce);
    println!("\n[System Initialized]");
    println!("State (Post-Warmup): {:?}", flutter.state);

    // 3. Encrypt a Payload
    let payload = b"Hello, APH Vacuum!";
    let mut buffer = payload.to_vec();
    
    println!("\nOriginal: {:?}", String::from_utf8_lossy(&buffer));
    
    flutter.process(&mut buffer);
    println!("Encrypted (Hex): {:02X?}", buffer);

    // 4. Decrypt (Re-init cipher with same key/nonce)
    let mut decryptor = FlutterCipher::new(key, nonce);
    decryptor.process(&mut buffer);
    
    println!("Decrypted: {:?}", String::from_utf8_lossy(&buffer));
    
    if buffer == payload {
        println!("\n[SUCCESS] Integrity Check Passed.");
    } else {
        println!("\n[FAIL] Decryption mismatch.");
    }
}