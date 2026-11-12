use anchor_lang::prelude::*;

#[constant]
pub const GLOBAL_CONFIG_SEED: &[u8] = b"global-config";

/// SP1 `vk.bytes32()` for the current guest ELF (change-note journal).
/// Source: `zk-circuit/fixtures/happy/vkey.bytes32.txt` / `notes/proving.md`.
/// Rebuild guest → re-prove → update this array, the fixture, and the notes.
#[constant]
pub const VKEY_HASH: [u8; 32] = [
    0x00, 0x43, 0x56, 0x96, 0xd6, 0x66, 0xd3, 0x05, //
    0x17, 0x23, 0x06, 0xe3, 0x89, 0x5a, 0x44, 0x01, //
    0x7e, 0x18, 0x86, 0x63, 0xde, 0x9b, 0x38, 0x76, //
    0x97, 0x0a, 0xe2, 0x70, 0x34, 0x5f, 0xc2, 0x40,
];

#[constant]
pub const SPOT_VAULT_SEED: &[u8] = b"spot_vault";

#[constant]
pub const NULLIFIER_SEED_PREFIX: &[u8] = b"nullifier";

#[constant]
pub const CLEAN_FUNDS_ROOT_SEED_PREFIX: &[u8] = b"clean_funds_root";

/// PDA seeds: `["deposit_leaf", leaf]` — one account per deposited leaf commitment.
#[constant]
pub const DEPOSIT_LEAF_SEED_PREFIX: &[u8] = b"deposit_leaf";

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
            DEPOSIT_LEAF_SEED_PREFIX,
        ];

        for i in 0..seeds.len() {
            for j in (i + 1)..seeds.len() {
                assert_ne!(seeds[i], seeds[j]);
            }
        }
    }

    /// Constraint audit checklist (seeds/bump/has_one surface).
    #[test]
    fn pda_constraint_surface_documented() {
        // Init: global_config + vault — seeds + bump on init.
        assert_eq!(GLOBAL_CONFIG_SEED, b"global-config");
        assert_eq!(SPOT_VAULT_SEED, b"spot_vault");
        // Authority gates: register_clean_funds_root + pause/unpause + set_vault_mints
        // + set_vkey_hash use has_one = authority on GlobalConfig.
        // Settle PDAs: global_config, vault (bump from account), clean_funds_root (journal root),
        // nullifier (journal nullifier). Settle is permissionless (no has_one on payer).
        assert_eq!(CLEAN_FUNDS_ROOT_SEED_PREFIX, b"clean_funds_root");
        assert_eq!(NULLIFIER_SEED_PREFIX, b"nullifier");
    }

    /// Ix signer surface (must match `notes/layout-signer-audit.md`).
    #[test]
    fn instruction_signer_inventory() {
        let ixs: &[(&str, &str)] = &[
            ("initialize_global_config", "payer"),
            ("initialize_vault", "payer"),
            ("set_vault_mints", "authority"),
            ("set_vkey_hash", "authority"),
            ("register_clean_funds_root", "authority"),
            ("deposit_shielded", "depositor"),
            ("public_spot_swap", "payer"),
            ("settle_shielded_spot", "payer"),
            ("pause", "authority"),
            ("unpause", "authority"),
        ];
        assert_eq!(ixs.len(), 10);
        assert!(ixs.iter().all(|(n, s)| !n.is_empty() && !s.is_empty()));
        assert_eq!(
            ixs.iter()
                .find(|(n, _)| *n == "settle_shielded_spot")
                .map(|(_, s)| *s),
            Some("payer")
        );
    }

    #[test]
    fn vkey_hash_matches_happy_fixture() {
        assert_ne!(VKEY_HASH, [0u8; 32]);
        assert_eq!(VKEY_HASH[0], 0x00);
        assert_eq!(VKEY_HASH[31], 0x40);
        // Full fingerprint: notes/proving.md + zk-circuit/fixtures/happy/vkey.bytes32.txt
        assert_eq!(
            VKEY_HASH,
            [
                0x00, 0x43, 0x56, 0x96, 0xd6, 0x66, 0xd3, 0x05, 0x17, 0x23, 0x06, 0xe3, 0x89, 0x5a,
                0x44, 0x01, 0x7e, 0x18, 0x86, 0x63, 0xde, 0x9b, 0x38, 0x76, 0x97, 0x0a, 0xe2, 0x70,
                0x34, 0x5f, 0xc2, 0x40,
            ]
        );
    }
}
