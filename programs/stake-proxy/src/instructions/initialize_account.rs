use anchor_lang::prelude::*;
use anchor_lang::{solana_program, system_program};
use anchor_lang::solana_program::stake::state::{Authorized, Lockup, StakeStateV2};
use anchor_lang::solana_program::stake;
use crate::stake_info::StakeInfo;
use crate::constants::{NATIVE_VAULT_SEED, STAKE_INFO_SEED, STAKE_STATE_SEED, STAKE_TOKEN_MINT, DELEGATE_AUTHORITY_SEED};

use anchor_spl::{associated_token::AssociatedToken, token, token::{Mint, Token, TokenAccount}};
use crate::error::ErrorCode::{NeedMoreStakeToken, StakeTokenMintMismatch};
use crate::instructions;
use crate::instructions::Stake;

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

    /// CHECK: stake::initialize() will check its ownership
    #[account(mut)]
    pub sys_stake_state: UncheckedAccount<'info>, // system stake account

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

    /// The vault that holds the token.
    #[account(init_if_needed, payer = payer,
        associated_token::mint = token_mint,
        associated_token::authority = native_vault,
    )]
    pub token_vault: Account<'info, TokenAccount>,

    /// CHECK: check its ownership
    #[account(
        mut,
        seeds = [NATIVE_VAULT_SEED.as_bytes()],
        bump
    )]
    pub native_vault: UncheckedAccount<'info>, // should init in genesis block

    #[account(address = STAKE_TOKEN_MINT @ StakeTokenMintMismatch)]
    pub token_mint: Account<'info, Mint>, // init in genesis block
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    pub stake: Program<'info, Stake>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitializeAccountArgs {
    pub amount: u64,
    pub staker: Pubkey,
    pub withdrawer: Pubkey,
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

    initialize_stake_account(
        &ctx.accounts.delegate_authority,
        &ctx.accounts.sys_stake_state,
        &ctx.accounts.rent, 
        &ctx.accounts.native_vault, 
        &ctx.accounts.system_program, 
        ctx.bumps.delegate_authority, 
        ctx.bumps.native_vault,
        args.amount
    )?;

    ctx.accounts.stake_info.amount = args.amount;
    ctx.accounts.stake_info.staker_pubkey = args.staker;
    ctx.accounts.stake_info.withdrawer_pubkey = args.withdrawer;

    Ok(())
}

pub fn initialize_stake_account<'info>(
    delegate_auth: &UncheckedAccount<'info>,
    sys_stake_state: &UncheckedAccount<'info>,
    rent: &Sysvar<'info, Rent>,
    native_vault: &UncheckedAccount<'info>,
    system: &Program<'info, System>,
    delegate_stake_bump: u8,
    native_vault_bump: u8,
    stake_amount: u64,
) -> Result<()> {
    let sys_stake_state_key = sys_stake_state.key();
    
    let delegate_auth_seeds: &[&[&[u8]]] = &[&[DELEGATE_AUTHORITY_SEED.as_bytes(), &[delegate_stake_bump]]];
    let native_vault_seeds: &[&[&[u8]]] = &[&[NATIVE_VAULT_SEED.as_bytes(), &[native_vault_bump]]];

    instructions::try_rebalance(sys_stake_state, rent, native_vault, system, native_vault_seeds, stake_amount)?;
    let authorized = &Authorized {
        staker: delegate_auth.key(),
        withdrawer: delegate_auth.key(),
    };
    let ix = stake::instruction::initialize(&sys_stake_state_key, authorized, &Lockup::default());
    
    let res: Result<()> = solana_program::program::invoke_signed(
        &ix,
        &[
            sys_stake_state.to_account_info(),
            rent.to_account_info(),
        ],
        delegate_auth_seeds,
    ).map_err(Into::into);
    res
}