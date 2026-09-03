#![no_main]
sp1_zkvm::entrypoint!(main);

use zk_circuit_io::{
    compute_leaf, compute_nullifier, verify_merkle_path, PrivateInputs, PublicOutputs,
};

pub fn main() {
    let inputs: PrivateInputs = sp1_zkvm::io::read();

    assert!(
        inputs.balance >= inputs.requested_swap_amount,
        "Solvency violation: Insufficient balance"
    );

    let leaf = compute_leaf(&inputs.secret, &inputs.user_address, inputs.balance);

    let computed_root = verify_merkle_path(leaf, &inputs.merkle_path);
    assert_eq!(
        computed_root, inputs.expected_root,
        "Merkle root mismatch: Computed path does not match state root"
    );

    let nullifier = compute_nullifier(&inputs.secret, &leaf, &inputs.asset_id_mint);

    let outputs = PublicOutputs {
        requested_swap_amount: inputs.requested_swap_amount,
        asset_id_mint: inputs.asset_id_mint,
        nullifier,
        merkle_root: computed_root,
    };

    sp1_zkvm::io::commit(&outputs);
}
