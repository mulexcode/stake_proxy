use anchor_lang::prelude::*;

#[cfg(feature="localnet")]
pub const STAKE_TOKEN_MINT: Pubkey = pubkey!("So11111111111111111111111111111111111111112");

#[cfg(not(feature="localnet"))]
pub const STAKE_TOKEN_MINT: Pubkey = pubkey!("StakeToken111111111111111111111111111111111");

#[constant]
pub const STAKE_STATE_SEED: &str = "stake_state";

#[constant]
pub const STAKE_INFO_SEED: &str = "stake_info";

#[constant]
pub const NATIVE_VAULT_SEED: &str = "native_vault";


