use halo2_proofs::poseidon::PoseidonChip;
use sp_std::vec::Vec;
use pasta_curves::pallas;

pub struct MerkleTree {
    depth: usize,
    leaves: Vec<pallas::Base>,
    nodes: Vec<Vec<pallas::Base>>,
}

impl MerkleTree {
    pub fn new(depth: usize) -> Self {
        let num_leaves = 1 << depth;
        MerkleTree {
            depth,
            leaves: Vec::with_capacity(num_leaves),
            nodes: vec![Vec::new(); depth + 1],
        }
    }

    pub fn insert(&mut self, leaf: pallas::Base) -> Result<usize, &'static str> {
        if self.leaves.len() >= 1 << self.depth {
            return Err("Tree is full");
        }

        let index = self.leaves.len();
        self.leaves.push(leaf);
        self.update_tree(index);
        Ok(index)
    }

    fn update_tree(&mut self, index: usize) {
        let mut current_index = index;
        let leaf = self.leaves[current_index];
        let mut current_hash = leaf;
        let mut current_depth = 0;

        while current_depth <= self.depth {
            let sibling_index = if current_index % 2 == 0 {
                current_index + 1
            } else {
                current_index - 1
            };

            let sibling_hash = if sibling_index < self.nodes[current_depth].len() {
                self.nodes[current_depth][sibling_index]
            } else {
                pallas::Base::zero()
            };

            current_hash = PoseidonChip::hash_two(current_hash, sibling_hash);

            if current_depth + 1 <= self.depth {
                let parent_index = current_index / 2;
                if self.nodes[current_depth + 1].len() <= parent_index {
                    self.nodes[current_depth + 1].resize(parent_index + 1, pallas::Base::zero());
                }
                self.nodes[current_depth + 1][parent_index] = current_hash;
            }

            current_index /= 2;
            current_depth += 1;
        }
    }

    pub fn get_root(&self) -> pallas::Base {
        *self.nodes[self.depth].first().unwrap_or(&pallas::Base::zero())
    }

    pub fn get_proof(&self, index: usize) -> Result<(Vec<pallas::Base>, Vec<bool>), &'static str> {
        if index >= self.leaves.len() {
            return Err("Leaf index out of bounds");
        }

        let mut proof = Vec::with_capacity(self.depth);
        let mut path_indices = Vec::with_capacity(self.depth);
        let mut current_index = index;

        for depth in 0..self.depth {
            let sibling_index = if current_index % 2 == 0 {
                current_index + 1
            } else {
                current_index - 1
            };

            if sibling_index < self.nodes[depth].len() {
                proof.push(self.nodes[depth][sibling_index]);
            } else {
                proof.push(pallas::Base::zero());
            }

            path_indices.push(current_index % 2 == 0);
            current_index /= 2;
        }

        Ok((proof, path_indices))
    }
}

pub fn compute_merkle_root(leaves: &[pallas::Base]) -> pallas::Base {
    let mut tree = MerkleTree::new(20);
    for leaf in leaves {
        tree.insert(*leaf).unwrap();
    }
    tree.get_root()
}

pub fn verify_merkle_proof(
    leaf: pallas::Base,
    proof: &[pallas::Base],
    path_indices: &[bool],
    root: pallas::Base,
) -> bool {
    let mut current_hash = leaf;
    for (i, (&element, &is_left)) in proof.iter().zip(path_indices).enumerate() {
        let sibling = if is_left { element } else { current_hash };
        current_hash = PoseidonChip::hash_two(current_hash, sibling);
    }
    current_hash == root
}