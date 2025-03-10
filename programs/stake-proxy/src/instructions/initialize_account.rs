use anchor_lang::prelude::*;
use anchor_lang::{solana_program, system_program};
use anchor_lang::solana_program::stake::state::{Authorized, Lockup, StakeStateV2};
use anchor_lang::solana_program::stake;
use crate::stake_info::StakeInfo;
use crate::state::*;
use crate::constants::{NATIVE_TOKEN_VAULT, NATIVE_VAULT_SEED, STAKE_INFO_SEED, STAKE_STATE_SEED, STAKE_TOKEN_MINT};

use anchor_spl::{associated_token, associated_token::AssociatedToken, token, token::{Mint, Token, TokenAccount}};
use crate::error::ErrorCode::{InsufficientFundsForTransaction, NeedMoreStakeToken, StakeTokenMintMismatch};
use crate::instructions;

#[derive(Accounts)]
pub struct InitializeAccount<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + std::mem::size_of::<StakeInfo>(),
        seeds = [
            STAKE_INFO_SEED.as_bytes(),
            sys_stake_state.key().as_ref()
        ],
        bump
    )]
    pub stake_info: Account<'info, StakeInfo>,

    /// CHECK: stake state
    #[account(mut)]
    pub sys_stake_state: UncheckedAccount<'info>, // system stake account

    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut,
        token::mint = token_mint,
        token::authority = payer,
    )]
    pub token_payer: Account<'info, TokenAccount>,

    /// CHECK: no need to check
    pub staker: UncheckedAccount<'info>,
    /// CHECK: no need to check
    pub withdrawer: UncheckedAccount<'info>,

    /// The vault that holds the token.
    #[account(init_if_needed, payer = payer,
        associated_token::mint = token_mint,
        associated_token::authority = native_vault,
    )]
    pub token_vault: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [NATIVE_VAULT_SEED.as_bytes()],
        bump
    )]
    pub native_vault: SystemAccount<'info>, // init in genesis block

    #[account(address = STAKE_TOKEN_MINT @ StakeTokenMintMismatch)]
    pub token_mint: Account<'info, Mint>, // init in genesis block
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitializeAccountArgs {
    pub amount: u64,
}

pub fn handler(ctx: Context<InitializeAccount>, args: InitializeAccountArgs) -> Result<()> {
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.token_payer.to_account_info(),
                to: ctx.accounts.token_vault.to_account_info(),
                authority: ctx.accounts.payer.to_account_info(),
            },
        ),
        args.amount,
    )?;
    
    initialize_stake_account(&ctx.accounts.stake_info, &ctx.accounts.sys_stake_state, &ctx.accounts.rent, &ctx.accounts.native_vault, ctx.bumps.stake_info, args.amount)?;
    
    ctx.accounts.stake_info.amount = args.amount;
    ctx.accounts.stake_info.staker_pubkey = ctx.accounts.staker.key();
    ctx.accounts.stake_info.withdrawer_pubkey = ctx.accounts.withdrawer.key();
    
    Ok(())
}

pub fn initialize_stake_account<'info>(
    stake_info: &Account<'info, StakeInfo>,
    sys_stake_state: &UncheckedAccount<'info>,
    rent: &Sysvar<'info, Rent>,
    native_vault: &SystemAccount<'info>,
    stake_info_bump: u8,
    stake_amount: u64,
) -> Result<()> {
    let sys_stake_state_key = sys_stake_state.key();
    
    // check sol balance
    let min_balance = rent.minimum_balance(sys_stake_state.data_len());
    let expected_balance = min_balance + stake_amount;
    if expected_balance < sys_stake_state.lamports() {
        return Err(Error::from(NeedMoreStakeToken));
    }
    let need_to_transfer = expected_balance - sys_stake_state.lamports();
    if need_to_transfer > 0 {
        instructions::transfer_lamports(&native_vault.to_account_info(), &sys_stake_state.to_account_info(), need_to_transfer)?;
    }
    instructions::try_rebalance(sys_stake_state, rent, native_vault, stake_amount)?;
    
    
    let stake_info_seeds: &[&[&[u8]]] = &[&[STAKE_INFO_SEED.as_bytes(), sys_stake_state_key.as_ref(), &[stake_info_bump]]];
    let authorized = &Authorized {
        staker: stake_info.key(),
        withdrawer: stake_info.key(),
    };
    let ix = stake::instruction::initialize(&sys_stake_state_key, authorized, &Lockup::default());
    solana_program::program::invoke_signed(
        &ix,
        &[
            sys_stake_state.to_account_info(),
            rent.to_account_info(),
        ],
        stake_info_seeds,
    ).map_err(Into::into)
}