use anchor_lang::prelude::*;
use anchor_spl::{
    token::{self, Token, TokenAccount, Transfer},
    associated_token::AssociatedToken,
};
use crate::{
    state::{PlatformConfig, SubscriptionPlan, UserSubscription, SubscriptionStatus},
    errors::BillingError,
    constants::MAX_PERIODS_PER_PAYMENT,
    events::SubscriptionCreated
};

#[derive(Accounts)]
pub struct Subscribe<'info> {
    #[account(
        mut,
        seeds = [PlatformConfig::SEED_PREFIX],
        bump = platform_config.bump
    )]
    pub platform_config: Account<'info, PlatformConfig>,
    
    #[account(
        mut,
        seeds = [SubscriptionPlan::SEED_PREFIX, subscription_plan.plan_id.to_le_bytes().as_ref()],
        bump = subscription_plan.bump,
        constraint = subscription_plan.active @ BillingError::PlanNotActive,
        constraint = !subscription_plan.is_at_capacity() @ BillingError::PlanAtCapacity
    )]
    pub subscription_plan: Account<'info, SubscriptionPlan>,
    
    #[account(
        init,
        payer = user,
        space = UserSubscription::LEN,
        seeds = [UserSubscription::SEED_PREFIX, user.key().as_ref(), subscription_plan.key().as_ref()],
        bump
    )]
    pub user_subscription: Account<'info, UserSubscription>,
    
    #[account(
        mut,
        constraint = user_payment_account.mint == payment_mint.key(),
        constraint = user_payment_account.owner == user.key()
    )]
    pub user_payment_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        seeds = [b"treasury"],
        bump,
        constraint = treasury.mint == payment_mint.key()
    )]
    pub treasury: Account<'info, TokenAccount>,
    
    pub payment_mint: Account<'info, anchor_spl::token::Mint>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn subscribe(
    ctx: Context<Subscribe>,
    periods_to_pay: u8,
) -> Result<()> {
    require!(
        periods_to_pay > 0 && periods_to_pay <= MAX_PERIODS_PER_PAYMENT,
        BillingError::InvalidPeriods
    );

    let plan = &ctx.accounts.subscription_plan;
    let clock = Clock::get()?;
    
    // Calculate payment amount
    let total_amount = plan.price_per_period
        .checked_mul(periods_to_pay as u64)
        .ok_or(BillingError::MathOverflow)?;

    // Calculate platform fee
    let platform_fee = total_amount
        .checked_mul(ctx.accounts.platform_config.platform_fee_bps as u64)
        .ok_or(BillingError::MathOverflow)?
        .checked_div(10000)
        .ok_or(BillingError::MathOverflow)?;

    // Transfer payment to treasury
    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.user_payment_account.to_account_info(),
            to: ctx.accounts.treasury.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        },
    );
    token::transfer(transfer_ctx, total_amount)?;

    // Set up subscription
    let subscription = &mut ctx.accounts.user_subscription;
    subscription.user = ctx.accounts.user.key();
    subscription.plan = ctx.accounts.subscription_plan.key();
    subscription.status = SubscriptionStatus::Active;
    subscription.subscribed_at = clock.unix_timestamp;
    subscription.expires_at = clock.unix_timestamp
        .checked_add(
            plan.period_duration
                .checked_mul(periods_to_pay as i64)
                .ok_or(BillingError::MathOverflow)?
        )
        .ok_or(BillingError::MathOverflow)?;
    subscription.last_payment_at = clock.unix_timestamp;
    subscription.total_paid = total_amount;
    subscription.payment_token = ctx.accounts.payment_mint.key();
    subscription.auto_renew = true;
    subscription.periods_paid = periods_to_pay as u64;
    subscription.bump = ctx.bumps.user_subscription;

    // Update plan stats
    let plan = &mut ctx.accounts.subscription_plan;
    plan.current_subscribers = plan.current_subscribers
        .checked_add(1)
        .ok_or(BillingError::MathOverflow)?;
    plan.total_revenue = plan.total_revenue
        .checked_add(total_amount.checked_sub(platform_fee).ok_or(BillingError::MathOverflow)?)
        .ok_or(BillingError::MathOverflow)?;

    // Update platform stats
    let platform_config = &mut ctx.accounts.platform_config;
    platform_config.total_subscriptions = platform_config.total_subscriptions
        .checked_add(1)
        .ok_or(BillingError::MathOverflow)?;
    platform_config.total_revenue = platform_config.total_revenue
        .checked_add(platform_fee)
        .ok_or(BillingError::MathOverflow)?;

    emit!(SubscriptionCreated {
        user: ctx.accounts.user.key(),
        plan: ctx.accounts.subscription_plan.key(),
        amount_paid: total_amount,
        expires_at: subscription.expires_at,
        timestamp: clock.unix_timestamp,
    });

    msg!("User subscribed to plan for {} periods, paid: {}", periods_to_pay, total_amount);
    
    Ok(())
}