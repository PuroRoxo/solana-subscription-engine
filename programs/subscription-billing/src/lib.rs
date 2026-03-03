use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, Token, TokenAccount, Transfer},
};

pub mod instructions;
pub mod state;
pub mod errors;
pub mod constants;
pub mod events;

use instructions::*;
use errors::*;
use state::*;

declare_id!("Sub1scr1pt1onB1ll1ngSyst3mOnCha1nPr0gram11");

#[cfg(not(feature = "no-entrypoint"))]
solana_security_txt::security_txt! {
    name: "Subscription Billing System",
    project_url: "https://github.com/superteam-subscription-billing",
    contacts: "email:security@superteam.fun",
    policy: "https://github.com/superteam-subscription-billing/blob/main/SECURITY.md",
    source_code: "https://github.com/superteam-subscription-billing"
}

#[program]
pub mod subscription_billing {
    use super::*;

    /// Initialize the platform configuration
    pub fn initialize_platform(
        ctx: Context<InitializePlatform>,
        platform_fee_bps: u16,
    ) -> Result<()> {
        instructions::initialize_platform(ctx, platform_fee_bps)
    }

    /// Create a new subscription plan
    pub fn create_plan(
        ctx: Context<CreatePlan>,
        plan_id: u64,
        name: String,
        price_per_period: u64,
        period_duration: i64,
        max_subscribers: Option<u64>,
    ) -> Result<()> {
        instructions::create_plan(ctx, plan_id, name, price_per_period, period_duration, max_subscribers)
    }

    /// Subscribe to a plan
    pub fn subscribe(
        ctx: Context<Subscribe>,
        periods_to_pay: u8,
    ) -> Result<()> {
        instructions::subscribe(ctx, periods_to_pay)
    }

    /// Process subscription renewal payment
    pub fn process_renewal(ctx: Context<ProcessRenewal>) -> Result<()> {
        instructions::process_renewal(ctx)
    }

    /// Cancel subscription
    pub fn cancel_subscription(ctx: Context<CancelSubscription>) -> Result<()> {
        instructions::cancel_subscription(ctx)
    }

    /// Update plan details (admin only)
    pub fn update_plan(
        ctx: Context<UpdatePlan>,
        new_price: Option<u64>,
        new_max_subscribers: Option<u64>,
        active: Option<bool>,
    ) -> Result<()> {
        instructions::update_plan(ctx, new_price, new_max_subscribers, active)
    }

    /// Withdraw platform fees (admin only)
    pub fn withdraw_fees(
        ctx: Context<WithdrawFees>,
        amount: u64,
    ) -> Result<()> {
        instructions::withdraw_fees(ctx, amount)
    }
}