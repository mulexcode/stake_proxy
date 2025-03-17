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
use crate::{STAKE_INFO_SEED, NATIVE_VAULT_SEED, STAKE_TOKEN_MINT, STAKE_CONFIG, Stake, DELEGATE_AUTHORITY_SEED};

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
        address = stake_info.staker_pubkey,
    )]
    pub authority: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK: check its ownership
    #[account(seeds = [DELEGATE_AUTHORITY_SEED.as_bytes()], bump)]
    pub delegate_authority: UncheckedAccount<'info>,
    #[account(mut,
        token::mint = token_mint,
        token::authority = payer,
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
    /// CHECK: no check
    #[account(address = STAKE_CONFIG @ StakeTokenMintMismatch)]
    pub stake_config: UncheckedAccount<'info>,
    pub stake: Program<'info, Stake>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct DelegateStakeArgs {
    pub amount: u64,
}

pub fn handler(ctx: Context<DelegateStakeAccount>, args: DelegateStakeArgs) -> Result<()> {
    if args.amount < ctx.accounts.stake_info.amount {
        return Err(StakeAmountTooSmall.into());
    }
    
    let transfer_amount =  args.amount - ctx.accounts.stake_info.amount;
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
            transfer_amount,
        )?;
    }
    
    
    delegate_stake(
        &ctx.accounts.delegate_authority,
        &ctx.accounts.sys_stake_state,
        &ctx.accounts.rent,
        &ctx.accounts.vote,
        &ctx.accounts.clock,
        &ctx.accounts.stake_history,
        &ctx.accounts.native_vault,
        &ctx.accounts.system_program,
        &ctx.accounts.stake_config,
        ctx.bumps.delegate_authority,
        ctx.bumps.native_vault,
        args.amount)?;
    ctx.accounts.stake_info.amount = args.amount;
    Ok(())
}

fn delegate_stake<'info>(
    delegate_auth: &UncheckedAccount<'info>,
    sys_stake_state: &UncheckedAccount<'info>,
    rent: &Sysvar<'info, Rent>,
    vote: &UncheckedAccount<'info>,
    clock: &Sysvar<'info, Clock>,
    stake_history: &Sysvar<'info, StakeHistory>,
    native_vault: &UncheckedAccount<'info>,
    system: &Program<'info, System>,
    stake_config: &UncheckedAccount<'info>,
    delegate_auth_bump: u8,
    native_bump: u8,
    stake_amount: u64,
) -> Result<()> {
    let delegate_auth_seeds: &[&[&[u8]]] = &[&[DELEGATE_AUTHORITY_SEED.as_bytes(), &[delegate_auth_bump]]];
    let native_vault_seeds: &[&[&[u8]]] = &[&[NATIVE_VAULT_SEED.as_bytes(), &[native_bump]]];
    
    rebalance(delegate_auth, sys_stake_state, rent, native_vault,  clock, stake_history, system, delegate_auth_seeds, native_vault_seeds, stake_amount)?;
    
    let ix = stake::instruction::delegate_stake(&sys_stake_state.key(), &delegate_auth.key(), &vote.key());
    solana_program::program::invoke_signed(
        &ix,
        &[
            sys_stake_state.to_account_info(),
            vote.to_account_info(),
            clock.to_account_info(),
            stake_history.to_account_info(),
            stake_config.to_account_info(),
            delegate_auth.to_account_info(),
        ],
        delegate_auth_seeds,
    ).map_err(Into::into)
}