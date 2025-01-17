    pragma circom 2.0.0;

    include "./merkle_tree.circom";
    include "node_modules/circomlib/circuits/poseidon.circom";
    include "node_modules/circomlib/circuits/mux1.circom";
    include "node_modules/circomlib/circuits/comparators.circom";

    template EnhancedTransaction(levels) {
        //  inputs
        signal input  sender;
        signal input  receiver;
        signal input  amount;
        signal input  nullifier;
        signal input  secret;
        signal input  merklePathElements[levels];
        signal input  merklePathIndices[levels];

        // Public inputs
        signal input root;
        signal input maxAmount;

        // Outputs
        signal output nullifierHash;
        signal output newCommitment;

        // Amount range check (0 <= amount <= maxAmount)
        signal rangeCheck;
        rangeCheck <== maxAmount - amount;
        component rangeProof = LessEqThan(32);
        rangeProof.in[0] <== amount;
        rangeProof.in[1] <== maxAmount;

        // Calculate commitment
        component commitmentHasher = Poseidon(3);
        commitmentHasher.inputs[0] <== amount;
        commitmentHasher.inputs[1] <== secret;
        commitmentHasher.inputs[2] <== nullifier;
        newCommitment <== commitmentHasher.out;

        // Verify Merkle proof
        component tree = MerkleTreeChecker(levels);
        tree.leaf <== newCommitment;
        for (var i = 0; i < levels; i++) {
            tree.pathElements[i] <== merklePathElements[i];
            tree.pathIndices[i] <== merklePathIndices[i];
        }
        tree.root === root;

        // Calculate nullifier hash
        component nullifierHasher = Poseidon(2);
        nullifierHasher.inputs[0] <== nullifier;
        nullifierHasher.inputs[1] <== secret;
        nullifierHash <== nullifierHasher.out;
    }

    // For batch verification
    template BatchTransaction(levels, batchSize) {
        // Arrays of inputs for each transaction
        signal input  senders[batchSize];
        signal input  receivers[batchSize];
        signal input  amounts[batchSize];
        signal input  nullifiers[batchSize];
        signal input  secrets[batchSize];
        signal input  merklePathElements[batchSize][levels];
        signal input  merklePathIndices[batchSize][levels];

        // Public inputs
        signal input roots[batchSize];
        signal input maxAmount;
        signal input  sender;

        // Outputs
        signal output nullifierHashes[batchSize];
        signal output newCommitments[batchSize];

        // Process each transaction
        component transactions[batchSize];
        for (var i = 0; i < batchSize; i++) {
            transactions[i] = EnhancedTransaction(levels);
            
            transactions[i].sender <== senders[i];
            transactions[i].receiver <== receivers[i];
            transactions[i].amount <== amounts[i];
            transactions[i].nullifier <== nullifiers[i];
            transactions[i].secret <== secrets[i];
            transactions[i].root <== roots[i];
            transactions[i].maxAmount <== maxAmount;

            for (var j = 0; j < levels; j++) {
                transactions[i].merklePathElements[j] <== merklePathElements[i][j];
                transactions[i].merklePathIndices[j] <== merklePathIndices[i][j];
            }

            nullifierHashes[i] <== transactions[i].nullifierHash;
            newCommitments[i] <== transactions[i].newCommitment;
        }
    }

    component main = BatchTransaction(20, 4); // 20 levels, batch size 4