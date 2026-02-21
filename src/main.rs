use olc_research::vdf;
use olc_research::gsh;
use olc_research::synergeia_sim;
use olc_research::hdwallet;
use olc_research::flt_cipher;
use olc_research::jordan_sig;
use olc_research::horizon;
use olc_research::horizon_net;
use olc_research::stark;
use olc_research::stark_vdf;
use std::time::Instant;


fn main() {
    stark_vdf::test_octostark_vdf_trace();
    stark_vdf::test_e2e_proof();
    println!("\n=================================================================");
    println!("=== OctoSTARK VDF: End-to-End Asymmetric Verification ===");
    println!("=================================================================\n");

    let iterations = 200_000;
    let security_queries = 40; // \lambda parameter for FRI
    
    let c = vdf::Octonion::from_seed(12345);
    let z_0 = vdf::Octonion::from_seed(67890);

    // ------------------------------------------------------------------------
    // PHASE 1: VDF EVALUATION (Symmetric Delay, O(T) Time)
    // ------------------------------------------------------------------------
    println!("[Phase 1] Prover: Evaluating VDF (T = {} iterations)...", iterations);
    let start_eval = Instant::now();
    
    let stark_output = vdf::evaluate_vdf(z_0, c, iterations);
    
    let eval_time = start_eval.elapsed();
    println!("   > VDF Evaluation Complete in {:.4}s", eval_time.as_secs_f64());
    println!("   > Final State [0]: {:?}", stark_output.final_state.coeffs[0].0);

    let pub_inputs = stark::PublicInputs {
        z_0,
        c,
        z_t: stark_output.final_state,
        t_iterations: iterations,
    };

    // ------------------------------------------------------------------------
    // PHASE 2: PROOF GENERATION (O(T log^2 T) Time)
    // ------------------------------------------------------------------------
    println!("\n[Phase 2] Prover: Generating zk-STARK Proof from Execution Trace...");
    let start_prove = Instant::now();
    
    let proof = stark::StarkProver::prove(&stark_output.trace, &pub_inputs, security_queries);
    
    let prove_time = start_prove.elapsed();
    println!("   > STARK Proof Generated in {:.4}s", prove_time.as_secs_f64());
    println!("   > Number of FRI trace queries sampled: {}", proof.queried_rows.len());

    // ------------------------------------------------------------------------
    // PHASE 3: ASYMMETRIC VERIFICATION (O(log^2 T) Time)
    // ------------------------------------------------------------------------
    println!("\n[Phase 3] Verifier: Validating VDF Asymmetrically...");
    let start_verify = Instant::now();
    
    let is_valid = stark::StarkVerifier::verify(&proof, &pub_inputs);
    
    let verify_time = start_verify.elapsed();
    
    if is_valid {
        println!("   > RESULT: VDF Payload VERIFIED SUCCESSFULLY!");
    } else {
        println!("   > RESULT: VDF Payload REJECTED.");
    }
    
    println!("   > Verification Time: {:.6}s", verify_time.as_secs_f64());
    
    let total_prover_time = eval_time.as_secs_f64() + prove_time.as_secs_f64();
    let asymmetry = total_prover_time / verify_time.as_secs_f64();
    println!("   > Asymmetric Speedup: {:.0}x", asymmetry);
    println!("=================================================================\n");

    println!("===========================================");
    println!("=== HORIZON: Stateless PQ Blockchain ===");
    println!("===========================================");
    println!("State Model: Holographic (Root encodes Bulk)");

    // 1. Setup: Create the Global Accumulator (The "Bulk")
    let mut accumulator = horizon::HorizonAccumulator::new();
    let mut rng = rand::thread_rng();

    // 2. User A receives a UTXO (Minting)
    println!("[1] Minting UTXO for User A...");
    let alice_keys = jordan_sig::JordanSchnorr::keygen(&mut rng);
    let bob_keys = jordan_sig::JordanSchnorr::keygen(&mut rng);

    let utxo_a = horizon::Utxo {
        id: [0xAA; 32],
        owner: alice_keys.pub_key,
        amount: 50,
    };
    
    // Position in the tree (Address space)
    let utxo_index = 12345; 
    accumulator.add_utxo(&utxo_a, utxo_index);
    
    let genesis_root = accumulator.root.clone();
    println!("    Genesis Horizon (Root): {}...", &genesis_root[0..16]);

    // 3. Stateless Validator comes online
    // It knows ONLY the Root, not the UTXO set.
    let validator = horizon::HorizonValidator::new(genesis_root.clone());

    // 4. User A creates a Transaction to User B
    println!("\n[2] User A creates Transaction (A -> B)...");
    
    // A. User A generates their own Witness (Merkle Proof)
    // This is the "Holographic Projection" of their funds.
    let witness = accumulator.generate_witness(utxo_index);
    
    // B. User A Signs the UTXO
    let msg = utxo_a.hash().into_bytes();
    let sig = jordan_sig::JordanSchnorr::sign(&alice_keys, &msg, &mut rng);

    let tx = horizon::Transaction {
        input_utxo: utxo_a,
        witness: witness,
        signature: sig,
        new_owner: bob_keys.pub_key,
        new_amount: 50,
    };

    // 5. Validator Processes Tx (Statelessly)
    println!("\n[3] Validator verifying Tx (Stateless)...");
    match validator.process_transaction(&tx) {
        Some(new_root) => {
            println!("    [SUCCESS] Transaction Valid.");
            println!("    Old Horizon: {}...", &validator.state_root[0..16]);
            println!("    New Horizon: {}...", &new_root[0..16]);
        },
        None => println!("    [FAILURE] Transaction Invalid."),
    }

    println!("=== HORIZON: Network Bootstrapping Demo ===");

    // 1. Genesis
    let genesis_root = "GENESIS_ROOT_HASH_0000".to_string();
    
    // 2. Node A (Local) - Has 1 block
    let mut node_a = horizon_net::HorizonPeer::new(genesis_root.clone());
    node_a.mine_next_block("STATE_ROOT_A1".to_string(), 1000); // 1000 iterations

    // 3. Node B (Remote) - Has 3 blocks (Longer/Heavier chain)
    let mut node_b = horizon_net::HorizonPeer::new(genesis_root.clone());
    node_b.mine_next_block("STATE_ROOT_B1".to_string(), 1000);
    node_b.mine_next_block("STATE_ROOT_B2".to_string(), 1000);
    node_b.mine_next_block("STATE_ROOT_B3".to_string(), 1000);

    println!("Node A Tip: {}...", node_a.current_horizon);
    println!("Node B Tip: {}...", node_b.current_horizon);

    // 4. Node A bootstraps from Node B
    // In a stateful chain, A would need to download blocks B1, B2, B3 AND verify all Tx.
    // In Horizon, A only verifies the VDFs in the headers.
    horizon_net::NetworkBootstrapper::sync(&mut node_a, &node_b.chain);

    println!("Node A New Tip: {}...", node_a.current_horizon);
    
    if node_a.current_horizon == node_b.current_horizon {
        println!("[SUCCESS] Instant Bootstrap complete.");
        println!("Node A is ready to validate transactions on the new Horizon.");
    }


    println!("\n\n===========================================");
    println!("=== JORDAN-DILITHIUM: Post-Quantum Sig ===");
    println!("===========================================");
    
    // 1. Key Generation
    println!("[1] Generating Keys (Lattice setup)...");
    let mut rng = rand::thread_rng();
    let keypair = jordan_sig::JordanSchnorr::keygen(&mut rng);
    println!("    Public Key Generator (Alpha): {}", keypair.pub_key.a.alpha);
    println!("    Public Key Target (Alpha): {}", keypair.pub_key.t.alpha);

    // 2. Signing
    let tx_msg = b"User A sends 50 BTC to User B";
    println!("\n[2] Signing Transaction: {:?}", String::from_utf8_lossy(tx_msg));
    let signature = jordan_sig::JordanSchnorr::sign(&keypair, tx_msg, &mut rng);
    println!("    Signature Challenge (c): {}", signature.c);
    println!("    Signature Response (z alpha): {}", signature.z.alpha);

    // 3. Verification
    println!("\n[3] Verifying Transaction...");
    let valid = jordan_sig::JordanSchnorr::verify(&keypair.pub_key, tx_msg, &signature);
    
    if valid {
        println!("    [SUCCESS] Signature is VALID.");
        println!("    Artin's Theorem bypassed via scalar challenge.");
    } else {
        println!("    [FAILURE] Invalid Signature.");
    }
    
    // 4. Forgery Test
    println!("\n[4] Attempting Forgery...");
    let fake_msg = b"User A sends 5000 BTC to User B";
    let valid_forge = jordan_sig::JordanSchnorr::verify(&keypair.pub_key, fake_msg, &signature);
    if !valid_forge {
        println!("    [SUCCESS] Forgery detected and rejected.");
    } else {
        println!("    [FAILURE] Forgery accepted!");
    }

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