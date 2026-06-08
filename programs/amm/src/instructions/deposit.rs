use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        mint_to, transfer_checked, Mint, MintTo, TokenAccount, TokenInterface, TransferChecked,
    },
};

use constant_product_curve::ConstantProduct;

use crate::{error::AmmErrorCode, state::Config};

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mint::token_program = token_program_x)]
    pub mint_x: Box<InterfaceAccount<'info, Mint>>,
    #[account(mint::token_program = token_program_y)]
    pub mint_y: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        has_one = mint_y,
        has_one = mint_x,
        seeds = [b"config", config.seed.to_le_bytes().as_ref()],
        bump = config.config_bump,
    )]
    pub config: Box<Account<'info, Config>>,

    #[account(
        mut,
        seeds = [b"lp", config.key().as_ref()],
        bump = config.lp_bump,
        mint::authority = config,
        mint::token_program = token_program
    )]
    pub mint_lp: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = config,
        associated_token::token_program = token_program_x
    )]
    pub vault_x: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = config,
        associated_token::token_program = token_program_y
    )]
    pub vault_y: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = user,
        associated_token::token_program = token_program_x
    )]
    pub user_x: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = user,
        associated_token::token_program = token_program_y
    )]
    pub user_y: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = mint_lp,
        associated_token::authority = user,
        associated_token::token_program = token_program
    )]
    pub user_lp: Box<InterfaceAccount<'info, TokenAccount>>,

    pub associated_token_program: Program<'info, AssociatedToken>,

    pub token_program_y: Interface<'info, TokenInterface>,
    pub token_program_x: Interface<'info, TokenInterface>,
    pub token_program: Interface<'info, TokenInterface>,
    system_program: Program<'info, System>,
}

impl<'info> Deposit<'info> {
    pub fn deposit(&mut self, amount: u64, max_x: u64, max_y: u64) -> Result<()> {
        require!(!self.config.locked, AmmErrorCode::CustomError);
        require_neq!(amount, 0, AmmErrorCode::CustomError);

        let (x, y) =
            if self.mint_lp.supply == 0 && self.vault_x.amount == 0 && self.vault_y.amount == 0 {
                (max_x, max_y)
            } else {
                let amounts = ConstantProduct::xy_deposit_amounts_from_l(
                    self.vault_x.amount,
                    self.vault_y.amount,
                    self.mint_lp.supply,
                    amount,
                    6,
                )
                .unwrap();

                require!(
                    amounts.x <= max_x && amounts.y <= max_y,
                    AmmErrorCode::CustomError
                );

                (amounts.x, amounts.y)
            };

        self.deposit_token(true, x).unwrap();
        self.deposit_token(false, y).unwrap();
        self.mint_lp_token(amount)
    }

    fn deposit_token(&self, is_x: bool, amount: u64) -> Result<()> {
        let (from, to, token_program, mint, decimals) = match is_x {
            true => (
                self.user_x.to_account_info(),
                self.vault_x.to_account_info(),
                self.token_program_x.to_account_info(),
                self.mint_x.to_account_info(),
                self.mint_x.decimals,
            ),
            false => (
                self.user_y.to_account_info(),
                self.vault_y.to_account_info(),
                self.token_program_y.to_account_info(),
                self.mint_y.to_account_info(),
                self.mint_y.decimals,
            ),
        };

        let cpi_acc = TransferChecked {
            from,
            to,
            mint,
            authority: self.user.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(token_program.key(), cpi_acc);

        transfer_checked(cpi_ctx, amount, decimals)
    }

    fn mint_lp_token(&self, amount: u64) -> Result<()> {
        let cpi_acc = MintTo {
            mint: self.mint_lp.to_account_info(),
            to: self.user_lp.to_account_info(),
            authority: self.config.to_account_info(),
        };

        let signer_seeds: &[&[&[u8]]] = &[&[
            b"config",
            &self.config.seed.to_le_bytes(),
            &[self.config.config_bump],
        ]];

        let cpi_ctx =
            CpiContext::new_with_signer(self.token_program.key(), cpi_acc, signer_seeds);

        mint_to(cpi_ctx, amount)
    }
}
