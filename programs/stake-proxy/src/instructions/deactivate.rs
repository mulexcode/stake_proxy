#![allow(clippy::result_large_err)]
use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use anchor_lang::solana_program::stake;
use crate::stake_info::StakeInfo;
use crate::{Stake, STAKE_INFO_SEED};

#[derive(Accounts)]
pub struct DeactivateAccount<'info> {
    #[account(
        mut,
        seeds = [
            STAKE_INFO_SEED.as_bytes(),
            sys_stake_state.key().as_ref()
        ],
        bump
    )]
    pub stake_info: Account<'info, StakeInfo>,

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
        &ctx.accounts.stake_info,
        &ctx.accounts.sys_stake_state,
        &ctx.accounts.clock,
        ctx.bumps.stake_info,
    )
}

fn deactivate<'info>(
    stake_info: &Account<'info, StakeInfo>,
    sys_stake_state: &UncheckedAccount<'info>,
    clock: &Sysvar<'info, Clock>,
    stake_info_bump: u8,
) -> Result<()> {
    let sys_stake_state_key = sys_stake_state.key();
    let stake_info_seeds: &[&[&[u8]]] = &[&[STAKE_INFO_SEED.as_bytes(), sys_stake_state_key.as_ref(), &[stake_info_bump]]];

    let ix = stake::instruction::deactivate_stake(&sys_stake_state.key(), &stake_info.key());

    solana_program::program::invoke_signed(
        &ix,
        &[
            sys_stake_state.to_account_info(),
            clock.to_account_info(),
            stake_info.to_account_info(),
        ],
        stake_info_seeds,
    ).map_err(Into::into)
}