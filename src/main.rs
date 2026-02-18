use olc_research::vdf;
use olc_research::gsh;
use olc_research::synergeia_sim;
use olc_research::hdwallet;


fn main() {
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