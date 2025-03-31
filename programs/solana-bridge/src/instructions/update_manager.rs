use anchor_lang::__private::base64::engine::Config;
use anchor_lang::Accounts;
use anchor_lang::prelude::*;
use crate::state::SystemConfig;
use crate::constants::*;

#[derive(Accounts)]
pub struct UpdateManagerAccount<'info> {

    #[account(
        mut,
        has_one=manager,
        seeds = [SYSTEM_CONFIG_SEED.as_bytes()],
        bump
    )]
    pub config: Account<'info, SystemConfig>,
    
    #[account(mut)]
    pub manager: Signer<'info>,
}

pub fn handler(ctx: Context<UpdateManagerAccount>, manager: Pubkey, secp256k1_manager: [u8; 20]) -> Result<()> {
    ctx.accounts.config.update_manager(manager, secp256k1_manager);
    Ok(())
}