use ark_bls12_381::{Bls12_381, G1Affine, G2Affine};
use ark_ff::{Fp256, PrimeField};
use sp_std::vec::Vec;
use codec::{Encode, Decode};
use frame_support::pallet_prelude::*;
use frame_system::{pallet_prelude::*, Config, Pallet};
use sp_runtime::DispatchError;
use ark_ec::AffineRepr;

#[derive(Clone, Encode, Decode, TypeInfo)]
pub struct Proof {
    // Groth16 proof components from circom
    pub a: Vec<u8>,      // pi_a
    pub b: Vec<u8>,      // pi_b
    pub c: Vec<u8>,      // pi_c
    pub public_inputs: Vec<u8>,
}

// Define the Error enum
#[derive(Debug, PartialEq)]
pub enum Error<T> {
    InvalidProof,
    ProofVerificationFailed,
    _Phantom(PhantomData<T>),
}

pub trait ZkSnarkVerifier {
    fn verify_proof(
        proof: &Proof,
        root: &[u8],
        nullifier_hash: &[u8],
        commitment: &[u8],
    ) -> Result<bool, DispatchError>;

    fn decode_g1_point(bytes: &[u8]) -> Result<G1Affine, &'static str>;

    fn decode_g2_point(bytes: &[u8]) -> Result<G2Affine, &'static str>;

    fn parse_verification_key(vk_bytes: &[u8]) -> Result<ark_groth16::VerifyingKey<Bls12_381>, &'static str>;

    fn verify_groth16(
        pi_a: &[u8],
        pi_b: &[u8],
        pi_c: &[u8], 
        public_inputs: &[u8],
        vk_bytes: &[u8]
    ) -> Result<bool, DispatchError>;
}

impl<T: Config> ZkSnarkVerifier for Pallet<T> {
    /// Verify the zk-SNARK proof
    fn verify_proof(
        proof: &Proof,
        root: &[u8],
        nullifier_hash: &[u8],
        commitment: &[u8],
    ) -> Result<bool, DispatchError> {
        // Load verification key that was generated during build
        let vk_bytes = include_bytes!(
            concat!(env!("OUT_DIR"), "/zksnark/verification_key.json")
        );
        
        // Convert inputs to field elements
        let mut public_inputs = Vec::new();
        
        // Add Merkle root
        public_inputs.extend_from_slice(root);
        
        // Add nullifier hash
        public_inputs.extend_from_slice(nullifier_hash);
        
        // Add commitment
        public_inputs.extend_from_slice(commitment);

        // Verify the proof using the verification key and public inputs
        let result = Self::verify_groth16(
            &proof.a,
            &proof.b, 
            &proof.c,
            &public_inputs,
            vk_bytes
        )?;

        Ok(result)
    }

    /// Internal function to verify a Groth16 proof
    fn verify_groth16(
        pi_a: &[u8],
        pi_b: &[u8],
        pi_c: &[u8], 
        public_inputs: &[u8],
        vk_bytes: &[u8]
    ) -> Result<bool, DispatchError> {
        // Read the proof points from the binary format
        let a_points = Self::decode_g1_point(pi_a)
            .map_err(|_| Error::<T>::InvalidProof)?;
            
        let b_points = Self::decode_g2_point(pi_b)
            .map_err(|_| Error::<T>::InvalidProof)?;
            
        let c_points = Self::decode_g1_point(pi_c)
            .map_err(|_| Error::<T>::InvalidProof)?;

        // Parse the verification key
        let vk = Self::parse_verification_key(vk_bytes)
            .map_err(|_| Error::<T>::InvalidProof)?;

        // Convert public inputs to field elements
        let mut inputs = Vec::new();
        for chunk in public_inputs.chunks(32) {
            let mut bytes = [0u8; 32];
            bytes.copy_from_slice(chunk);
            inputs.push(Fp256::from_be_bytes_mod_order(&bytes));
        }

        // Verify the proof using arkworks Groth16 verify
        use ark_groth16::prepare_verifying_key;
        use frame_support::traits::base16::verify_proof;
        let pvk = prepare_verifying_key(&vk);
        let verified = Self::verify_proof(
            &pvk,
            &a_points,
            &b_points,
            &c_points,
            &inputs
        ).map_err(|_| Error::<T>::ProofVerificationFailed)?;

        Ok(verified)
    }

    // Helper function to decode a G1 point from bytes
    fn decode_g1_point(bytes: &[u8]) -> Result<G1Affine, &'static str> {
        if bytes.len() != 64 {
            return Err("Invalid G1 point length");
        }
        
        let mut x_bytes = [0u8; 32];
        let mut y_bytes = [0u8; 32];
        x_bytes.copy_from_slice(&bytes[..32]);
        y_bytes.copy_from_slice(&bytes[32..]);
        
        // Use arkworks to create the point
        Ok(G1Affine::new(
            Fp256::from_be_bytes_mod_order(&x_bytes),
            Fp256::from_be_bytes_mod_order(&y_bytes)
        ))
    }

    // Helper function to decode a G2 point from bytes
    fn decode_g2_point(bytes: &[u8]) -> Result<G2Affine, &'static str> {
        if bytes.len() != 128 {
            return Err("Invalid G2 point length");
        }
        
        let mut x_c0_bytes = [0u8; 32];
        let mut x_c1_bytes = [0u8; 32];
        let mut y_c0_bytes = [0u8; 32];
        let mut y_c1_bytes = [0u8; 32];
        
        x_c0_bytes.copy_from_slice(&bytes[..32]);
        x_c1_bytes.copy_from_slice(&bytes[32..64]);
        y_c0_bytes.copy_from_slice(&bytes[64..96]);
        y_c1_bytes.copy_from_slice(&bytes[96..]);
        
        // Use arkworks to create the point
        Ok(G2Affine::new(
            Fp256::from_be_bytes_mod_order(&x_c0_bytes),
            Fp256::from_be_bytes_mod_order(&x_c1_bytes)
        ))
    }

    // Helper function to parse the verification key from JSON bytes
    fn parse_verification_key(vk_bytes: &[u8]) -> Result<ark_groth16::VerifyingKey<Bls12_381>, &'static str> {
        // Parse the JSON verification key
        let vk: serde_json::Value = serde_json::from_slice(vk_bytes)
            .map_err(|_| "Failed to parse verification key")?;
        
        // Extract the components
        let alpha = vk["alpha"]
            .as_array()
            .ok_or("Invalid alpha in vk")?;
            
        let beta = vk["beta"]
            .as_array()
            .ok_or("Invalid beta in vk")?;
            
        let gamma = vk["gamma"]
            .as_array()
            .ok_or("Invalid gamma in vk")?;
            
        let delta = vk["delta"]
            .as_array()
            .ok_or("Invalid delta in vk")?;

        // Convert to arkworks VerifyingKey
        Ok(ark_groth16::VerifyingKey {
            alpha_g1: Self::decode_g1_point(&hex::decode(&alpha[0].as_str().unwrap()).unwrap())?,
            beta_g2: Self::decode_g2_point(&hex::decode(&beta[0].as_str().unwrap()).unwrap())?,
            gamma_g2: Self::decode_g2_point(&hex::decode(&gamma[0].as_str().unwrap()).unwrap())?,
            delta_g2: Self::decode_g2_point(&hex::decode(&delta[0].as_str().unwrap()).unwrap())?,
            gamma_abc_g1: vec![], // Will be filled from IC section
        })
    }
}