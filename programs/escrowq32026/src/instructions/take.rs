use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        close_account, transfer_checked, CloseAccount, Mint, TokenAccount, TokenInterface,
        TransferChecked,
    },
};

use crate::{error::ErrorCode, Escrow, ESCROW_SEED};

#[derive(Accounts)]
pub struct Take<'info> {
    #[account(mut)]
    pub taker: Signer<'info>,

    #[account(mut)]
    pub maker: SystemAccount<'info>,

    #[account(
        mint::token_program = token_program
    )]
    pub mint_a: InterfaceAccount<'info, Mint>,

    #[account(
        mint::token_program = token_program
    )]
    pub mint_b: InterfaceAccount<'info, Mint>,

    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = mint_a,
        associated_token::authority = taker,
        associated_token::token_program = token_program,
    )]
    pub taker_ata_a: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = mint_b,
        associated_token::authority = taker,
        associated_token::token_program = token_program,
    )]
    pub taker_ata_b: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = mint_b,
        associated_token::authority = maker,
        associated_token::token_program = token_program,
    )]
    pub maker_ata_b: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        close = maker,
        has_one = maker,
        has_one = mint_a,
        has_one = mint_b,
        seeds = [
            ESCROW_SEED,
            maker.key().as_ref(),
            escrow.seed.to_le_bytes().as_ref()
        ],
        bump = escrow.bump
    )]
    pub escrow: Account<'info, Escrow>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
        associated_token::token_program = token_program
    )]
    pub vault: Box<InterfaceAccount<'info, TokenAccount>>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> Take<'info> {
    fn transfer_payment_to_maker(&self) -> Result<()> {
        let accounts = TransferChecked {
            from: self.taker_ata_b.to_account_info(),
            mint: self.mint_b.to_account_info(),
            to: self.maker_ata_b.to_account_info(),
            authority: self.taker.to_account_info(),
        };

        let context = CpiContext::new(self.token_program.key(), accounts);

        transfer_checked(context, self.escrow.receive, self.mint_b.decimals)
    }

    fn transfer_deposit_to_taker(&self) -> Result<()> {
        let maker_key = self.maker.key();
        let seed_bytes = self.escrow.seed.to_le_bytes();
        let bump = [self.escrow.bump];

        let signer_seeds: &[&[&[u8]]] = &[&[
            ESCROW_SEED,
            maker_key.as_ref(),
            seed_bytes.as_ref(),
            bump.as_ref(),
        ]];

        let accounts = TransferChecked {
            from: self.vault.to_account_info(),
            mint: self.mint_a.to_account_info(),
            to: self.taker_ata_a.to_account_info(),
            authority: self.escrow.to_account_info(),
        };

        let context = CpiContext::new_with_signer(self.token_program.key(), accounts, signer_seeds);

        transfer_checked(context, self.vault.amount, self.mint_a.decimals)
    }

    fn close_vault(&self) -> Result<()> {
        let maker_key = self.maker.key();
        let seed_bytes = self.escrow.seed.to_le_bytes();
        let bump = [self.escrow.bump];

        let signer_seeds: &[&[&[u8]]] = &[&[
            ESCROW_SEED,
            maker_key.as_ref(),
            seed_bytes.as_ref(),
            bump.as_ref(),
        ]];

        let accounts = CloseAccount {
            account: self.vault.to_account_info(),
            destination: self.maker.to_account_info(),
            authority: self.escrow.to_account_info(),
        };

        let context = CpiContext::new_with_signer(self.token_program.key(), accounts, signer_seeds);

        close_account(context)
    }

    pub fn exchange(&mut self) -> Result<()> {
        require!(
            Clock::get()?.unix_timestamp <= self.escrow.expiration,
            ErrorCode::EscrowExpired
        );

        self.transfer_payment_to_maker()?;
        self.transfer_deposit_to_taker()?;
        self.close_vault()
    }
}
