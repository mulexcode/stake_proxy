#![allow(clippy::result_large_err)]
use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use anchor_lang::solana_program::stake;
use crate::stake_info::StakeInfo;
use crate::{Stake, STAKE_INFO_SEED, DELEGATE_AUTHORITY_SEED};

#[derive(Accounts)]
pub struct DeactivateAccount<'info> {
    #[account(
        seeds = [
            STAKE_INFO_SEED.as_bytes(),
            sys_stake_state.key().as_ref()
        ],
        bump
    )]
    pub stake_info: Account<'info, StakeInfo>,
    
    /// CHECK: check its ownership
    #[account(seeds = [DELEGATE_AUTHORITY_SEED.as_bytes()], bump)]
    pub delegate_authority: UncheckedAccount<'info>,

    /// CHECK: check its ownership
    #[account(mut, owner=stake::program::ID)]
    pub sys_stake_state: UncheckedAccount<'info>, // system stake account

    #[account(mut,
        address = stake_info.staker_pubkey,
    )]
    pub authority: Signer<'info>,
    pub clock: Sysvar<'info, Clock>,
    pub stake: Program<'info, Stake>,
}

pub fn handler(ctx: Context<DeactivateAccount>) -> Result<()> {
    deactivate(
        &ctx.accounts.delegate_authority,
        &ctx.accounts.sys_stake_state,
        &ctx.accounts.clock,
        ctx.bumps.delegate_authority,
    )
}

fn deactivate<'info>(
    delegate_auth: &UncheckedAccount<'info>,
    sys_stake_state: &UncheckedAccount<'info>,
    clock: &Sysvar<'info, Clock>,
    delegate_auth_bump: u8,
) -> Result<()> {
    let delegate_auth_seeds: &[&[&[u8]]] = &[&[DELEGATE_AUTHORITY_SEED.as_bytes(), &[delegate_auth_bump]]];

    let ix = stake::instruction::deactivate_stake(&sys_stake_state.key(), &delegate_auth.key());

    solana_program::program::invoke_signed(
        &ix,
        &[
            sys_stake_state.to_account_info(),
            clock.to_account_info(),
            delegate_auth.to_account_info(),
        ],
        delegate_auth_seeds,
    ).map_err(Into::into)
}