use anchor_lang::__private::base64::engine::Config;
use anchor_lang::Accounts;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::keccak::hash;
use anchor_lang::solana_program::sysvar::SysvarId;
use anchor_spl::token;
use anchor_spl::token::{Mint, Token, TokenAccount};
use crate::state::{SystemConfig, ChainConfig, TokenConfig};
use anchor_lang::solana_program::sysvar;
use anchor_lang::solana_program::sysvar::instructions::{load_current_index_checked, load_instruction_at_checked};
use crate::constants::*;
use crate::error::ErrorCode;
use crate::utils;

#[event]
pub struct PayoutEvent {
    pub nonce: u64,
    pub token_name: String,
    pub target: Pubkey,
    pub token_mint: Pubkey,
    pub amount: u64,
    pub from_chain_id: u64,
}

#[derive(Accounts)]
#[instruction(token_name: String, from_chain_id: u64)]
pub struct PayoutAccount<'info> {

    #[account(
        mut,
        seeds = [SYSTEM_CONFIG_SEED.as_bytes()],
        bump
    )]
    pub config: Account<'info, SystemConfig>,

    #[account(
        mut,
        seeds = [CHAIN_CONFIG_SEED.as_bytes(), from_chain_id.to_le_bytes().as_slice()],
        bump
    )]
    pub chain_config: Account<'info, ChainConfig>,

    #[account(
        seeds = [TOKEN_CONFIG_SEED.as_bytes(), token_name.as_bytes()],
        bump
    )]
    pub token_config: Account<'info, TokenConfig>,

    /// CHECK:  no need
    #[account(
        seeds = [NATIVE_TOKEN_VAULT_SEED.as_bytes()],
        bump
    )]
    pub native_token_vault: AccountInfo<'info>,

    #[account(mut)]
    pub token_mint: Account<'info, Mint>,
    #[account(mut,
        token::mint = token_mint,
    )]
    pub token_receiver: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    /// CHECK: check its id
    #[account(address = sysvar::instructions::id())]
    pub instruction_sysvar: AccountInfo<'info>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct PayoutArgs {
    token_name: String,
    from_chain_id: u64,
    amount: u64,
    magic: [u8; 8], 
    nonce: u64, 
    signature: [u8; 64], 
    recovery_id: u8
}

pub fn handler(ctx: Context<PayoutAccount>, args: PayoutArgs) -> Result<()> {
    if !ctx.accounts.token_config.enabled {
        return Ok(()); // TODO ERROR
    }

    if ctx.accounts.chain_config.payout_nonce != args.nonce {
        return Err(ErrorCode::InvalidPayoutNonce.into());
    }

    let current_index = load_current_index_checked(&ctx.accounts.instruction_sysvar.to_account_info())?;
    if current_index == 0 {
        return Err(ErrorCode::MissingSecp256k1Instruction.into());
    }

    let secp256k1_index = current_index - 1;
    let secp256k1_instruction = load_instruction_at_checked(secp256k1_index as usize, &ctx.accounts.instruction_sysvar.to_account_info())?;

    let mut message = Vec::new();
    message.extend_from_slice(args.magic.as_slice());
    message.extend_from_slice(&args.nonce.to_le_bytes());
    message.extend_from_slice(args.token_name.as_bytes());
    message.extend_from_slice(&ctx.accounts.token_receiver.key().to_bytes());
    message.extend_from_slice(&args.amount.to_le_bytes());
    message.extend_from_slice(&args.from_chain_id.to_le_bytes());
    // let message_hash = hash(&message);

    let hex_string: String = message.iter()
        .map(|b| format!("{:02x}", b)) // 每个字节转换为两位十六进制
        .collect(); // 收集为字符串
    
    msg!(hex_string.as_str());

    utils::verify_secp256k1_ix(
        &secp256k1_instruction,
        &ctx.accounts.config.secp256k1_manager,
        message.as_ref(),
        args.signature.as_slice(),
        args.recovery_id,
    )?;
    
    token::mint_to(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::MintTo {
                mint: ctx.accounts.token_mint.to_account_info(),
                to: ctx.accounts.token_receiver.to_account_info(),
                authority: ctx.accounts.native_token_vault.to_account_info(),
            },
        ).with_signer(&[&[NATIVE_TOKEN_VAULT_SEED.as_bytes(), &[ctx.bumps.native_token_vault]]]),
        args.amount,
    )?;

    emit!(PayoutEvent{
        nonce:args.nonce,
        token_name: args.token_name,
        target: ctx.accounts.token_receiver.key(),
        token_mint: ctx.accounts.token_mint.key(),
        amount:args.amount,
        from_chain_id: args.from_chain_id,
    });
    ctx.accounts.chain_config.increase_payout_nonce();
    Ok(())
}