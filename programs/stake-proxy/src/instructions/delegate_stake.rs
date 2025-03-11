use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use anchor_lang::solana_program::stake;
use anchor_lang::solana_program::vote;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token;
use anchor_spl::token::{Mint, Token, TokenAccount};
use crate::error::ErrorCode::{StakeAmountTooSmall, StakeTokenMintMismatch};
use crate::instructions::rebalance;
use crate::stake_info::StakeInfo;
use crate::{STAKE_INFO_SEED,NATIVE_VAULT_SEED, STAKE_TOKEN_MINT};

#[derive(Accounts)]
pub struct DelegateStakeAccount<'info>  {
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
        token::authority = authority,
    )]
    pub token_payer: Account<'info, TokenAccount>,

    /// CHECK: check its ownership
    #[account(mut, owner=vote::program::ID)]
    pub vote: UncheckedAccount<'info>,
    pub clock: Sysvar<'info, Clock>,
    pub stake_history: Sysvar<'info, StakeHistory>,
    #[account(address = STAKE_TOKEN_MINT @ StakeTokenMintMismatch)]
    pub token_mint: Account<'info, Mint>, // init in genesis block
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct DelegateStakeArgs {
    pub amount: u64,
}

pub fn handler(ctx: Context<DelegateStakeAccount>, args: DelegateStakeArgs) -> Result<()> {
    if args.amount < ctx.accounts.stake_info.amount {
        return Err(StakeAmountTooSmall.into());
    }
    
    let transfer_amount = ctx.accounts.stake_info.amount - args.amount;
    if transfer_amount > 0 {
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.token_payer.to_account_info(),
                    to: ctx.accounts.token_vault.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                },
            ),
            args.amount,
        )?;
    }
    
    
    delegate_stake(
        &ctx.accounts.stake_info,
        &ctx.accounts.sys_stake_state,
        &ctx.accounts.rent,
        &ctx.accounts.vote,
        &ctx.accounts.clock,
        &ctx.accounts.stake_history,
        &ctx.accounts.native_vault, 
        ctx.bumps.stake_info,
        args.amount)
}

fn delegate_stake<'info>(
    stake_info: &Account<'info, StakeInfo>,
    sys_stake_state: &UncheckedAccount<'info>,
    rent: &Sysvar<'info, Rent>,
    vote: &UncheckedAccount<'info>,
    clock: &Sysvar<'info, Clock>,
    stake_history: &Sysvar<'info, StakeHistory>,
    native_vault: &UncheckedAccount<'info>,
    stake_info_bump: u8,
    stake_amount: u64,
) -> Result<()> {
    let sys_stake_state_key = sys_stake_state.key();
    let stake_info_seeds: &[&[&[u8]]] = &[&[STAKE_INFO_SEED.as_bytes(), sys_stake_state_key.as_ref(), &[stake_info_bump]]];
    
    rebalance(stake_info, sys_stake_state, rent, native_vault,  clock, stake_history, stake_info_seeds, stake_amount)?;
    
    let ix = stake::instruction::delegate_stake(&sys_stake_state.key(), &stake_info.key(), &vote.key());
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