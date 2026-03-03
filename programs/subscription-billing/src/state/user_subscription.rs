use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum SubscriptionStatus {
    Active,
    Expired,
    Cancelled,
}

#[account]
pub struct UserSubscription {
    pub user: Pubkey,
    pub plan: Pubkey,
    pub status: SubscriptionStatus,
    pub subscribed_at: i64,
    pub expires_at: i64,
    pub last_payment_at: i64,
    pub total_paid: u64,
    pub payment_token: Pubkey,
    pub auto_renew: bool,
    pub periods_paid: u64,
    pub bump: u8,
}

impl UserSubscription {
    pub const LEN: usize = 32 + 32 + 1 + 8 + 8 + 8 + 8 + 32 + 1 + 8 + 1 + 8; // +8 for discriminator

    pub const SEED_PREFIX: &'static [u8] = b"subscription";

    pub fn is_active(&self) -> bool {
        self.status == SubscriptionStatus::Active && 
        Clock::get().unwrap().unix_timestamp <= self.expires_at
    }

    pub fn is_expired(&self) -> bool {
        Clock::get().unwrap().unix_timestamp > self.expires_at
    }

    pub fn extend_subscription(&mut self, periods: u8, period_duration: i64, amount_paid: u64) -> Result<()> {
        let additional_time = period_duration
            .checked_mul(periods as i64)
            .ok_or(crate::errors::BillingError::MathOverflow)?;

        self.expires_at = self.expires_at
            .checked_add(additional_time)
            .ok_or(crate::errors::BillingError::MathOverflow)?;

        self.total_paid = self.total_paid
            .checked_add(amount_paid)
            .ok_or(crate::errors::BillingError::MathOverflow)?;

        self.periods_paid = self.periods_paid
            .checked_add(periods as u64)
            .ok_or(crate::errors::BillingError::MathOverflow)?;

        self.last_payment_at = Clock::get()?.unix_timestamp;
        self.status = SubscriptionStatus::Active;

        Ok(())
    }
}