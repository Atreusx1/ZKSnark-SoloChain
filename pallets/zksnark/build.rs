use halo2_proofs::{
    plonk::{VerificationKey, Circuit, keygen_vk},
    circuit::{Layouter, SimpleFloorPlanner},
    pasta::pallas::Base,
};
use pasta_curves::{pallas, vesta};
use serde::{Serialize};
use serde_json;
use std::fs;

#[derive(Serialize)]
struct VerifyingKeyWrapper {
    alpha_g1: pallas::Affine,
    beta_g2: vesta::Affine,
    gamma_g2: vesta::Affine,
    delta_g2: vesta::Affine,
    ic: Vec<pallas::Affine>,
}

impl From<VerificationKey<Base>> for VerifyingKeyWrapper {
    fn from(vk: VerificationKey<Base>) -> Self {
        VerifyingKeyWrapper {
            alpha_g1: vk.alpha_g1,
            beta_g2: vk.beta_g2,
            gamma_g2: vk.gamma_g2,
            delta_g2: vk.delta_g2,
            ic: vk.ic,
        }
    }
}

struct BuildTransactionCircuit;

impl Circuit<Base> for BuildTransactionCircuit {
    type Config = ();
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self
    }

    fn configure(_meta: &mut halo2_proofs::plonk::ConstraintSystem<Base>) -> Self::Config {
        ()
    }

    fn synthesize(
        &self,
        _config: Self::Config,
        _layouter: impl Layouter<Base>,
    ) -> Result<(), halo2_proofs::plonk::Error> {
        Ok(())
    }
}

fn main() {
    let circuit = BuildTransactionCircuit;
    let params = halo2_proofs::poly::commitment::Params::new(9);
    let vk = keygen_vk(&params, &circuit).unwrap();

    let vk_wrapper: VerifyingKeyWrapper = vk.into();
    let verification_key_json = serde_json::to_vec(&vk_wrapper).unwrap();
    fs::write("src/verification_key.json", verification_key_json).unwrap();
}