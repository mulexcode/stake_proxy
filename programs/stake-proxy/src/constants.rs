use anchor_lang::prelude::*;

#[cfg(feature="localnet")]
pub const STAKE_TOKEN_MINT: Pubkey = pubkey!("2vSxuEFrcRCrj95jGnQ5kfSPMN2JyqXhfA22iaiivR7f");

#[cfg(not(feature="localnet"))]
pub const STAKE_TOKEN_MINT: Pubkey = pubkey!("2vSxuEFrcRCrj95jGnQ5kfSPMN2JyqXhfA22iaiivR7f"); // [213,50,144,69,65,59,187,93,206,199,77,226,167,39,149,36,254,45,245,78,48,69,138,238,83,154,149,142,144,116,208,229,28,140,211,193,128,241,114,45,179,80,71,32,158,243,51,166,90,93,254,59,46,156,238,126,126,142,21,9,97,38,161,26]

#[constant]
pub const STAKE_STATE_SEED: &str = "stake_state";

#[constant]
pub const STAKE_INFO_SEED: &str = "stake_info";

#[constant]
pub const NATIVE_VAULT_SEED: &str = "native_vault";

#[constant]
pub const STAKE_CONFIG: Pubkey = pubkey!("StakeConfig11111111111111111111111111111111");


