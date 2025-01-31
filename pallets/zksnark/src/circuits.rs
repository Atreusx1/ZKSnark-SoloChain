use halo2_proofs::{
    arithmetic::Field,
    circuit::{Layouter, SimpleFloorPlanner, Value},
    plonk::{Circuit, Column, ConstraintSystem, Error, Instance},
    poly::Rotation,
};
use crate::halo2::poseidon::{Hash, PoseidonChip};
use pasta_curves::pallas;

#[derive(Clone)]
pub struct BuildTransactionCircuit;

impl TransactionCircuit {
    pub fn default() -> Self {
        BuildTransactionCircuit
    }
}

impl Circuit<pallas::Base> for BuildTransactionCircuit {
    type Config = ();
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self
    }

    fn configure(_meta: &mut ConstraintSystem<pallas::Base>) -> Self::Config {
        ()
    }

    fn synthesize(
        &self,
        _config: Self::Config,
        _layouter: impl Layouter<pallas::Base>,
    ) -> Result<(), Error> {
        Ok(())
    }
}

#[derive(Clone)]
pub struct TransactionCircuit {
    pub sender: Value<pallas::Base>,
    pub receiver: Value<pallas::Base>,
    pub amount: Value<pallas::Base>,
    pub nullifier: Value<pallas::Base>,
    pub secret: Value<pallas::Base>,
    pub merkle_path_elements: Value<Vec<pallas::Base>>,
    pub merkle_path_indices: Value<Vec<bool>>,
}

#[derive(Clone, Debug)]
pub struct TransactionCircuitConfig {
    commitment: Column<Instance>,
    nullifier_hash: Column<Instance>,
    merkle_root: Column<Instance>,
    sender: Column<Advice>,
    receiver: Column<Advice>,
    amount: Column<Advice>,
    nullifier: Column<Advice>,
    secret: Column<Advice>,
}

impl Circuit<pallas::Base> for TransactionCircuit {
    type Config = TransactionCircuitConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self {
            sender: Value::default(),
            receiver: Value::default(),
            amount: Value::default(),
            nullifier: Value::default(),
            secret: Value::default(),
            merkle_path_elements: Value::default(),
            merkle_path_indices: Value::default(),
        }
    }

    fn configure(meta: &mut ConstraintSystem<pallas::Base>) -> Self::Config {
        let commitment = meta.instance_column();
        let nullifier_hash = meta.instance_column();
        let merkle_root = meta.instance_column();
        let sender = meta.advice_column();
        let receiver = meta.advice_column();
        let amount = meta.advice_column();
        let nullifier = meta.advice_column();
        let secret = meta.advice_column();

        meta.enable_equality(sender);
        meta.enable_equality(receiver);
        meta.enable_equality(amount);
        meta.enable_equality(nullifier);
        meta.enable_equality(secret);
        meta.enable_equality(commitment);
        meta.enable_equality(nullifier_hash);
        meta.enable_equality(merkle_root);

        TransactionCircuitConfig {
            commitment,
            nullifier_hash,
            merkle_root,
            sender,
            receiver,
            amount,
            nullifier,
            secret,
        }
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<pallas::Base>,
    ) -> Result<(), Error> {
        let poseidon_chip = PoseidonChip::construct(config.clone());

        // Assign private inputs
        let sender_val = layouter.assign_region(
            || "Sender",
            |mut region| {
                region.assign_advice(
                    || "Sender",
                    config.sender,
                    0,
                    || self.sender.clone(),
                )
            },
        )?;

        let receiver_val = layouter.assign_region(
            || "Receiver",
            |mut region| {
                region.assign_advice(
                    || "Receiver",
                    config.receiver,
                    0,
                    || self.receiver.clone(),
                )
            },
        )?;

        let amount_val = layouter.assign_region(
            || "Amount",
            |mut region| {
                region.assign_advice(
                    || "Amount",
                    config.amount,
                    0,
                    || self.amount.clone(),
                )
            },
        )?;

        let nullifier_val = layouter.assign_region(
            || "Nullifier",
            |mut region| {
                region.assign_advice(
                    || "Nullifier",
                    config.nullifier,
                    0,
                    || self.nullifier.clone(),
                )
            },
        )?;

        let secret_val = layouter.assign_region(
            || "Secret",
            |mut region| {
                region.assign_advice(
                    || "Secret",
                    config.secret,
                    0,
                    || self.secret.clone(),
                )
            },
        )?;

        // Compute commitment
        let commitment_chip = poseidon_chip.hash_three(
            sender_val.value().cloned(),
            receiver_val.value().cloned(),
            amount_val.value().cloned(),
        )?;

        // Compute nullifier hash
        let nullifier_chip = poseidon_chip.hash_two(
            nullifier_val.value().cloned(),
            secret_val.value().cloned(),
        )?;

        // Verify Merkle proof
        let merkle_path_elements = self.merkle_path_elements.as_ref().unwrap();
        let merkle_path_indices = self.merkle_path_indices.as_ref().unwrap();

        let mut current_hash = commitment_chip;
        for (i, (&element, &is_right)) in merkle_path_elements
            .iter()
            .cloned()
            .zip(merkle_path_indices)
            .enumerate()
        {
            let element_val = layouter.assign_region(
                || format!("Merkle path element {}", i),
                |mut region| {
                    region.assign_advice(
                        || "Element",
                        config.sender, // Reuse sender column for elements
                        i,
                        || Value::known(element),
                    )
                },
            )?;

            let (left, right) = if is_right {
                (element_val, current_hash)
            } else {
                (current_hash, element_val)
            };

            current_hash = poseidon_chip.hash_two(left, right)?;
        }

        // Constrain public inputs
        layouter.constrain_instance(
            commitment_chip,
            config.commitment,
            0,
        )?;
        layouter.constrain_instance(
            nullifier_chip,
            config.nullifier_hash,
            0,
        )?;
        layouter.constrain_instance(
            current_hash,
            config.merkle_root,
            0,
        )?;

        Ok(())
    }
}