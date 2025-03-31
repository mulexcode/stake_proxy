#![allow(unexpected_cfgs)]
mod state;
mod constants;
mod instructions;
mod utils;
mod error;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("HFkN6HWCb94UNExgdv7sQ9XydRsQGpRoMvFZPW4BmcqW");

#[program]
pub mod solana_bridge {
    use super::*;

    pub fn cash_out(ctx: Context<CashOutAccount>, args: CashOutArgs) -> Result<()> {
        cash_out::handler(ctx, args)
    }

    pub fn cash_out_sol(ctx: Context<CashOutSolAccount>, chain_id: u64,  amount: u64) -> Result<()> {
        cash_out_sol::handler(ctx, chain_id, amount)
    }

    pub fn enable_chain(ctx: Context<EnableChainAccount>, chain_id: u64) -> Result<()> {
        enable_chain::handler(ctx, chain_id)
    }

    pub fn enable_token(ctx: Context<EnableTokenAccount>, token_name: String) -> Result<()> {
        enable_token::handler(ctx, token_name)
    }

    pub fn initialize(ctx: Context<InitializeAccount>, chain_id: u64, manager: Pubkey, secp256k1_manager: [u8; 20]) -> Result<()> {
        initialize::handler(ctx, chain_id, manager, secp256k1_manager)
    }

    pub fn payout(ctx: Context<PayoutAccount>, args: PayoutArgs) -> Result<()> {
        payout::handler(ctx, args)
    }

    pub fn payout_sol(ctx: Context<PayoutSolAccount>, args: PayoutSolArgs) -> Result<()> {
        payout_sol::handler(ctx, args)
    }

    pub fn update_manager(ctx: Context<UpdateManagerAccount>, manager: Pubkey, secp256k1_manager: [u8; 20]) -> Result<()> {
        update_manager::handler(ctx, manager, secp256k1_manager)
    }
}




