use anchor_lang::account;
use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct TokenConfig {
    pub enabled: bool,
    #[max_len(10)]
    pub token_name: String,
    pub token_mint: Pubkey,
}

impl TokenConfig {
    pub fn initialize(&mut self, token_mint: Pubkey, token_name: String) {
        self.enabled = true;
        self.token_mint = token_mint;
        self.token_name = token_name;
    }
}