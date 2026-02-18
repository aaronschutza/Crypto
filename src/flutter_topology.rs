// src/flutter_topology.rs

// Represents a node in the binary operation tree (The "Observer Bracket")
#[derive(Clone, Debug)]
pub enum BracketTree {
    Leaf(usize), // Index of the Octonion in the sequence
    Node(Box<BracketTree>, Box<BracketTree>), // (Left * Right)
}

impl BracketTree {
    // Generate a random bracketing topology for N inputs
    // This effectively samples from the Catalan distribution
    pub fn random(n: usize, rng: &mut impl rand::Rng) -> Self {
        if n == 1 {
            return BracketTree::Leaf(0);
        }
        // Recursively split the sequence [0..n] at a random pivot
        // This creates the variable topology
        let split = rng.gen_range(1..n); 
        BracketTree::Node(
            Box::new(Self::random_recursive(0, split, rng)),
            Box::new(Self::random_recursive(split, n, rng)),
        )
    }

    // Internal recursive helper to track indices
    fn random_recursive(start: usize, end: usize, rng: &mut impl rand::Rng) -> Self {
        if end - start == 1 {
            return BracketTree::Leaf(start);
        }
        let split = rng.gen_range(start + 1..end);
        BracketTree::Node(
            Box::new(Self::random_recursive(start, split, rng)),
            Box::new(Self::random_recursive(split, end, rng)),
        )
    }
    
    // Execute the topology on a sequence of inputs
    pub fn evaluate<T, F>(&self, inputs: &[T], op: &F) -> T 
    where T: Clone, F: Fn(T, T) -> T 
    {
        match self {
            BracketTree::Leaf(idx) => inputs[*idx].clone(),
            BracketTree::Node(left, right) => {
                let l_val = left.evaluate(inputs, op);
                let r_val = right.evaluate(inputs, op);
                op(l_val, r_val) // The non-associative operation
            }
        }
    }
}