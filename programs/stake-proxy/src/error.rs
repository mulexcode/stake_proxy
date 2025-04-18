use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Custom error message")]
    CustomError,
    #[msg("Insufficient funds")]
    InsufficientFundsForTransaction,
    #[msg("Need more stake token")]
    NeedMoreStakeToken,
    #[msg("Stake token mint mismatch")]
    StakeTokenMintMismatch,
    #[msg("Stake amount too small")]
    StakeAmountTooSmall,
    #[msg("Stake amount too big")]
    StakeAmountTooBig,
}
