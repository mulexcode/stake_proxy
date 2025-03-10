use crate::borsh;
use anchor_lang::*;
use anchor_lang::prelude::Pubkey;

#[account]
pub struct StakeInfo {
    pub amount: u64,
    pub staker_pubkey: Pubkey,
    pub withdrawer_pubkey: Pubkey,
}