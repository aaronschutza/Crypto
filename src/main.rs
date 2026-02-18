use olc_research::vdf;
use olc_research::gsh;
use olc_research::synergeia_sim;
use olc_research::hdwallet;
use olc_research::flt_cipher;


fn main() {

    println!("=== FLUTTER: IoT Vacuum Cipher ===");
    
    // 1. Define Key and Nonce (128-bit each)
    let key = [0x1337, 0xC0DE, 0xDEAD, 0xBEEF, 0xCAFE, 0xBABE, 0x8080, 0xFFFF];
    let nonce = [0, 1, 2, 3, 4, 5, 6, 7];

    println!("Key: {:X?}", key);
    println!("Nonce: {:X?}", nonce);

    // 2. Initialize Cipher
    let mut flutter = flt_cipher::FlutterCipher::new(key, nonce);
    println!("\n[System Initialized]");
    println!("State (Post-Warmup): {:?}", flutter.state);

    // 3. Encrypt a Payload
    let payload = b"Hello, Vacuum!";
    let mut buffer = payload.to_vec();
    
    println!("\nOriginal: {:?}", String::from_utf8_lossy(&buffer));
    
    flutter.process(&mut buffer);
    println!("Encrypted (Hex): {:02X?}", buffer);

    // 4. Decrypt (Re-init cipher with same key/nonce)
    let mut decryptor = flt_cipher::FlutterCipher::new(key, nonce);
    decryptor.process(&mut buffer);
    
    println!("Decrypted: {:?}", String::from_utf8_lossy(&buffer));
    
    if buffer == payload {
        println!("\n[SUCCESS] Integrity Check Passed.");
    } else {
        println!("\n[FAIL] Decryption mismatch.");
    }


    println!("=== FLUTTER ENGINE: Bi-Octonion HD Wallet ===");

    // 1. Setup Engine (Cosmological Constant)
    let kappa = 0x1910;
    let c_bytes = [0xAB; 16];
    let engine = hdwallet::FlutterEngine::new(kappa, c_bytes);

    // 2. Master Seed
    let seed = hdwallet::MasterSeed { seed_bytes: [0x42; 32] };
    
    // 3. Derive Identity
    println!("Deriving KeyPair #0...");
    let kp = seed.derive_keypair(&engine, 0);
    println!("Public Key (Z_final):\nLeft: {:?}\nRight: {:?}", kp.public_key.left.c, kp.public_key.right.c);

    // 4. Sign Message
    let msg = b"Octonions Rule The Vacuum";
    println!("\nSigning Message: {:?}", String::from_utf8_lossy(msg));
    let sig = kp.sign(&engine, msg);
    println!("Signature Generated ({} Chain States)", sig.revealed_states.len());

    // 5. Verify
    let valid = hdwallet::verify(&engine, &kp.public_key, msg, &sig);
    if valid {
        println!("\n[SUCCESS] Signature Verified.");
    } else {
        println!("\n[FAIL] Verification Failed.");
    }

    // 6. Run the VDF Benchmark from the new module
    vdf::run_benchmark();

    // 7. Run GSH-256 Demo
    println!("\n\n===========================================");
    println!("=== GSH-256: Geometric Stiffness Hash ===");
    println!("===========================================");
    let input = b"The vacuum is empty.";
    let hash = gsh::GSH256::hash_bytes(input);
    println!("Input: {:?}", String::from_utf8_lossy(input));
    println!("Hash: {}", hash);
    let input = b"The vacuum is not empty, but merely highly conductive.";
    let hash = gsh::GSH256::hash_bytes(input);
    println!("Input: {:?}", String::from_utf8_lossy(input));
    println!("Hash: {}", hash);

    // 8. Run Synergeia Consensus Simulation
    synergeia_sim::run_simulation(10_000);
}