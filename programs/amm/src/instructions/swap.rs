use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};
use constant_product_curve::{ConstantProduct, LiquidityPair};

use crate::{error::AmmErrorCode, state::Config};

#[derive(Accounts)]
pub struct Swap<'info>{
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
        mint::token_program = token_program_lp
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
        init_if_needed,
        payer = user,
        associated_token::mint = mint_x,
        associated_token::authority = user,
        associated_token::token_program = token_program_x
    )]
    pub user_x: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = mint_y,
        associated_token::authority = user,
        associated_token::token_program = token_program_y
    )]
    pub user_y: Box<InterfaceAccount<'info, TokenAccount>>,

    pub associated_token_program: Program<'info, AssociatedToken>,

    pub token_program_y: Interface<'info, TokenInterface>,
    pub token_program_x: Interface<'info, TokenInterface>,
    pub token_program_lp: Interface<'info, TokenInterface>,
    system_program: Program<'info, System>,
}

impl<'info> Swap<'info> {
    pub fn swap(&mut self, is_x: bool, amount: u64, min: u64) -> Result<()> {
        require!(amount > 0, AmmErrorCode::CustomError);

        let mut curve = ConstantProduct::init(
            self.vault_x.amount,
            self.vault_y.amount,
            self.mint_lp.supply,
            self.config.fee,
            Some(6),
        )
        .unwrap();

        let p = match is_x {
            true => LiquidityPair::X,
            false => LiquidityPair::Y,
        };

        let swap_results = curve
            .swap(p, amount, min)
            .map_err(|_| AmmErrorCode::CustomError)?;

        self.deposit_token(is_x, swap_results.deposit)?;
        self.withdraw_token(!is_x, swap_results.withdraw)
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

    pub fn withdraw_token(&self, is_x: bool, amount: u64) -> Result<()> {
        let (from, to, token_program, mint, decimals) = match is_x {
            true => (
                self.vault_x.to_account_info(),
                self.user_x.to_account_info(),
                self.token_program_x.to_account_info(),
                self.mint_x.to_account_info(),
                self.mint_x.decimals,
            ),
            false => (
                self.vault_y.to_account_info(),
                self.user_y.to_account_info(),
                self.token_program_y.to_account_info(),
                self.mint_y.to_account_info(),
                self.mint_y.decimals,
            ),
        };

        let signer_seeds: &[&[&[u8]]] = &[&[
            b"config",
            &self.config.seed.to_le_bytes(),
            &[self.config.config_bump],
        ]];

        let cpi_ctx = CpiContext::new_with_signer(
            token_program.key(),
            TransferChecked {
                authority: self.config.to_account_info(),
                to,
                from,
                mint,
            },
            signer_seeds,
        );

        transfer_checked(cpi_ctx, amount, decimals)
    }
}