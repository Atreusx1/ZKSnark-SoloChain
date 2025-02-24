// src/bin/generate-params.rs
use halo2_proofs::plonk::{keygen_vk, Parameters, VerifyingKey};
use crate::circuits::TransactionCircuit;
use pasta_curves::pallas;

use serde_json;

fn main() {
    let k = 9;
    let params: Parameters<pallas::Affine> = Parameters::new(k).unwrap();

    let merkle_depth = 20;
    let dummy_merkle_path_elements = vec![pallas::Base::zero(); merkle_depth];
    let dummy_merkle_path_indices = vec![false; merkle_depth];

    let circuit = TransactionCircuit {
        sender: Value::known(pallas::Base::zero()),
        receiver: Value::known(pallas::Base::zero()),
        amount: Value::known(pallas::Base::zero()),
        nullifier: Value::known(pallas::Base::zero()),
        secret: Value::known(pallas::Base::zero()),
        merkle_path_elements: Value::known(dummy_merkle_path_elements),
        merkle_path_indices: Value::known(dummy_merkle_path_indices),
    };

    let vk = keygen_vk(&params, &circuit).unwrap();

    let vk_wrapper = VerifyingKeyWrapper {
        alpha_g1: vk.alpha_g1,
        beta_g2: vk.beta_g2,
        gamma_g2: vk.gamma_g2,
        delta_g2: vk.delta_g2,
        ic: vk.ic,
    };

    fs::write("verification_key.json", serde_json::to_string_pretty(&vk_wrapper).unwrap()).unwrap();
}