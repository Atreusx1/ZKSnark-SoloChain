use ark_bls12_381::{Bls12_381, Fr, G1Affine, G2Affine, Fq, Fq2};
use ark_ff::PrimeField;
use ark_groth16::{prepare_verifying_key, Groth16, VerifyingKey};
use codec::{Encode, Decode};
use frame_support::pallet_prelude::*;
use frame_system::{Config, Pallet};
use sp_std::vec::Vec;
use sp_runtime::DispatchError;
use core::marker::PhantomData;
use ark_snark::SNARK;   
use serde_json;
use hex;

#[derive(Clone, Encode, Decode, TypeInfo)]
pub struct Proof {
    pub a: Vec<u8>,
    pub b: Vec<u8>,
    pub c: Vec<u8>,
    pub public_inputs: Vec<u8>,
}

#[derive(Debug, PartialEq)] 
pub enum Error<T> {
    InvalidProof,
    ProofVerificationFailed,
    _Phantom(PhantomData<T>),
}

impl<T> From<Error<T>> for DispatchError {
    fn from(_error: Error<T>) -> Self {
        DispatchError::Other("ZkSnark verification error")
    }
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
    
    fn parse_verification_key(vk_bytes: &[u8]) -> Result<VerifyingKey<Bls12_381>, &'static str>;
    
    fn verify_groth16(
        pi_a: &[u8],
        pi_b: &[u8],
        pi_c: &[u8],
        public_inputs: &[u8],
        vk_bytes: &[u8]
    ) -> Result<bool, DispatchError>;
}

impl<T: Config> ZkSnarkVerifier for Pallet<T> {
    fn verify_proof(
        proof: &Proof,
        root: &[u8],
        nullifier_hash: &[u8],
        commitment: &[u8],
    ) -> Result<bool, DispatchError> {
        let vk_bytes = include_bytes!(
            concat!(env!("OUT_DIR"), "/zksnark/verification_key.json")
        );
        
        let mut public_inputs = Vec::new();
        public_inputs.extend_from_slice(root);
        public_inputs.extend_from_slice(nullifier_hash);
        public_inputs.extend_from_slice(commitment);

        Self::verify_groth16(
            &proof.a,
            &proof.b,
            &proof.c,
            &public_inputs,
            vk_bytes
        )
    }

    fn verify_groth16(
        pi_a: &[u8],
        pi_b: &[u8],
        pi_c: &[u8],
        public_inputs: &[u8],
        vk_bytes: &[u8]
    ) -> Result<bool, DispatchError> {
        let a_points = Self::decode_g1_point(pi_a)
            .map_err(|_| Error::<T>::InvalidProof)?;
        let b_points = Self::decode_g2_point(pi_b)
            .map_err(|_| Error::<T>::InvalidProof)?;
        let c_points = Self::decode_g1_point(pi_c)
            .map_err(|_| Error::<T>::InvalidProof)?;
    
        let vk = Self::parse_verification_key(vk_bytes)
            .map_err(|_| Error::<T>::InvalidProof)?;
    
        let pvk = prepare_verifying_key(&vk);
    
        let mut inputs = Vec::new();
        for chunk in public_inputs.chunks(32) {
            let mut bytes = [0u8; 32];
            bytes.copy_from_slice(chunk);
            let fr = Fr::from_be_bytes_mod_order(&bytes);
            inputs.push(fr);
        }
    
        let proof = ark_groth16::Proof {
            a: a_points,
            b: b_points,
            c: c_points,
        };
    
        let verified = Groth16::<Bls12_381>::verify_with_processed_vk(
            &pvk, 
            &inputs, 
            &proof
        ).map_err(|_| Error::<T>::ProofVerificationFailed)?;
        
        Ok(verified)
    }

    fn decode_g1_point(bytes: &[u8]) -> Result<G1Affine, &'static str> {
        if bytes.len() != 64 {  
            return Err("Invalid G1 point length");
        }
        
        let mut x_bytes = [0u8; 32];
        let mut y_bytes = [0u8; 32];
        x_bytes.copy_from_slice(&bytes[..32]);
        y_bytes.copy_from_slice(&bytes[32..]);
        
        let x = Fq::from_be_bytes_mod_order(&x_bytes);
        let y = Fq::from_be_bytes_mod_order(&y_bytes);
        
        Ok(G1Affine::new(x, y))
    }

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
        
        let x_c0 = Fq::from_be_bytes_mod_order(&x_c0_bytes);
        let x_c1 = Fq::from_be_bytes_mod_order(&x_c1_bytes);
        let y_c0 = Fq::from_be_bytes_mod_order(&y_c0_bytes);
        let y_c1 = Fq::from_be_bytes_mod_order(&y_c1_bytes);
        
        let x = Fq2::new(x_c0, x_c1);
        let y = Fq2::new(y_c0, y_c1);
        
        Ok(G2Affine::new(x, y))
    }

    fn parse_verification_key(vk_bytes: &[u8]) -> Result<VerifyingKey<Bls12_381>, &'static str> {
        let vk: serde_json::Value = serde_json::from_slice(vk_bytes)
            .map_err(|_| "Failed to parse verification key")?;
        
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

        let ic = vk["IC"]
            .as_array()
            .ok_or("Invalid IC in vk")?;

        let mut gamma_abc_g1 = Vec::new();
        for point in ic {
            let point_hex = point.as_str().ok_or("Invalid IC point")?;
            let point_bytes = hex::decode(point_hex).map_err(|_| "Invalid IC point hex")?;
            let g1_point = Self::decode_g1_point(&point_bytes)?;
            gamma_abc_g1.push(g1_point);
        }

        Ok(VerifyingKey {
            alpha_g1: Self::decode_g1_point(&hex::decode(alpha[0].as_str().ok_or("Invalid alpha hex")?).map_err(|_| "Invalid alpha encoding")?)?,
            beta_g2: Self::decode_g2_point(&hex::decode(beta[0].as_str().ok_or("Invalid beta hex")?).map_err(|_| "Invalid beta encoding")?)?,
            gamma_g2: Self::decode_g2_point(&hex::decode(gamma[0].as_str().ok_or("Invalid gamma hex")?).map_err(|_| "Invalid gamma encoding")?)?,
            delta_g2: Self::decode_g2_point(&hex::decode(delta[0].as_str().ok_or("Invalid delta hex")?).map_err(|_| "Invalid delta encoding")?)?,
            gamma_abc_g1,
        })
    }
}
