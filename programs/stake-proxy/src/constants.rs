use anchor_lang::prelude::*;

pub const STAKE_TOKEN_MINT: Pubkey = pubkey!("StakeToken111111111111111111111111111111111");

pub const NATIVE_TOKEN_VAULT: Pubkey = pubkey!("StakeVau1t111111111111111111111111111111111");

#[constant]
pub const STAKE_STATE_SEED: &str = "stake_state";

#[constant]
pub const STAKE_INFO_SEED: &str = "stake_info";

#[constant]
pub const NATIVE_VAULT_SEED: &str = "native_vault";


