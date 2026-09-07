use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("The escrow has expired")]
    EscrowExpired,

    #[msg("The requested receive amount must be greater than zero")]
    InvalidReceiveAmount,

    #[msg("The expiration must be in the future")]
    InvalidExpiration,
}
