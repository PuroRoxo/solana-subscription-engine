use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};
use crate::{state::PlatformConfig, errors::BillingError, constants::MAX_PLATFORM_FEE_BPS};

#[derive(Accounts)]
pub struct InitializePlatform<'info> {
    #[account(
        init,
        payer = admin,
        space = PlatformConfig::LEN,
        seeds = [PlatformConfig::SEED_PREFIX],
        bump
    )]
    pub platform_config: Account<'info, PlatformConfig>,
    
    #[account(
        init,
        payer = admin,
        token::mint = payment_mint,
        token::authority = platform_config,
        seeds = [b"treasury"],
        bump
    )]
    pub treasury: Account<'info, TokenAccount>,
    
    pub payment_mint: Account<'info, anchor_spl::token::Mint>,
    
    #[account(mut)]
    pub admin: Signer<'info>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

pub fn initialize_platform(
    ctx: Context<InitializePlatform>,
    platform_fee_bps: u16,
) -> Result<()> {
    require!(
        platform_fee_bps <= MAX_PLATFORM_FEE_BPS,
        BillingError::InvalidPlatformFee
    );

    let platform_config = &mut ctx.accounts.platform_config;
    platform_config.admin = ctx.accounts.admin.key();
    platform_config.platform_fee_bps = platform_fee_bps;
    platform_config.treasury = ctx.accounts.treasury.key();
    platform_config.total_plans = 0;
    platform_config.total_subscriptions = 0;
    platform_config.total_revenue = 0;
    platform_config.bump = ctx.bumps.platform_config;

    msg!("Platform initialized with fee: {} bps", platform_fee_bps);
    
    Ok(())
}