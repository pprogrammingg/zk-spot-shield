#![no_main]
sp1_zkvm::entrypoint!(main);

use zk_circuit_io::{public_outputs_from_private, CircuitReject, PrivateInputs};

pub fn main() {
    let inputs: PrivateInputs = sp1_zkvm::io::read();

    let outputs = match public_outputs_from_private(&inputs) {
        Ok(o) => o,
        Err(CircuitReject::Underfunded) => {
            panic!("Solvency violation: Insufficient balance")
        }
        Err(CircuitReject::MerkleRootMismatch) => {
            panic!("Merkle root mismatch: Computed path does not match state root")
        }
        Err(CircuitReject::Conservation) => {
            panic!("Conservation violation: change + spend != old balance")
        }
    };

    sp1_zkvm::io::commit(&outputs);
}
