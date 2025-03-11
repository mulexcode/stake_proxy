use anchor_lang::context::{Context};
use anchor_lang::system_program::System;
use crate::constants::{NATIVE_VAULT_SEED};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct Initialize<'info> {
    
    /// CHECK: stake::initialize() 
    #[account(
        init, payer=payer, space=0,
        seeds = [NATIVE_VAULT_SEED.as_bytes()],
        bump
    )]
    pub native_vault: UncheckedAccount<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Initialize>) -> Result<()> {
    Ok(())
}