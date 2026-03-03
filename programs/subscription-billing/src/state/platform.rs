use anchor_lang::prelude::*;

#[account]
pub struct PlatformConfig {
    pub admin: Pubkey,
    pub platform_fee_bps: u16, // basis points (100 = 1%)
    pub treasury: Pubkey,
    pub total_plans: u64,
    pub total_subscriptions: u64,
    pub total_revenue: u64,
    pub bump: u8,
}

impl PlatformConfig {
    pub const LEN: usize = 32 + 2 + 32 + 8 + 8 + 8 + 1 + 8; // +8 for discriminator

    pub const SEED_PREFIX: &'static [u8] = b"platform";
}