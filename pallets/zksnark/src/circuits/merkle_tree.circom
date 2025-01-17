pragma circom 2.0.0;

include "node_modules/circomlib/circuits/poseidon.circom";
include "node_modules/circomlib/circuits/mux1.circom";

template MerkleTreeChecker(levels) {
    signal input leaf;
    signal input pathElements[levels];
    signal input pathIndices[levels];
    signal output root;

    component poseidons[levels];
    component mux[levels];

    signal levelHashes[levels + 1];
    levelHashes[0] <== leaf;

    for (var i = 0; i < levels; i++) {
        poseidons[i] = Poseidon(2);
        mux[i] = Mux1();

        mux[i].c[0] <== levelHashes[i];
        mux[i].c[1] <== pathElements[i];
        mux[i].s <== pathIndices[i];

        poseidons[i].inputs[0] <== mux[i].out;
        poseidons[i].inputs[1] <== mux[i].out;

        levelHashes[i + 1] <== poseidons[i].out;
    }

    root <== levelHashes[levels];
}