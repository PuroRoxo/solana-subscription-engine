use anchor_lang::prelude::*;

#[error_code]
pub enum BillingError {
    #[msg("Invalid plan name: must be 1-50 characters")]
    InvalidPlanName,
    
    #[msg("Plan has reached maximum subscriber capacity")]
    PlanAtCapacity,
    
    #[msg("Plan is not active")]
    PlanNotActive,
    
    #[msg("Subscription is not active")]
    SubscriptionNotActive,
    
    #[msg("Subscription has expired")]
    SubscriptionExpired,
    
    #[msg("Invalid payment amount")]
    InvalidPaymentAmount,
    
    #[msg("Invalid period duration")]
    InvalidPeriodDuration,
    
    #[msg("Invalid number of periods")]
    InvalidPeriods,
    
    #[msg("Unauthorized: only admin can perform this action")]
    Unauthorized,
    
    #[msg("Math overflow occurred")]
    MathOverflow,
    
    #[msg("Invalid platform fee: must be 0-1000 bps")]
    InvalidPlatformFee,
    
    #[msg("Insufficient payment for subscription")]
    InsufficientPayment,
    
    #[msg("Cannot cancel already cancelled subscription")]
    AlreadyCancelled,
    
    #[msg("Withdrawal amount exceeds available balance")]
    InsufficientBalance,
}