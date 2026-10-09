use anchor_lang::prelude::*;


#[error_code]
pub enum ErrorCode {
    // tutorial errors
    #[msg("Only the counter authority can update this counter")]
    Unauthorized,

    #[msg("Counter has reached the maximum value")]
    CounterOverflow,

    // zk-spot-shield errors
    #[msg("The zero-knowledge proof is invalid.")]
    InvalidProof,

    #[msg("The nullifier has already been used.")]
    NullifierAlreadyUsed,

    #[msg("Failed to deserialize zero-copy account data.")]
    ZeroCopyDeserializationError,

    #[msg("The Merkle root was not found.")]
    MerkleRootNotFound,

    #[msg("Proof bytes are required for settle.")]
    EmptyProof,

    #[msg("Journal bytes are required for settle.")]
    EmptyJournal,

    #[msg("Failed to parse the proof journal public values.")]
    InvalidJournal,

    #[msg("Journal mint does not match vault mint_a or mint_b.")]
    UnknownVaultMint,

    #[msg("Vault reserve overflow.")]
    VaultReserveOverflow,

    #[msg("Vault reserve underflow.")]
    VaultReserveUnderflow,

    #[msg("Swap amount must be non-zero.")]
    ZeroSwapAmount,

    #[msg("Token account mint or authority does not match the vault.")]
    InvalidTokenAccount,

    #[msg("Protocol is paused; settle is disabled.")]
    ProtocolPaused,
}