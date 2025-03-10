use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use anchor_lang::solana_program::stake;
use anchor_lang::solana_program::stake::state::Lockup;
use crate::stake_info::StakeInfo;
use crate::STAKE_INFO_SEED;

#[derive(Accounts)]
pub struct DelegateStakeAccount {}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct DelegateStakeArgs {
    pub amount: u64,
}

pub fn handler(ctx: Context<DelegateStakeAccount>, args: DelegateStakeArgs) -> Result<()> {
    msg!("Greetings");
    Ok(())
}

fn delegate_stake<'info>(
    stake_info: &Account<'info, StakeInfo>,
    sys_stake_state: &UncheckedAccount<'info>,
    vote: &UncheckedAccount<'info>,
    clock: &Sysvar<'info, Clock>,
    stake_history: &Sysvar<'info, StakeHistory>,
    native_vault: &SystemAccount<'info>,
    stake_info_bump: u8,
    stake_amount: u64,
) -> Result<()> {
    let sys_stake_state_key = sys_stake_state.key();
    let ix = stake::instruction::delegate_stake(&sys_stake_state.key(), &stake_info.key(), &vote.key());
    let stake_info_seeds: &[&[&[u8]]] = &[&[STAKE_INFO_SEED.as_bytes(), sys_stake_state_key.as_ref(), &[stake_info_bump]]];
    solana_program::program::invoke_signed(
        &ix,
        &[
            sys_stake_state.to_account_info(),
            vote.to_account_info(),
            clock.to_account_info(),
            stake_history.to_account_info(),
            stake_info.to_account_info(),
        ],
        stake_info_seeds,
    ).map_err(Into::into)
}