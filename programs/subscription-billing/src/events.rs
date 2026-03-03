use anchor_lang::prelude::*;

#[event]
pub struct PlanCreated {
    pub plan_id: u64,
    pub admin: Pubkey,
    pub name: String,
    pub price_per_period: u64,
    pub period_duration: i64,
    pub timestamp: i64,
}

#[event]
pub struct SubscriptionCreated {
    pub user: Pubkey,
    pub plan: Pubkey,
    pub amount_paid: u64,
    pub expires_at: i64,
    pub timestamp: i64,
}

#[event]
pub struct SubscriptionRenewed {
    pub user: Pubkey,
    pub plan: Pubkey,
    pub amount_paid: u64,
    pub new_expiry: i64,
    pub timestamp: i64,
}

#[event]
pub struct SubscriptionCancelled {
    pub user: Pubkey,
    pub plan: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct PlatformFeesWithdrawn {
    pub admin: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}