use anchor_lang::context::{Context};
use anchor_lang::system_program::System;
use crate::constants::{NATIVE_VAULT_SEED};
use anchor_lang::prelude::*;
use anchor_lang::system_program;

#[derive(Accounts)]
pub struct Initialize<'info> {
    
    /// CHECK: stake::initialize() 
    #[account(
        mut,
        seeds = [NATIVE_VAULT_SEED.as_bytes()],
        bump
    )]
    pub native_vault: UncheckedAccount<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(ctx: Context<Initialize>) -> Result<()> {
    
    system_program::create_account(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::CreateAccount {
                from: ctx.accounts.payer.to_account_info(),
                to: ctx.accounts.native_vault.to_account_info(),
            },
        )
            .with_signer(&[&[NATIVE_VAULT_SEED.as_bytes(), &[ctx.bumps.native_vault]]]),
        ctx.accounts.rent.minimum_balance(0), // rent-free
        0,
        &ctx.accounts.system_program.key(),
    )
}