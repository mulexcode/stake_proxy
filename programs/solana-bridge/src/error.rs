use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Signature verification failed.")]
    SigVerificationFailed,
    #[msg("Missing secp256k1 instruction")]
    MissingSecp256k1Instruction,
    #[msg("Invalid payout nonce")]
    InvalidPayoutNonce,
    #[msg("Signature verification failed2.")]
    SigVerificationFailed2,
    #[msg("Signature verification failed3.")]
    SigVerificationFailed3,
}