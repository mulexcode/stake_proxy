use anchor_lang::__private::base64::engine::Config;
use anchor_lang::{system_program, Accounts};
use anchor_lang::prelude::*;
use anchor_spl::token;
use anchor_spl::token::{Mint, Token, TokenAccount};
use crate::cash_out::CashOutEvent;
use crate::state::{ChainConfig, SystemConfig};
use crate::constants::*;

pub const SOL_TOKEN_NAME: &str = "sol";


#[derive(Accounts)]
#[instruction(target_chain_id: u64)]
pub struct CashOutSolAccount<'info> {
    #[account(
        mut,
        seeds = [SYSTEM_CONFIG_SEED.as_bytes()],
        bump
    )]
    pub config: Account<'info, SystemConfig>,
    
    #[account(
        mut,
        seeds = [CHAIN_CONFIG_SEED.as_bytes(), target_chain_id.to_le_bytes().as_slice()],
        bump
    )]
    pub chain_config: Account<'info, ChainConfig>,
    /// CHECK:  no need
    #[account(
        mut,
        seeds = [NATIVE_TOKEN_VAULT_SEED.as_bytes()],
        bump
    )]
    pub native_token_vault: AccountInfo<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<CashOutSolAccount>, target_chain_id: u64, target: Pubkey, amount: u64) -> Result<()> {
    let transfer_cpi_accounts = system_program::Transfer {
        from: ctx.accounts.payer.to_account_info(),
        to: ctx.accounts.native_token_vault.to_account_info(),
    };

    let cpi_ctx = CpiContext::new(ctx.accounts.system_program.to_account_info(), transfer_cpi_accounts);
    system_program::transfer(cpi_ctx, amount)?;

    emit!(CashOutEvent {
        nonce: ctx.accounts.chain_config.cash_out_nonce,
        token_name: SOL_TOKEN_NAME.to_string(),
        from: ctx.accounts.payer.key(),
        target: target.to_string(),
        target_chain_id: ctx.accounts.chain_config.chain_id,
        amount,
        chain_id: ctx.accounts.config.chain_id,
        decimals: 9,
    });
    ctx.accounts.chain_config.increase_cash_out_nonce();
    Ok(())
}