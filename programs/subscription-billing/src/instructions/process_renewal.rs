use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::{
    state::{PlatformConfig, SubscriptionPlan, UserSubscription},
    errors::BillingError,
    events::SubscriptionRenewed
};

#[derive(Accounts)]
pub struct ProcessRenewal<'info> {
    #[account(
        mut,
        seeds = [PlatformConfig::SEED_PREFIX],
        bump = platform_config.bump
    )]
    pub platform_config: Account<'info, PlatformConfig>,
    
    #[account(
        mut,
        seeds = [SubscriptionPlan::SEED_PREFIX, subscription_plan.plan_id.to_le_bytes().as_ref()],
        bump = subscription_plan.bump
    )]
    pub subscription_plan: Account<'info, SubscriptionPlan>,
    
    #[account(
        mut,
        seeds = [UserSubscription::SEED_PREFIX, user_subscription.user.key().as_ref(), subscription_plan.key().as_ref()],
        bump = user_subscription.bump,
        constraint = user_subscription.auto_renew @ BillingError::SubscriptionNotActive
    )]
    pub user_subscription: Account<'info, UserSubscription>,
    
    #[account(
        mut,
        constraint = user_payment_account.mint == user_subscription.payment_token,
        constraint = user_payment_account.owner == user_subscription.user
    )]
    pub user_payment_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        seeds = [b"treasury"],
        bump,
        constraint = treasury.mint == user_subscription.payment_token
    )]
    pub treasury: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
}

pub fn process_renewal(ctx: Context<ProcessRenewal>) -> Result<()> {
    let subscription = &ctx.accounts.user_subscription;
    let plan = &ctx.accounts.subscription_plan;
    
    // Check if subscription needs renewal (within 24 hours of expiry)
    let clock = Clock::get()?;
    let renewal_window = 86400; // 24 hours
    
    require!(
        clock.unix_timestamp >= (subscription.expires_at - renewal_window),
        BillingError::SubscriptionNotActive
    );

    let amount = plan.price_per_period;
    let platform_fee = amount
        .checked_mul(ctx.accounts.platform_config.platform_fee_bps as u64)
        .ok_or(BillingError::MathOverflow)?
        .checked_div(10000)
        .ok_or(BillingError::MathOverflow)?;

    // Process payment
    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.user_payment_account.to_account_info(),
            to: ctx.accounts.treasury.to_account_info(),
            authority: ctx.accounts.user_payment_account.to_account_info(),
        },
    );
    token::transfer(transfer_ctx, amount)?;

    // Extend subscription
    let subscription = &mut ctx.accounts.user_subscription;
    subscription.extend_subscription(1, plan.period_duration, amount)?;

    // Update stats
    let plan = &mut ctx.accounts.subscription_plan;
    plan.total_revenue = plan.total_revenue
        .checked_add(amount.checked_sub(platform_fee).ok_or(BillingError::MathOverflow)?)
        .ok_or(BillingError::MathOverflow)?;

    let platform_config = &mut ctx.accounts.platform_config;
    platform_config.total_revenue = platform_config.total_revenue
        .checked_add(platform_fee)
        .ok_or(BillingError::MathOverflow)?;

    emit!(SubscriptionRenewed {
        user: subscription.user,
        plan: ctx.accounts.subscription_plan.key(),
        amount_paid: amount,
        new_expiry: subscription.expires_at,
        timestamp: clock.unix_timestamp,
    });

    msg!("Subscription renewed for user: {}, new expiry: {}", subscription.user, subscription.expires_at);
    
    Ok(())
}