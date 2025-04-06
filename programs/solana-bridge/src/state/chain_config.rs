use crate::borsh;
use anchor_lang::*;
use anchor_lang::prelude::Pubkey;


#[account]
pub struct ChainConfig {
    pub enabled: bool,
    pub chain_id: u64,
    pub cash_out_nonce: u64,
    pub payout_nonce: u64,
}

impl ChainConfig {
    pub const LEN: usize = 1 + 8 + 8 + 8;
    
    pub fn initialize(&mut self, chain_id: u64) {
        self.chain_id = chain_id;
        self.enabled = true;
        self.payout_nonce = 0;
        self.cash_out_nonce = 0;
    }
    
    pub fn increase_payout_nonce(&mut self) {
        self.payout_nonce += 1;
    }
    
    pub fn increase_cash_out_nonce(&mut self) {
        self.cash_out_nonce += 1;
    }
}