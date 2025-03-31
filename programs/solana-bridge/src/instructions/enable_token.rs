use anchor_lang::__private::base64::engine::Config;
use anchor_lang::Accounts;
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use crate::state::{SystemConfig, TokenConfig};
use crate::constants::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

#[derive(Accounts)]
#[instruction(token_name: String)]
pub struct EnableTokenAccount<'info> {
    #[account(
        seeds = [SYSTEM_CONFIG_SEED.as_bytes()],
        bump
    )]
    pub config: Account<'info, SystemConfig>,
    #[account(
        init,
        space = 8 + TokenConfig::INIT_SPACE,
        payer=payer,
        seeds = [TOKEN_CONFIG_SEED.as_bytes(), token_name.as_bytes()],
        bump
    )]
    pub token_config: Account<'info, TokenConfig>,
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(address=config.manager)]
    pub authority: Signer<'info>,
    pub token_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<EnableTokenAccount>, token_name: String) -> Result<()> {
    ctx.accounts.token_config.initialize(ctx.accounts.token_mint.key(), token_name);
    Ok(())
}