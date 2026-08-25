use anchor_lang::prelude::*;

#[constant]
pub const COUNTER_SEED: &[u8] = b"counter";

#[constant]
pub const HELLO_WORLD_LAMPORTS: u64 = 1;

#[constant]
pub const MAX_COUNT: u64 = 10;

#[constant]
pub const GLOBAL_CONFIG_SEED: &[u8] = b"global-config";

#[constant]
pub const VKEY_HASH: [u8; 32] = [0u8; 32];