pub mod initialize_account;
pub mod delegate_stake;
pub mod initialize;
pub mod withdraw;
pub mod deactivate;

use anchor_lang::prelude::{msg, Account, AccountInfo, Clock, Program, Pubkey, Rent, StakeHistory, SystemAccount, Sysvar, UncheckedAccount};
use anchor_lang::error::Error;
use anchor_lang::solana_program::stake;
use anchor_lang::{pubkey, solana_program, system_program, Key, ToAccountInfo};
use anchor_lang::context::CpiContext;
use anchor_lang::system_program::System;

pub use initialize_account::*;
pub use delegate_stake::*;
pub use initialize::*;
pub use withdraw::*;
pub use deactivate::*;

use crate::error::ErrorCode::NeedMoreStakeToken;
use crate::stake_info::StakeInfo;

fn transfer_lamports<'info>(
    from_account: &AccountInfo<'info>,
    to_account: &AccountInfo<'info>,
    system: &AccountInfo<'info>,
    signer_seed:  &[&[&[u8]]],
    amount_of_lamports: u64,
) -> anchor_lang::Result<()> {
    // transfer fee to recipient
    system_program::transfer(
        CpiContext::new(
            system.to_account_info(),
            system_program::Transfer {
                from: from_account.to_account_info(),
                to: to_account.to_account_info(),
            },
        ).with_signer(signer_seed),
        amount_of_lamports,
    )?;
    Ok(())
}

fn rebalance<'info>(
    stake_info: &Account<'info, StakeInfo>,
    sys_stake_state: &UncheckedAccount<'info>,
    rent: &Sysvar<'info, Rent>,
    native_vault: &UncheckedAccount<'info>,
    clock: &Sysvar<'info, Clock>,
    stake_history: &Sysvar<'info, StakeHistory>,
    system: &Program<'info, System>,
    stake_info_seeds: &[&[&[u8]]],
    native_vault_seeds: &[&[&[u8]]],
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
        transfer_lamports(&native_vault.to_account_info(), &sys_stake_state.to_account_info(), &system.to_account_info(), native_vault_seeds, need_to_stake)?;
    }
    Ok(())
}

fn try_rebalance<'info>(
    sys_stake_state: &UncheckedAccount<'info>,
    rent: &Sysvar<'info, Rent>,
    native_vault: &UncheckedAccount<'info>,
    system: &Program<'info, System>,
    native_vault_seeds: &[&[&[u8]]],
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
        transfer_lamports(&native_vault.to_account_info(), &sys_stake_state.to_account_info(), &system.to_account_info(), native_vault_seeds, need_to_stake)?;
    }
    Ok(())
}

fn sys_stake_withdraw<'info>(stake_info: &Account<'info, StakeInfo>,
                             sys_stake_state: &UncheckedAccount<'info>,
                             clock: &Sysvar<'info, Clock>,
                             stake_history: &Sysvar<'info, StakeHistory>,
                             native_vault: &UncheckedAccount<'info>,
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

#[derive(Clone)]
pub struct Stake;

impl anchor_lang::Id for Stake {
    fn id() -> Pubkey {
        pubkey!("Stake11111111111111111111111111111111111111")
    }
}