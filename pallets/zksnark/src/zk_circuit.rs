//pallets/zksnark/zk_circuit.rs

use ark_ff::Field;
use ark_bls12_381::{Bls12_381, Fr};
use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
use ark_r1cs_std::fields::fp::FpVar;
use std::ops::Add;

/// A zkSNARK circuit for private transactions
#[derive(Clone)]
pub struct TransactionCircuit {
    pub commitment: Option<Fr>,
    pub sender: Option<Fr>,
    pub receiver: Option<Fr>,
    pub amount: Option<Fr>,
}

impl TransactionCircuit {
    /// Constructor for TransactionCircuit
    pub fn new(
        commitment: Option<Fr>,
        sender: Option<Fr>,
        receiver: Option<Fr>,
        amount: Option<Fr>,
    ) -> Self {
        TransactionCircuit {
            commitment,
            sender,
            receiver,
            amount,
        }
    }
}

impl ConstraintSynthesizer<Fr> for TransactionCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        // Allocate private inputs
        let sender_var = FpVar::<Fr>::new_witness(cs.clone(), || {
            self.sender.ok_or(SynthesisError::AssignmentMissing)
        })?;

        let receiver_var = FpVar::<Fr>::new_witness(cs.clone(), || {
            self.receiver.ok_or(SynthesisError::AssignmentMissing)
        })?;

        let amount_var = FpVar::<Fr>::new_witness(cs.clone(), || {
            self.amount.ok_or(SynthesisError::AssignmentMissing)
        })?;

        // Compute commitment = sender + receiver + amount
        let commitment_computed = sender_var.clone() + receiver_var.clone() + amount_var.clone();


        let commitment_var = FpVar::<Fr>::new_input(cs.clone(), || {
            self.commitment.ok_or(SynthesisError::AssignmentMissing)
        })?;

        // Enforce the commitment constraint
        commitment_computed.enforce_equal(&commitment_var)?;

        Ok(())
    }
}

fn main() {
    // Example of creating a circuit with some dummy values
    let commitment = Some(Fr::from(100u64));
    let sender = Some(Fr::from(50u64));
    let receiver = Some(Fr::from(30u64));
    let amount = Some(Fr::from(20u64));

    let circuit = TransactionCircuit::new(commitment, sender, receiver, amount);

    // Create a constraint system reference
    let cs = ark_relations::r1cs::ConstraintSystem::<Fr>::new_ref();

    // Generate constraints
    circuit.generate_constraints(cs.clone()).unwrap();

    // Output the number of constraints for debugging
    println!("Number of constraints: {}", cs.num_constraints());
}
