// src/horizon.rs
// THE HORIZON PROTOCOL: A Stateless, Post-Quantum Blockchain Layer
// Uses GSH-256 for a Quantum-Resistant State Commitment.
//
// Concept: The "Horizon" (State Root) encodes the entropy of the 
// entire "Bulk" (UTxO Set), following the Holographic Principle.

use crate::gsh::GSH256;
use crate::jordan_sig::{JordanSchnorr, PublicKey, Signature};
use std::collections::HashMap;

// --- CONFIGURATION ---
// Depth of the Sparse Merkle Tree (2^64 address space)
const TREE_DEPTH: usize = 64; 
// Empty leaf hash (computed once)
const EMPTY_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";

// --- DATA STRUCTURES ---

#[derive(Clone, Debug)]
pub struct Utxo {
    pub id: [u8; 32],      // Unique ID (Hash of tx input)
    pub owner: PublicKey,  // Jordan-Dilithium Public Key
    pub amount: u64,       // Value
}

impl Utxo {
    pub fn hash(&self) -> String {
        // Serialize and Hash via GSH (Geometric Stiffness Hash)
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.id);
        bytes.extend_from_slice(&self.amount.to_le_bytes());
        // Serialize Owner (Albert Element - simplified for demo)
        bytes.extend_from_slice(&self.owner.t.alpha.to_le_bytes()); 
        GSH256::hash_bytes(&bytes)
    }
}

// THE STATELESS WITNESS (Holographic Projection)
// This is what the user must provide. Validators do NOT store the Bulk.
#[derive(Clone, Debug)]
pub struct Witness {
    pub siblings: Vec<String>, // Merkle Branch (Hashes)
    pub index: u64,            // Position in the tree
}

// THE TRANSACTION
#[derive(Clone, Debug)]
pub struct Transaction {
    pub input_utxo: Utxo,
    pub witness: Witness,        // Proof input exists in current Horizon
    pub signature: Signature,    // Proof owner authorizes spend
    pub new_owner: PublicKey,    // Recipient
    pub new_amount: u64,
}

// --- THE HORIZON ACCUMULATOR (Sparse Merkle Tree) ---
pub struct HorizonAccumulator {
    // In a full node, we might cache nodes, but logically we only need the root
    // to verify if the witness is provided.
    // For this simulation, we act as a "Bridge Node" that holds the data 
    // to generate witnesses for the user.
    nodes: HashMap<(usize, u64), String>, // (Level, Index) -> Hash
    pub root: String,
}

impl HorizonAccumulator {
    pub fn new() -> Self {
        HorizonAccumulator {
            nodes: HashMap::new(),
            root: Self::compute_empty_root(TREE_DEPTH),
        }
    }

    // Precompute empty roots for sparse tree
    fn compute_empty_root(height: usize) -> String {
        if height == 0 { return EMPTY_HASH.to_string(); }
        let child = Self::compute_empty_root(height - 1);
        GSH256::hash_bytes(&(child.clone() + &child).into_bytes())
    }

    // Get Node Hash (or default empty)
    fn get_node(&self, level: usize, index: u64) -> String {
        self.nodes.get(&(level, index)).cloned()
             .unwrap_or_else(|| Self::compute_empty_root(level))
    }

    // INSERT UTXO (Minting)
    pub fn add_utxo(&mut self, utxo: &Utxo, index: u64) {
        let leaf_hash = utxo.hash();
        self.update_leaf(index, leaf_hash);
    }

    // SPEND UTXO (Remove from state)
    // In SMT, we replace the leaf with Empty Hash
    pub fn remove_utxo(&mut self, index: u64) {
        self.update_leaf(index, EMPTY_HASH.to_string());
    }

    fn update_leaf(&mut self, index: u64, hash: String) {
        let mut curr_idx = index;
        let mut curr_hash = hash;

        // Store Leaf
        self.nodes.insert((0, curr_idx), curr_hash.clone());

        // Bubble up
        for level in 0..TREE_DEPTH {
            let sibling_idx = curr_idx ^ 1; // Flip last bit
            let sibling_hash = self.get_node(level, sibling_idx);

            let (left, right) = if curr_idx % 2 == 0 {
                (curr_hash, sibling_hash)
            } else {
                (sibling_hash, curr_hash)
            };

            // Hash Parent using GSH (Sedenion Sponge)
            curr_hash = GSH256::hash_bytes(&(left + &right).into_bytes());
            curr_idx /= 2;
            
            self.nodes.insert((level + 1, curr_idx), curr_hash.clone());
        }
        self.root = curr_hash;
    }

    // GENERATE WITNESS (User needs this to create a Tx)
    pub fn generate_witness(&self, index: u64) -> Witness {
        let mut siblings = Vec::new();
        let mut curr_idx = index;
        for level in 0..TREE_DEPTH {
            let sibling_idx = curr_idx ^ 1;
            siblings.push(self.get_node(level, sibling_idx));
            curr_idx /= 2;
        }
        Witness { siblings, index }
    }
}

// --- THE HORIZON VALIDATOR ---
// This struct holds NO UTXO data, only the Root Hash.
pub struct HorizonValidator {
    pub state_root: String,
}

impl HorizonValidator {
    pub fn new(root: String) -> Self {
        HorizonValidator { state_root: root }
    }

    // VERIFY AND TRANSITION
    // Returns the NEW Root if valid, or None if invalid.
    pub fn process_transaction(&self, tx: &Transaction) -> Option<String> {
        // 1. Verify Cryptographic Signature (Jordan-Dilithium)
        // Check that tx.signature matches tx.input_utxo.owner
        let msg = tx.input_utxo.hash().into_bytes();
        
        let sig_valid = JordanSchnorr::verify(&tx.input_utxo.owner, &msg, &tx.signature);
        if !sig_valid {
            println!("   [Horizon] Invalid Signature");
            return None;
        }

        // 2. Verify Witness (Merkle Inclusion Proof)
        // Does this UTXO actually exist in the current Horizon?
        let calculated_root = self.calculate_root(&tx.input_utxo.hash(), &tx.witness);
        
        if calculated_root != self.state_root {
            println!("   [Horizon] Invalid Witness (State Mismatch)");
            println!("      Expected: {}", self.state_root);
            println!("      Got:      {}", calculated_root);
            return None;
        }

        // 3. Compute New State Root
        // Stateless update: If valid, we calculate what the root WOULD be
        // if we removed the old UTXO.
        
        // Remove Old (Replace leaf with Empty)
        let root_after_removal = self.calculate_root(&EMPTY_HASH.to_string(), &tx.witness);
        
        Some(root_after_removal)
    }

    // Merkle Root calculation from leaf + branch
    fn calculate_root(&self, leaf_hash: &String, witness: &Witness) -> String {
        let mut curr_hash = leaf_hash.clone();
        let mut curr_idx = witness.index;

        for sibling in &witness.siblings {
            let (left, right) = if curr_idx % 2 == 0 {
                (curr_hash, sibling.clone())
            } else {
                (sibling.clone(), curr_hash)
            };
            
            curr_hash = GSH256::hash_bytes(&(left + &right).into_bytes());
            curr_idx /= 2;
        }
        curr_hash
    }
}