use anchor_lang::Accounts;
use anchor_lang::prelude::*;
use crate::{ChainConfig, SystemConfig};
use crate::constants::*;

#[derive(Accounts)]
#[instruction(chain_id: u64)]
pub struct UpdateNonceAccount<'info> {

    #[account(
        has_one=manager,
        seeds = [SYSTEM_CONFIG_SEED.as_bytes()],
        bump
    )]
    pub config: Account<'info, SystemConfig>,

    #[account(
        mut,
        seeds = [CHAIN_CONFIG_SEED.as_bytes(), chain_id.to_le_bytes().as_slice()],
        bump
    )]
    pub chain_config: Account<'info, ChainConfig>,

    #[account(mut)]
    pub manager: Signer<'info>,
}

pub fn handler(ctx: Context<UpdateNonceAccount>, chain_id: u64, nonce: u64, is_payout_nonce: bool) -> Result<()> {
    if is_payout_nonce {
        ctx.accounts.chain_config.payout_nonce = nonce;
    } else {
        ctx.accounts.chain_config.cash_out_nonce = nonce;
    }
    Ok(())
}