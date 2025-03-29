use anchor_lang::prelude::*;
use crate::state::{SystemConfig, ChainConfig};
use crate::constants::*;

#[derive(Accounts)]
#[instruction(chain_id: u64)]
pub struct EnableChainAccount<'info> {
    #[account(
        seeds = [SYSTEM_CONFIG_SEED.as_bytes()],
        bump
    )]
    pub config: Account<'info, SystemConfig>,
    
    #[account(
        init,
        space = 8 + ChainConfig::LEN,
        payer=payer,
        seeds = [CHAIN_CONFIG_SEED.as_bytes(), chain_id.to_le_bytes().as_slice()],
        bump
    )]
    pub chain_config: Account<'info, ChainConfig>,
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(address=config.manager)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<EnableChainAccount>, chain_id: u64) -> Result<()> {
    ctx.accounts.chain_config.initialize(chain_id);
    Ok(())
}