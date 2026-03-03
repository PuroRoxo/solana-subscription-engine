use anchor_lang::prelude::*;

#[account]
pub struct SubscriptionPlan {
    pub plan_id: u64,
    pub admin: Pubkey,
    pub name: String, // Max 50 chars
    pub price_per_period: u64,
    pub period_duration: i64, // seconds
    pub max_subscribers: Option<u64>,
    pub current_subscribers: u64,
    pub total_revenue: u64,
    pub active: bool,
    pub created_at: i64,
    pub bump: u8,
}

impl SubscriptionPlan {
    pub const LEN: usize = 8 + 32 + (4 + 50) + 8 + 8 + (1 + 8) + 8 + 8 + 1 + 8 + 1 + 8; // +8 for discriminator

    pub const SEED_PREFIX: &'static [u8] = b"plan";
    pub const MAX_NAME_LENGTH: usize = 50;

    pub fn is_at_capacity(&self) -> bool {
        if let Some(max) = self.max_subscribers {
            self.current_subscribers >= max
        } else {
            false
        }
    }

    pub fn validate_name(name: &str) -> Result<()> {
        require!(
            !name.is_empty() && name.len() <= Self::MAX_NAME_LENGTH,
            crate::errors::BillingError::InvalidPlanName
        );
        Ok(())
    }
}