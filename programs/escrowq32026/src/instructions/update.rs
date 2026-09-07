use anchor_lang::prelude::*;

use crate::constants::ESCROW_SEED;
use crate::error::ErrorCode;
use crate::Escrow;

#[derive(Accounts)]
pub struct Update<'info> {
    #[account()]
    pub maker: Signer<'info>,

    #[account(
        mut,
        has_one = maker,
        seeds = [
            ESCROW_SEED,
            maker.key().as_ref(),
            escrow.seed.to_le_bytes().as_ref(),
        ],
        bump = escrow.bump,
    )]
    pub escrow: Account<'info, Escrow>,
}

impl<'info> Update<'info> {
    pub fn update_terms(&mut self, receive: u64, expiration: i64) -> Result<()> {
        require!(receive > 0, ErrorCode::InvalidReceiveAmount);
        require!(
            expiration > Clock::get()?.unix_timestamp,
            ErrorCode::InvalidExpiration
        );

        self.escrow.receive = receive;
        self.escrow.expiration = expiration;

        Ok(())
    }
}
