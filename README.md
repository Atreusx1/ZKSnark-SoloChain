# Enhanced zkSNARK-based Transaction System

## Introduction

This project aims to create a zkSNARK-based private transaction system built using the Substrate framework. By leveraging Circom for designing cryptographic circuits and integrating them into a custom Substrate runtime pallet (`pallet-zksnark`), the system enables secure and private on-chain verification of zero-knowledge proofs.



## Features

- **Enhanced Privacy:** Zero-knowledge proofs ensure sensitive transaction details (sender, receiver, amount) remain confidential while verifying their validity.
- **Scalable Verification:** The use of zkSNARKs allows efficient and scalable on-chain proof verification, reducing computational overhead for validators.
- **Customizable Circuits:** Circom circuits allow modular designs for a variety of use cases, including single transactions, batch transactions, and Merkle tree validation.
- **On-Chain Integration:** The project integrates zkSNARK verification logic directly into the Substrate runtime.

## Components

1. **Circom Circuits**  
   Circom is used to define cryptographic circuits. Two major circuits are included:  
   - `EnhancedTransaction.circom`: Handles private transaction inputs, including sender, receiver, and amount, and enforces constraints such as valid range checks and hash-based commitments.  
   - `MerkleTree.circom`: Verifies membership in a Merkle tree to ensure the transaction is part of a valid dataset.  

2. **Proof System**  
   The `snarkjs` tool is used to generate proofs and verification keys.  
   - The `.r1cs` file contains the circuit's constraints.  
   - The `.zkey` file is used for generating zkSNARK proofs.  
   - The `verification_key.json` is used for on-chain proof verification.  

3. **Substrate Pallet**  
   The custom `pallet-zksnark` handles the logic for verifying zkSNARK proofs. It includes:  
   - Storage for the verification key.  
   - Functions for proof submission and verification.  
   - Events to log the success or failure of proof verification.  

4. **Workflow Automation**  
   A `build.rs` script automates the process of compiling circuits and integrating their outputs into the Substrate runtime. This ensures seamless updates when the circuits are modified.
