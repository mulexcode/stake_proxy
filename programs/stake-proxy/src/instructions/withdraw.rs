use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use anchor_lang::solana_program::stake;
use anchor_lang::solana_program::vote;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token;
use anchor_spl::token::{Mint, Token, TokenAccount};
use crate::error::ErrorCode::{StakeAmountTooBig, StakeAmountTooSmall, StakeTokenMintMismatch};
use crate::stake_info::StakeInfo;
use crate::{STAKE_INFO_SEED, NATIVE_VAULT_SEED, STAKE_TOKEN_MINT, Stake};

#[derive(Accounts)]
pub struct WithdrawAccount<'info>  {
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

    /// The vault that holds the token.
    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = native_vault,
    )]
    pub token_vault: Account<'info, TokenAccount>,
    /// CHECK: init in genesis block
    #[account(
        mut,
        seeds = [NATIVE_VAULT_SEED.as_bytes()],
        bump
    )]
    pub native_vault: UncheckedAccount<'info>, // init in genesis block

    #[account(mut,
        address = stake_info.withdrawer_pubkey,
    )]
    pub authority: Signer<'info>,
    
    #[account(mut,
        token::mint = token_mint,
    )]
    pub token_receiver: Account<'info, TokenAccount>,
    
    pub clock: Sysvar<'info, Clock>,
    pub stake_history: Sysvar<'info, StakeHistory>,
    #[account(address = STAKE_TOKEN_MINT @ StakeTokenMintMismatch)]
    pub token_mint: Account<'info, Mint>, // init in genesis block
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub stake: Program<'info, Stake>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct WithdrawArgs {
    pub amount: u64,
}

pub fn handler(ctx: Context<WithdrawAccount>, args: WithdrawArgs) -> Result<()> {
    if args.amount > ctx.accounts.stake_info.amount {
        return Err(StakeAmountTooBig.into());
    }
    
    if args.amount == 0 {
        return Ok(());
    }
    
    withdraw(
        &ctx.accounts.stake_info,
        &ctx.accounts.sys_stake_state,
        &ctx.accounts.clock,
        &ctx.accounts.stake_history,
        &ctx.accounts.native_vault,
        ctx.bumps.stake_info,
        args.amount)?;

    let native_vault_seeds: &[&[&[u8]]] = &[&[NATIVE_VAULT_SEED.as_bytes(),  &[ctx.bumps.native_vault]]];
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.token_vault.to_account_info(),
                to: ctx.accounts.token_receiver.to_account_info(),
                authority: ctx.accounts.native_vault.to_account_info(),
            },
        ).with_signer(native_vault_seeds),
        args.amount,
    )?;
    
    ctx.accounts.stake_info.amount -= args.amount;
    Ok(())
}

fn withdraw<'info>(
    stake_info: &Account<'info, StakeInfo>,
    sys_stake_state: &UncheckedAccount<'info>,
    clock: &Sysvar<'info, Clock>,
    stake_history: &Sysvar<'info, StakeHistory>,
    native_vault: &UncheckedAccount<'info>,
    stake_info_bump: u8,
    lamport: u64,
) -> Result<()> {
    let sys_stake_state_key = sys_stake_state.key();
    let stake_info_seeds: &[&[&[u8]]] = &[&[STAKE_INFO_SEED.as_bytes(), sys_stake_state_key.as_ref(), &[stake_info_bump]]];

    let ix = stake::instruction::withdraw(&sys_stake_state.key(), &stake_info.key(), &native_vault.key(), lamport, None);
    
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