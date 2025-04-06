use anchor_lang::__private::base64::engine::Config;
use anchor_lang::{system_program, Accounts};
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
use crate::{utils, payout::PayoutEvent, cash_out_sol::SOL_TOKEN_NAME};

#[derive(Accounts)]
#[instruction(from_chain_id: u64)]
pub struct PayoutSolAccount<'info> {

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
        seeds = [TOKEN_CONFIG_SEED.as_bytes(), SOL_TOKEN_NAME.as_bytes()],
        bump
    )]
    pub token_config: Account<'info, TokenConfig>,

    /// CHECK:  no need
    #[account(
        mut,
        seeds = [NATIVE_TOKEN_VAULT_SEED.as_bytes()],
        bump
    )]
    pub native_token_vault: AccountInfo<'info>,
    
    /// CHECK: no need
    #[account(mut)]
    pub receiver: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
    /// CHECK: check its id
    #[account(address = sysvar::instructions::id())]
    pub instruction_sysvar: AccountInfo<'info>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct PayoutSolArgs {
    from_chain_id: u64,
    amount: u64,
    magic: [u8; 8], 
    nonce: u64, 
    signature: [u8; 64], 
    recovery_id: u8
}

pub fn handler(ctx: Context<PayoutSolAccount>, args: PayoutSolArgs) -> Result<()> {
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
    message.extend_from_slice(SOL_TOKEN_NAME.as_bytes());
    message.extend_from_slice(&ctx.accounts.receiver.key().to_bytes());
    message.extend_from_slice(&args.amount.to_le_bytes());
    message.extend_from_slice(&args.from_chain_id.to_le_bytes());
    // let message_hash = hash(&message);

    let hex_string: String = message.iter()
        .map(|b| format!("{:02x}", b)) // 每个字节转换为两位十六进制
        .collect(); // 收集为字符串
    
    msg!(hex_string.as_str());

    utils::verify_secp256k1_ix(
        &secp256k1_instruction,
        secp256k1_index as u8,
        &ctx.accounts.config.secp256k1_manager,
        message.as_ref(),
        args.signature.as_slice(),
        args.recovery_id,
    )?;
    
    **ctx.accounts.native_token_vault.try_borrow_mut_lamports()? -= args.amount;
    **ctx.accounts.receiver.try_borrow_mut_lamports()? += args.amount;


    emit!(PayoutEvent{
        nonce:args.nonce,
        token_name: SOL_TOKEN_NAME.to_string(),
        target: ctx.accounts.receiver.key(),
        token_mint: system_program::ID,
        amount:args.amount,
        from_chain_id: args.from_chain_id,
    });
    ctx.accounts.chain_config.increase_payout_nonce();
    Ok(())
}