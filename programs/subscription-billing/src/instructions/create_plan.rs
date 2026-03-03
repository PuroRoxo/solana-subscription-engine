use anchor_lang::prelude::*;
use crate::{
    state::{PlatformConfig, SubscriptionPlan}, 
    errors::BillingError,
    constants::{MIN_PERIOD_DURATION, MAX_PERIOD_DURATION},
    events::PlanCreated
};

#[derive(Accounts)]
#[instruction(plan_id: u64)]
pub struct CreatePlan<'info> {
    #[account(
        mut,
        seeds = [PlatformConfig::SEED_PREFIX],
        bump = platform_config.bump,
        has_one = admin
    )]
    pub platform_config: Account<'info, PlatformConfig>,
    
    #[account(
        init,
        payer = admin,
        space = SubscriptionPlan::LEN,
        seeds = [SubscriptionPlan::SEED_PREFIX, plan_id.to_le_bytes().as_ref()],
        bump
    )]
    pub subscription_plan: Account<'info, SubscriptionPlan>,
    
    #[account(mut)]
    pub admin: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn create_plan(
    ctx: Context<CreatePlan>,
    plan_id: u64,
    name: String,
    price_per_period: u64,
    period_duration: i64,
    max_subscribers: Option<u64>,
) -> Result<()> {
    // Validation
    SubscriptionPlan::validate_name(&name)?;
    
    require!(
        price_per_period > 0,
        BillingError::InvalidPaymentAmount
    );
    
    require!(
        period_duration >= MIN_PERIOD_DURATION && period_duration <= MAX_PERIOD_DURATION,
        BillingError::InvalidPeriodDuration
    );

    let plan = &mut ctx.accounts.subscription_plan;
    plan.plan_id = plan_id;
    plan.admin = ctx.accounts.admin.key();
    plan.name = name.clone();
    plan.price_per_period = price_per_period;
    plan.period_duration = period_duration;
    plan.max_subscribers = max_subscribers;
    plan.current_subscribers = 0;
    plan.total_revenue = 0;
    plan.active = true;
    plan.created_at = Clock::get()?.unix_timestamp;
    plan.bump = ctx.bumps.subscription_plan;

    // Update platform stats
    let platform_config = &mut ctx.accounts.platform_config;
    platform_config.total_plans = platform_config.total_plans
        .checked_add(1)
        .ok_or(BillingError::MathOverflow)?;

    emit!(PlanCreated {
        plan_id,
        admin: ctx.accounts.admin.key(),
        name,
        price_per_period,
        period_duration,
        timestamp: Clock::get()?.unix_timestamp,
    });

    msg!("Plan created: {} with price: {}", plan.name, price_per_period);
    
    Ok(())
}