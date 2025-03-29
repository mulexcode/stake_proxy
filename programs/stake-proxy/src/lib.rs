#![allow(unexpected_cfgs)]
pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("StakePPyu7cgbWngmEhW5Gr86D9x1HZoseKy2JFTNbP");


#[program]
pub mod stake_proxy {
    use super::*;
    
    pub fn initialize<'a, 'b, 'c, 'info>(ctx: Context<'a, 'b, 'c, 'info, Initialize<'info>>) -> Result<()> {
        initialize::handler(ctx)
    }

    pub fn initialize_account<'a, 'b, 'c, 'info>(ctx: Context<'a, 'b, 'c, 'info, InitializeAccount<'info>>, args: InitializeAccountArgs) -> Result<()> {
        initialize_account::handler(ctx, args)
    }

    pub fn delegate_stake<'a, 'b, 'c, 'info>(ctx: Context<'a, 'b, 'c, 'info, DelegateStakeAccount>, args: DelegateStakeArgs) -> Result<()> {
        delegate_stake::handler(ctx, args)
    }

    pub fn withdraw<'a, 'b, 'c, 'info>(ctx: Context<'a, 'b, 'c, 'info, WithdrawAccount>, args: WithdrawArgs) -> Result<()> {
        withdraw::handler(ctx, args)
    }
    
    pub fn deactivate<'a, 'b, 'c, 'info>(ctx: Context<'a, 'b, 'c, 'info, DeactivateAccount>) -> Result<()> {
        deactivate::handler(ctx)
    }
}
