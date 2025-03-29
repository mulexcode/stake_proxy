use anchor_lang::__private::base64::engine::Config;
use anchor_lang::Accounts;
use anchor_lang::prelude::*;
use anchor_spl::token;
use anchor_spl::token::{Mint, Token, TokenAccount};
use crate::state::{SystemConfig, ChainConfig, TokenConfig};
use crate::constants::*;
use crate::SOL_TOKEN_NAME;

#[event]
pub struct CashOutEvent {
    pub nonce: u64,
    pub token_name: String,
    pub from: Pubkey,
    pub target: String,
    pub amount: u64,
    pub chain_id: u64,
    pub target_chain_id: u64,
    pub decimals: u8,
}

#[derive(Accounts)]
#[instruction(token_name: String, targer_chain_id: u64)]
pub struct CashOutAccount<'info> {

    #[account(
        mut,
        seeds = [SYSTEM_CONFIG_SEED.as_bytes()],
        bump
    )]
    pub config: Account<'info, SystemConfig>,

    #[account(
        seeds = [TOKEN_CONFIG_SEED.as_bytes(), token_name.as_bytes()],
        bump
    )]
    pub token_config: Account<'info, TokenConfig>,

    #[account(
        mut,
        seeds = [CHAIN_CONFIG_SEED.as_bytes(), targer_chain_id.to_le_bytes().as_slice()],
        bump
    )]
    pub chain_config: Account<'info, ChainConfig>,

    #[account(mut)]
    pub token_mint: Account<'info, Mint>,
    #[account(mut,
        token::mint = token_mint,
        token::authority = payer,
    )]
    pub token_payer: Account<'info, TokenAccount>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CashOutArgs {
    token_name: String,
    target_chain_id: u64,
    target: String, 
    amount: u64,
}

pub fn handler(ctx: Context<CashOutAccount>, args: CashOutArgs) -> Result<()> { 
    if !ctx.accounts.token_config.enabled {
        return Ok(()); // TODO ERROR
    }
    
    if args.token_name == SOL_TOKEN_NAME {
        return Ok(()); // TODO ERROR
    }
    token::burn(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Burn {
                mint: ctx.accounts.token_mint.to_account_info(),
                from: ctx.accounts.token_payer.to_account_info(),
                authority: ctx.accounts.payer.to_account_info(),
            },
        ),
        args.amount,
    )?;
    
    emit!(CashOutEvent {
        nonce: ctx.accounts.chain_config.cash_out_nonce,
        token_name: args.token_name.clone(),
        from: ctx.accounts.payer.key(),
        target: args.target,
        target_chain_id: args.target_chain_id,
        amount: args.amount,
        chain_id: ctx.accounts.config.chain_id,
        decimals: ctx.accounts.token_mint.decimals,
    });
    ctx.accounts.chain_config.increase_cash_out_nonce();
    Ok(())
}