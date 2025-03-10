pub mod initialize_account;
pub mod delegate_stake;

use anchor_lang::prelude::{Account, AccountInfo, Clock, Rent, StakeHistory, SystemAccount, Sysvar, UncheckedAccount};
use anchor_lang::error::Error;
use anchor_lang::solana_program::stake;
use anchor_lang::{solana_program, Key, ToAccountInfo};
pub use initialize_account::*;
pub use delegate_stake::*;
use crate::error::ErrorCode::{InsufficientFundsForTransaction, NeedMoreStakeToken};
use crate::instructions;
use crate::stake_info::StakeInfo;

fn transfer_lamports(
    from_account: &AccountInfo,
    to_account: &AccountInfo,
    amount_of_lamports: u64,
) -> anchor_lang::Result<()> {
    if **from_account.try_borrow_lamports()? < amount_of_lamports {
        return Err(Error::from(InsufficientFundsForTransaction));
    }
    // Debit from_account and credit to_account
    **from_account.try_borrow_mut_lamports()? -= amount_of_lamports;
    **to_account.try_borrow_mut_lamports()? += amount_of_lamports;
    Ok(())
}

fn rebalance<'info>(
    stake_info: &Account<'info, StakeInfo>,
    sys_stake_state: &UncheckedAccount<'info>,
    rent: &Sysvar<'info, Rent>,
    native_vault: &SystemAccount<'info>,
    clock: &Sysvar<'info, Clock>,
    stake_history: &Sysvar<'info, StakeHistory>,
    stake_info_seeds: &[&[&[u8]]],
    stake_amount: u64
) -> anchor_lang::Result<()> {
    // check sol balance
    let min_balance = rent.minimum_balance(sys_stake_state.data_len());
    let expected_balance = min_balance + stake_amount;
    if expected_balance < sys_stake_state.lamports() {
        let need_to_withdraw = expected_balance - sys_stake_state.lamports();
        sys_stake_withdraw(stake_info, sys_stake_state, clock, stake_history, native_vault, stake_info_seeds, need_to_withdraw)?
    }

    let need_to_stake = expected_balance - sys_stake_state.lamports();
    if need_to_stake > 0 {
        transfer_lamports(&native_vault.to_account_info(), &sys_stake_state.to_account_info(), need_to_stake)?;
    }
    Ok(())
}

fn try_rebalance<'info>(
    sys_stake_state: &UncheckedAccount<'info>,
    rent: &Sysvar<'info, Rent>,
    native_vault: &SystemAccount<'info>,
    stake_amount: u64
) -> anchor_lang::Result<()> {
    // check sol balance
    let min_balance = rent.minimum_balance(sys_stake_state.data_len());
    let expected_balance = min_balance + stake_amount;
    if expected_balance < sys_stake_state.lamports() {
        return Err(Error::from(NeedMoreStakeToken));
    }

    let need_to_stake = expected_balance - sys_stake_state.lamports();
    if need_to_stake > 0 {
        transfer_lamports(&native_vault.to_account_info(), &sys_stake_state.to_account_info(), need_to_stake)?;
    }
    Ok(())
}

fn sys_stake_withdraw<'info>(stake_info: &Account<'info, StakeInfo>,
                             sys_stake_state: &UncheckedAccount<'info>,
                             clock: &Sysvar<'info, Clock>,
                             stake_history: &Sysvar<'info, StakeHistory>,
                             native_vault: &SystemAccount<'info>,
                             stake_info_seeds: &[&[&[u8]]],
                             withdraw_amount: u64) -> anchor_lang::Result<()> {
    let ix = stake::instruction::withdraw(&sys_stake_state.key(), &stake_info.key(), &native_vault.key(), withdraw_amount, None);
    solana_program::program::invoke_signed(
        &ix,
        &[
            sys_stake_state.to_account_info(),
            native_vault.to_account_info(),
            clock.to_account_info(),
            stake_history.to_account_info(),
            stake_info.to_account_info(),
        ],
        stake_info_seeds,
    ).map_err(Into::into)
}
