pub mod initialize_account;
pub mod delegate_stake;

use anchor_lang::prelude::{Account, AccountInfo, Rent, SystemAccount, Sysvar, UncheckedAccount};
use anchor_lang::error::Error;
use anchor_lang::ToAccountInfo;
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

fn rebalance<'info>(sys_stake_state: &UncheckedAccount<'info>,
             rent: &Sysvar<'info, Rent>,
             native_vault: &SystemAccount<'info>,
             stake_amount: u64, init: bool) -> anchor_lang::Result<()> {
    // check sol balance
    let min_balance = rent.minimum_balance(sys_stake_state.data_len());
    let expected_balance = min_balance + stake_amount;
    if expected_balance < sys_stake_state.lamports() {
        if init {
            return Err(Error::from(NeedMoreStakeToken));
        }
        let need_to_withdraw = expected_balance - sys_stake_state.lamports();
    }
    
    let need_to_stake = expected_balance - sys_stake_state.lamports();
    if need_to_stake > 0 {
        transfer_lamports(&native_vault.to_account_info(), &sys_stake_state.to_account_info(), need_to_stake)?;
    }
    Ok(())
}