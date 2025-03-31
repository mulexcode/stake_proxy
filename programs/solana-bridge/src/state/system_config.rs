use anchor_lang::account;
use anchor_lang::prelude::*;

#[account]
pub struct SystemConfig {
    pub manager: Pubkey,

    pub secp256k1_manager: [u8; 20],
    
    pub chain_id: u64,
}

impl SystemConfig {
    pub const LEN: usize = 32 + 20 + 8;
    
    pub fn initialize(&mut self, chain_id: u64, manager: Pubkey, secp256k1_manager: [u8; 20]) {
        self.secp256k1_manager = secp256k1_manager;
        self.manager = manager;
        self.chain_id = chain_id;
    }

    pub fn update_manager(&mut self, manager: Pubkey, secp256k1_manager: [u8; 20]) {
        self.secp256k1_manager = secp256k1_manager;
        self.manager = manager;
    }
}