use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Counter {
    pub count: u64,
    pub authority: Pubkey,
}

#[account]
#[derive(InitSpace)]
pub struct GlobalConfig {
    pub authority: Pubkey,
    pub vkey_hash: [u8; 32],
    pub pause_flag: bool,
}

#[account(zero_copy)]
#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct VaultState {
    pub authority: Pubkey,       // 32 bytes
    pub mint_a: Pubkey,          // 32 bytes
    pub mint_b: Pubkey,          // 32 bytes
    pub reserve_a: u64,          // 8 bytes
    pub reserve_b: u64,          // 8 bytes
    pub bump: u8,                // 1 byte
    pub _padding: [u8; 7],       // 7 bytes (explicit padding to align to 8-byte boundary)
}

// Static alignment and size checks compile-time
const _: () = {
    // Total size: 32 + 32 + 32 + 8 + 8 + 1 + 7 = 120 bytes
    assert!(size_of::<VaultState>() == 120);
    assert!(size_of::<VaultState>() % 8 == 0);
};