use anchor_lang::prelude::*;

#[constant]
pub const GLOBAL_CONFIG_SEED: &[u8] = b"global-config";

/// SP1 `vk.bytes32()` for the frozen happy-path guest ELF (Day 15).
/// Source: `zk-circuit/fixtures/happy/vkey.bytes32.txt` / `notes/proving.md`.
/// Rebuild guest → re-prove → update this array, the fixture, and the notes.
#[constant]
pub const VKEY_HASH: [u8; 32] = [
    0x00, 0xb3, 0xa1, 0x5c, 0xe4, 0xc0, 0xea, 0x94, //
    0xe3, 0xb0, 0x26, 0x74, 0x73, 0xc6, 0xb7, 0x54, //
    0x3a, 0x80, 0xc7, 0x2d, 0x20, 0x9b, 0x2c, 0x94, //
    0x7f, 0x71, 0x88, 0x6c, 0x6a, 0x57, 0x35, 0xd7,
];

#[constant]
pub const SPOT_VAULT_SEED: &[u8] = b"spot_vault";

#[constant]
pub const NULLIFIER_SEED_PREFIX: &[u8] = b"nullifier";

#[constant]
pub const CLEAN_FUNDS_ROOT_SEED_PREFIX: &[u8] = b"clean_funds_root";



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pda_seed_prefixes_are_unique() {
        let seeds = [
            GLOBAL_CONFIG_SEED,
            SPOT_VAULT_SEED,
            NULLIFIER_SEED_PREFIX,
            CLEAN_FUNDS_ROOT_SEED_PREFIX,
        ];

        for i in 0..seeds.len() {
            for j in (i + 1)..seeds.len() {
                assert_ne!(seeds[i], seeds[j]);
            }
        }
    }

    #[test]
    fn vkey_hash_matches_day12_happy_fixture() {
        assert_ne!(VKEY_HASH, [0u8; 32]);
        assert_eq!(VKEY_HASH[0], 0x00);
        assert_eq!(VKEY_HASH[31], 0xd7);
        // Full fingerprint: notes/proving.md + zk-circuit/fixtures/happy/vkey.bytes32.txt
        assert_eq!(
            VKEY_HASH,
            [
                0x00, 0xb3, 0xa1, 0x5c, 0xe4, 0xc0, 0xea, 0x94, 0xe3, 0xb0, 0x26, 0x74, 0x73,
                0xc6, 0xb7, 0x54, 0x3a, 0x80, 0xc7, 0x2d, 0x20, 0x9b, 0x2c, 0x94, 0x7f, 0x71,
                0x88, 0x6c, 0x6a, 0x57, 0x35, 0xd7,
            ]
        );
    }
}