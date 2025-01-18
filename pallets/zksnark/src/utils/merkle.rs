//zksnark/src/merkle.rs
use sp_std::prelude::*;
use sp_runtime::traits::Hash;
use sp_io::hashing::poseidon;

// Constants for Merkle tree and proof handling
const MERKLE_TREE_LEVELS: usize = 20;

#[derive(Clone, Debug)]
pub struct MerkleTree<T: Hash> {
    pub depth: u32,
    pub leaves: Vec<T::Output>,
    pub nodes: Vec<Vec<T::Output>>, // Nodes at each level of the tree
}

impl<T: Hash> MerkleTree<T> {
    /// Create a new Merkle Tree with a given depth
    pub fn new(depth: u32) -> Self {
        let num_leaves = 2usize.pow(depth);
        MerkleTree {
            depth,
            leaves: Vec::with_capacity(num_leaves),
            nodes: vec![Vec::new(); depth as usize + 1],
        }
    }

    /// Insert a single leaf into the Merkle Tree
    pub fn insert(&mut self, leaf: T::Output) -> Result<u32, &'static str> {
        if self.leaves.len() >= 2usize.pow(self.depth) {
            return Err("Tree is full");
        }

        let index = self.leaves.len() as u32;
        self.leaves.push(leaf);
        self.update_tree(index);
        Ok(index)
    }

    /// Batch insert multiple leaves
    pub fn insert_batch(&mut self, leaves: &[T::Output]) -> Result<Vec<u32>, &'static str> {
        if self.leaves.len() + leaves.len() > 2usize.pow(self.depth) {
            return Err("Batch would exceed tree capacity");
        }

        let mut indices = Vec::with_capacity(leaves.len());
        for leaf in leaves {
            let idx = self.insert(*leaf)?;
            indices.push(idx);
        }
        Ok(indices)
    }

    /// Update the tree after inserting a leaf
    fn update_tree(&mut self, leaf_index: u32) {
        let mut current_hash = self.leaves[leaf_index as usize];
        let mut current_index = leaf_index;

        for level in 0..self.depth as usize {
            let sibling_index = current_index ^ 1; // XOR for sibling index calculation
            let sibling_hash = self.nodes[level]
                .get(sibling_index as usize)
                .copied()
                .unwrap_or_default();

            current_hash = if current_index & 1 == 0 {
                self.hash_pair(current_hash, sibling_hash)
            } else {
                self.hash_pair(sibling_hash, current_hash)
            };

            current_index >>= 1;
            if self.nodes[level + 1].len() <= current_index as usize {
                self.nodes[level + 1].resize(current_index as usize + 1, T::Output::default());
            }
            self.nodes[level + 1][current_index as usize] = current_hash;
        }
    }

    /// Compute the hash of two nodes
    fn hash_pair(&self, left: T::Output, right: T::Output) -> T::Output {
        let mut input = Vec::with_capacity(64);
        input.extend_from_slice(&left.as_ref());
        input.extend_from_slice(&right.as_ref());
        poseidon::hash(&input)
    }

    /// Get the Merkle root of the tree
    pub fn get_root(&self) -> T::Output {
        if self.nodes[self.depth as usize].is_empty() {
            T::Output::default()
        } else {
            self.nodes[self.depth as usize][0]
        }
    }

    /// Verify the root of the Merkle Tree
    pub fn verify_root(&self, root: T::Output) -> bool {
        self.get_root() == root
    }

    /// Generate a proof for a given leaf index
    pub fn get_proof(&self, index: u32) -> Result<(Vec<T::Output>, Vec<bool>), &'static str> {
        if index >= self.leaves.len() as u32 {
            return Err("Leaf index out of bounds");
        }

        let mut proof_elements = Vec::with_capacity(self.depth as usize);
        let mut proof_indices = Vec::with_capacity(self.depth as usize);
        let mut current_index = index;

        for level in 0..self.depth as usize {
            let sibling_index = current_index ^ 1;
            let is_left = current_index & 1 == 0;
            proof_indices.push(is_left);

            let sibling_hash = self.nodes[level]
                .get(sibling_index as usize)
                .copied()
                .unwrap_or_default();
            proof_elements.push(sibling_hash);

            current_index >>= 1;
        }

        Ok((proof_elements, proof_indices))
    }
}

/// Utility functions for proof handling
pub mod utils {
    use super::*;

    /// Compute a nullifier hash using Poseidon
    pub fn compute_nullifier_hash(nullifier: &[u8], secret: &[u8]) -> Vec<u8> {
        let mut input = Vec::with_capacity(64);
        input.extend_from_slice(nullifier);
        input.extend_from_slice(secret);
        poseidon::hash(&input).to_vec()
    }

    /// Compute a commitment using Poseidon
    pub fn compute_commitment(amount: u64, secret: &[u8], nullifier: &[u8]) -> Vec<u8> {
        let mut input = Vec::with_capacity(72);
        input.extend_from_slice(&amount.to_le_bytes());
        input.extend_from_slice(secret);
        input.extend_from_slice(nullifier);
        poseidon::hash(&input).to_vec()
    }
}
