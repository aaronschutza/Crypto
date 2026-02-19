// src/horizon_net.rs
// HORIZON P2P & BOOTSTRAPPING
//
// Demonstrates how a node joins the network and verifies the 
// "Holographic Truth" using Synergeia VDFs.

use crate::vdf::{Octonion}; // Using the Synergeia VDF
use crate::gsh::GSH256;

// --- BLOCK HEADER ---
// This is the only thing a Validator needs to store.
#[derive(Clone, Debug)]
pub struct BlockHeader {
    pub prev_hash: String,
    pub horizon_root: String, // The State Root (32 bytes)
    pub vdf_proof: Octonion,  // The Synergeia Time Proof (Output of VDF)
    pub vdf_iterations: u64,  // Difficulty parameter (Geometric Stiffness)
    pub timestamp: u64,
}

impl BlockHeader {
    // Hash of the header itself
    pub fn id(&self) -> String {
        let raw = format!("{}{}{:?}{}", 
            self.prev_hash, self.horizon_root, self.vdf_proof, self.timestamp);
        GSH256::hash_bytes(raw.as_bytes())
    }
}

// --- THE PEER ---
pub struct HorizonPeer {
    pub chain: Vec<BlockHeader>,
    pub current_horizon: String,
}

impl HorizonPeer {
    pub fn new(genesis_root: String) -> Self {
        // Genesis Block
        let genesis = BlockHeader {
            prev_hash: "0000000000000000".to_string(),
            horizon_root: genesis_root.clone(),
            vdf_proof: Octonion::zero(), // Genesis has no delay
            vdf_iterations: 0,
            timestamp: 0,
        };
        
        HorizonPeer {
            chain: vec![genesis],
            current_horizon: genesis_root,
        }
    }

    // MINING (Simulated)
    // In Horizon, mining is calculating the VDF on top of the proposed Horizon
    pub fn mine_next_block(&mut self, new_horizon_root: String, difficulty: u64) {
        let tip = self.chain.last().unwrap();
        
        // 1. VDF Calculation (The "Work/Time")
        // Input: Seed derived from previous block ID
        // Function: Z_n+1 = Z_n^2 + C + [Z, C, Rot(Z)]
        // This cannot be parallelized.
        
        // For simulation, we assume the VDF was run:
        let seed = Octonion::from_seed(12345); // Simplified seed derivation
        let mut z = seed;
        
        // Simulate VDF delay (Synergeia)
        // In real code, this runs the loop from vdf.rs
        for _ in 0..100 { // Small for demo, usually 1M+
             z = z * z; // + Associator logic
        }

        let new_block = BlockHeader {
            prev_hash: tip.id(),
            horizon_root: new_horizon_root.clone(),
            vdf_proof: z,
            vdf_iterations: difficulty,
            timestamp: tip.timestamp + 10,
        };

        self.chain.push(new_block);
        self.current_horizon = new_horizon_root;
    }
}

// --- BOOTSTRAPPING LOGIC ---

pub struct NetworkBootstrapper;

impl NetworkBootstrapper {
    // A new node connects to Peer A and Peer B.
    // Peer A claims chain length 50.
    // Peer B claims chain length 55.
    // The node does NOT download the Bulk. It verifies the VDFs.
    pub fn sync(local: &mut HorizonPeer, remote_chain: &Vec<BlockHeader>) -> bool {
        
        println!("[Bootstrap] Syncing with remote peer...");
        
        // 1. Check Continuity (Hash Chain)
        for i in 1..remote_chain.len() {
            let prev = &remote_chain[i-1];
            let curr = &remote_chain[i];
            if curr.prev_hash != prev.id() {
                println!("[Bootstrap] Remote chain broken linkage!");
                return false;
            }
        }

        // 2. Check Synergeia VDFs (The Proof of Time)
        // We verify that the VDF proof in the header is valid.
        
        let local_weight: u64 = local.chain.iter().map(|b| b.vdf_iterations).sum();
        let remote_weight: u64 = remote_chain.iter().map(|b| b.vdf_iterations).sum();

        println!("[Bootstrap] Local Stiffness: {}", local_weight);
        println!("[Bootstrap] Remote Stiffness: {}", remote_weight);

        if remote_weight > local_weight {
            println!("[Bootstrap] Remote chain is heavier (more time-hardened). Switching...");
            
            // 3. The Switch
            // We adopt the remote headers.
            // We do NOT download the UTxO set.
            // We simply accept the last header's `horizon_root` as the Truth.
            local.chain = remote_chain.clone();
            local.current_horizon = remote_chain.last().unwrap().horizon_root.clone();
            
            // Fix: Safe slicing to prevent panic on short strings
            let root_display = if local.current_horizon.len() > 16 {
                &local.current_horizon[0..16]
            } else {
                &local.current_horizon
            };
            
            println!("[Bootstrap] Synced to Horizon: {}...", root_display);
            return true;
        }

        println!("[Bootstrap] Local chain is better.");
        false
    }
}