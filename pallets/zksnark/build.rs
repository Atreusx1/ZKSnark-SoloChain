use halo2_proofs::{
    circuit::{Layouter, SimpleFloorPlanner},
    plonk::{keygen_vk, VerifyingKey},
    poly::commitment::Params,
    pasta::pallas,
};
use std::fs;
use base64::engine::general_purpose;
use serde_json;

struct BuildTransactionCircuit;

impl halo2_proofs::plonk::Circuit<pallas::Base> for BuildTransactionCircuit {
    type Config = ();
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self
    }

    fn configure(_meta: &mut halo2_proofs::plonk::ConstraintSystem<pallas::Base>) -> Self::Config {
        ()
    }

    fn synthesize(
        &self,
        _config: Self::Config,
        _layouter: impl Layouter<pallas::Base>,
    ) -> Result<(), halo2_proofs::plonk::Error> {
        Ok(())
    }
}

fn main() {
    let circuit = BuildTransactionCircuit;
    let params: Params<pallas::Affine> = Params::new(9);
    let vk = keygen_vk(&params, &circuit).expect("Keygen VK failed");

    // Serialize the verification key to bytes
    let mut vk_bytes = Vec::new();
    vk.write(&mut vk_bytes).expect("Failed to serialize VK");

    // Save the binary data to a file
    fs::write("src/verification_key.bin", vk_bytes).expect("Write verification key failed");

    // Encode the binary data as Base64 and save it in a JSON file
    let verification_key_base64 = general_purpose::STANDARD.encode(vk_bytes);
    let verification_key_json = serde_json::json!({
        "vk": verification_key_base64
    });
    fs::write("src/verification_key.json", verification_key_json.to_string())
        .expect("Write verification key JSON failed");
}