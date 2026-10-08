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
}