---
# Architectural Diagram
![image](https://github.com/user-attachments/assets/d8fc28d9-3b6e-49be-85df-091ae73c98cd)
---

## Workflow

### Step 1: Circuit Design in Circom

Circuits are designed in `.circom` files to define the cryptographic logic. Here's a snippet of the `EnhancedTransaction` circuit:
```
circom
template EnhancedTransaction() {
    signal private input sender;
    signal private input receiver;
    signal private input amount;
    signal output commitment;

    // Poseidon Hash for the commitment
    signal intermediate_hash;
    intermediate_hash <== Poseidon([sender, receiver, amount]);

    // Range checks for valid inputs
    sender >= 0;
    receiver >= 0;
    amount >= 0;

    commitment <== intermediate_hash;
}
```
```
This circuit:
Accepts sender, receiver, and amount as private inputs.
Computes a Poseidon hash as the commitment.
```
Ensures all inputs are non-negative using range constraints.
### Step 2: Circuit Compilation
The circuits are compiled using the circom tool to generate .r1cs, .wasm, and .zkey files. These outputs are essential for generating and verifying zkSNARK proofs.

Commands:

bash

``` circom enhanced_transaction.circom --r1cs --wasm --sym ```

``` snarkjs groth16 setup enhanced_transaction.r1cs pot12_final.ptau circuit_final.zkey ```

```snarkjs zkey export verificationkey circuit_final.zkey verification_key.json```

### Step 3: Proof Generation
Proofs are generated using the .wasm file and the .zkey proving key. This step can be performed off-chain by users.

Example proof generation:
```
javascript

const { generateProof } = require("snarkjs");

const input = {
  sender: 123,
  receiver: 456,
  amount: 789,
};

generateProof(input, "enhanced_transaction.wasm", "circuit_final.zkey"); 
```
### Step 4: Substrate Runtime Integration
The pallet-zksnark integrates the zkSNARK verification logic into the Substrate runtime. The runtime uses the verification_key.json to verify proofs on-chain.

Key Rust snippet:
```
rust
#[pallet::call]
fn submit_proof(
    origin: OriginFor<T>,
    proof: Vec<u8>,
    public_inputs: Vec<u8>,
) -> DispatchResult {
    let sender = ensure_signed(origin)?;

    // Verify the proof using the on-chain verification key
    ensure!(
        Self::verify_proof(proof, public_inputs),
        Error::<T>::InvalidProof
    );

    // Emit an event if verification succeeds
    Self::deposit_event(Event::ProofVerified { sender });
    Ok(())
```
### Step 5: On-Chain Verification
When a user submits a proof, the Substrate runtime validates it using the pallet-zksnark. Verified proofs ensure that all transaction constraints are satisfied without revealing private data.


# Drawbacks

**Trusted Setup Dependency:** zkSNARKs require a trusted setup, which introduces a potential risk if the setup phase is compromised.

**High Computational Costs:** Proof generation and on-chain verification can be computationally expensive.

**Storage Overhead:** Verifying keys and large proofs may require significant storage space.

**Limited Scalability:** While zkSNARKs reduce transaction data, verifying multiple proofs in a single block could strain resources.

**Complex Development Process:** Circuit design, compilation, and integration demand specialized knowledge.

## Acknowledgements

This project was made possible with the support and contributions of several tools, libraries, and resources. Special thanks to:

- **[Circom](https://docs.circom.io/):** For providing a powerful tool to define and compile zkSNARK circuits.
- **[SnarkJS](https://github.com/iden3/snarkjs):** For enabling proof generation and verification workflows.
- **[Substrate Framework](https://substrate.dev):** For its robust blockchain development framework and modular architecture.
- **[Arkworks](https://arkworks.rs/):** For cryptographic libraries used in Rust-based zkSNARK integrations.
- **[Poseidon Hash](https://eprint.iacr.org/2019/458):** For the efficient hash function used in the cryptographic circuit design.
- **Polkadot and Phala Network Documentation:** For inspiration in privacy-focused blockchain development.
- Open-source contributors, researchers, and the blockchain community for their ongoing efforts to innovate in the field of zero-knowledge and privacy technologies.

---

## Roadmap

Here’s a high-level roadmap to outline future development plans for this project:

### Phase 1: Core Features (✅ Almost Completed)
- Design and implement zkSNARK circuits for private transactions.
- Integrate Circom-generated outputs into a Substrate-based blockchain runtime.
- Implement proof verification logic in `pallet-zksnark`.
- Automate circuit compilation and deployment workflows using `build.rs`.

### Phase 2: Advanced Features (🔄 In Progress)
- Add support for Merkle tree-based proof validation to enhance transaction scalability.
- Implement batch transaction support in Circom circuits to reduce proof generation overhead.
- Integrate advanced range proofs to handle more complex constraints.

### Phase 3: Developer Tools (🔜 Upcoming)
- Create a user-friendly CLI tool for automating proof generation and submission.
- Build a React-based front-end for submitting private transactions to the blockchain.
- Develop detailed documentation and examples to help developers integrate this solution.

### Phase 4: Optimization and Testing (🔜 Upcoming)
- Optimize circuit designs to minimize proof size and generation time.
- Conduct stress testing and benchmarking on large-scale datasets.
- Explore support for Groth16 alternatives, such as PLONK, to reduce trusted setup dependency.

### Phase 5: Community Adoption (🚀 Future Goals)
- Open-source the project and encourage community contributions.
- Conduct developer workshops and create video tutorials for onboarding.
- Deploy a live testnet to allow users to experiment with the system.
- Explore cross-chain interoperability with Ethereum and Polkadot ecosystems.

---

### Contribute to the Roadmap
We welcome suggestions and contributions from the community to improve and expand this project. Feel free to open an issue or submit a pull request to share your ideas.
