#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{dispatch::DispatchResult, RuntimeDebug};
use sp_runtime::DispatchError;
use halo2_proofs::{plonk::{Params, VerifyingKey, Proof}, pasta::pallas};
use serde_json;
mod circuits;
mod merkle;

// Add this to the top of your lib.rs
#[derive(Serialize)]
pub struct VerifyingKeyWrapper { 
    pub alpha_g1: pallas::Affine, 
    pub beta_g2: vesta::Affine, 
    pub gamma_g2: vesta::Affine, 
    pub delta_g2: vesta::Affine, 
    pub ic: Vec<pallas::Affine>, 
}

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default, PartialEq, Clone, RuntimeDebug, TypeInfo)]
pub struct ZkProof {
    pub proof: Proof<pallas::Affine>,
    pub public_inputs: Vec<pallas::Base>,
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

#[pallet::pallet]
#[pallet::generate_store(pub(super) trait Store)]
pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {}

    #[pallet::storage]
    #[pallet::getter(fn verification_key)]
    pub type VerificationKey<T: Config> = StorageValue<_, Vec<u8>, ValueQuery>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        pub fn submit_proof(origin: OriginFor<T>, proof: ZkProof<T::Hash>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::verify_transaction_proof(&proof)?;
            Ok(())
        }
    }

    #[pallet::error]
    pub enum Error<T> {
        ProofVerificationFailed,
        InvalidProof,
    }
}use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;

impl<T: Config> Pallet<T> {
    pub fn verify_transaction_proof(proof: &ZkProof<T::Hash>) -> Result<(), DispatchError> {
        let vk: VerifyingKey<pallas::Affine> = serde_json::from_slice(
            &VerificationKey::<T>::get()
        ).map_err(|_| Error::<T>::InvalidProof)?;
        
        let params = Params::new(9);
        let public_inputs = proof.public_inputs.clone();
        
        let verifier = halo2_proofs::plonk::Verifier::new(params, vk);
        verifier
            .verify(&public_inputs, &proof.proof)
            .map_err(|_| Error::<T>::ProofVerificationFailed)?;
        
        Ok(())
    }

    pub fn generate_proof(
        sender: pallas::Base,
        receiver: pallas::Base,
        amount: pallas::Base,
        nullifier: pallas::Base,
        secret: pallas::Base,
    ) -> Result<(Proof<pallas::Affine>, Vec<pallas::Base>), DispatchError> {
        // Compute commitment
        let commitment = circuits::poseidon::PoseidonChip::hash_three(
            sender,
            receiver,
            amount,
        );

        // Generate Merkle tree and proof
        let mut tree = merkle::MerkleTree::new(20);
        tree.insert(commitment).unwrap();
        let (proof_elements, path_indices) = tree.get_proof(0).unwrap();

        // Compute nullifier hash
        let nullifier_hash = circuits::poseidon::PoseidonChip::hash_two(
            nullifier,
            secret,
        );

        // Create and synthesize the circuit
        let circuit = circuits::TransactionCircuit {
            sender: Value::known(sender),
            receiver: Value::known(receiver),
            amount: Value::known(amount),
            nullifier: Value::known(nullifier),
            secret: Value::known(secret),
            merkle_path_elements: Value::known(proof_elements),
            merkle_path_indices: Value::known(path_indices),
        };

        let params = halo2_proofs::dev::mock::load_params(9);
        let vk = halo2_proofs::dev::to_halo_verifying_key(&circuit, &params);

        let proof = halo2_proofs::dev::create_proof(
            &params,
            &vk,
            vec![circuit],
            vec![],
        )?;

        Ok((proof, vec![
            commitment,
            nullifier_hash,
            tree.get_root(),
        ]))
    }
}