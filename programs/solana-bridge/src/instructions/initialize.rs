use anchor_lang::__private::base64::engine::Config;
use anchor_lang::Accounts;
use anchor_lang::prelude::*;
use crate::state::SystemConfig;
use crate::constants::*;

#[derive(Accounts)]
pub struct InitializeAccount<'info> {
    
    #[account(
        init,
        space = 8 + SystemConfig::LEN,
        payer=payer,
        seeds = [SYSTEM_CONFIG_SEED.as_bytes()],
        bump
    )]
    pub config: Account<'info, SystemConfig>,

    /// CHECK: The stake pool account
    #[account(
        init,
        payer = payer,
        space = 0,
        seeds = [NATIVE_TOKEN_VAULT_SEED.as_bytes()],
        bump
    )]
    pub native_token_vault: AccountInfo<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(ctx: Context<InitializeAccount>, chain_id: u64, manager: Pubkey, secp256k1_manager: [u8; 20]) -> Result<()> {
    ctx.accounts.config.initialize(chain_id, manager, secp256k1_manager);
    Ok(())
}