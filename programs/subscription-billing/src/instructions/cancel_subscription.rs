use anchor_lang::prelude::*;
use crate::{
    state::{SubscriptionPlan, UserSubscription, SubscriptionStatus},
    errors::BillingError,
    events::SubscriptionCancelled
};

#[derive(Accounts)]
pub struct CancelSubscription<'info> {
    #[account(
        mut,
        seeds = [SubscriptionPlan::SEED_PREFIX, subscription_plan.plan_id.to_le_bytes().as_ref()],
        bump = subscription_plan.bump
    )]
    pub subscription_plan: Account<'info, SubscriptionPlan>,
    
    #[account(
        mut,
        seeds = [UserSubscription::SEED_PREFIX, user.key().as_ref(), subscription_plan.key().as_ref()],
        bump = user_subscription.bump,
        has_one = user,
        constraint = user_subscription.status != SubscriptionStatus::Cancelled @ BillingError::AlreadyCancelled
    )]
    pub user_subscription: Account<'info, UserSubscription>,
    
    pub user: Signer<'info>,
}

pub fn cancel_subscription(ctx: Context<CancelSubscription>) -> Result<()> {
    let subscription = &mut ctx.accounts.user_subscription;
    subscription.status = SubscriptionStatus::Cancelled;
    subscription.auto_renew = false;

    // Update plan subscriber count
    let plan = &mut ctx.accounts.subscription_plan;
    plan.current_subscribers = plan.current_subscribers.saturating_sub(1);

    emit!(SubscriptionCancelled {
        user: ctx.accounts.user.key(),
        plan: ctx.accounts.subscription_plan.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });

    msg!("Subscription cancelled for user: {}", ctx.accounts.user.key());
    
    Ok(())
}