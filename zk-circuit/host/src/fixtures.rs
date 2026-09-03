use zk_circuit_io::{compute_leaf, hash_nodes, verify_merkle_path, PrivateInputs};
pub fn happy_path_inputs() -> PrivateInputs {
    let secret = [1u8; 32];
    let user_address = [2u8; 32];
    let asset_id_mint = [3u8; 32];
    let balance = 1_000;
    let requested_swap_amount = 100; // <= balance
    let leaf = compute_leaf(&secret, &user_address, balance);
    // empty[h] = hash of a 2^h-wide all-zero subtree
    let mut empty = [[0u8; 32]; 20];
    empty[0] = [0u8; 32];
    for h in 1..20 {
        empty[h] = hash_nodes(&empty[h - 1], &empty[h - 1]);
    }
    let mut merkle_path = [([0u8; 32], false); 20];
    for h in 0..20 {
        merkle_path[h] = (empty[h], true); // index 0: sibling always on the right
    }
    let expected_root = verify_merkle_path(leaf, &merkle_path);
    PrivateInputs {
        secret,
        user_address,
        merkle_path,
        balance,
        requested_swap_amount,
        asset_id_mint,
        expected_root,
    }
}