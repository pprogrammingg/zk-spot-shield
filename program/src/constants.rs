use anchor_lang::prelude::*;

#[constant]
pub const GLOBAL_CONFIG_SEED: &[u8] = b"global-config";

#[constant]
pub const VKEY_HASH: [u8; 32] = [0u8; 32];

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
            NULLIFIER_SEED_PREFIX,
            CLEAN_FUNDS_ROOT_SEED_PREFIX,
        ];

        for i in 0..seeds.len() {
            for j in (i + 1)..seeds.len() {
                assert_ne!(seeds[i], seeds[j]);
            }
        }
    }
}