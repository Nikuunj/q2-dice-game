use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};
use constant_product_curve::ConstantProduct;
use solana_instructions_sysvar::get_instruction_relative;

use crate::{error::AmmErrorCode, instruction::BurnLp, Config};

#[derive(Accounts)]
pub struct WithdrawWithIntrospection<'info> {
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

    /// CHECK: This should be safe
    #[account(
        address= solana_sdk_ids::sysvar::instructions::ID
    )]
    pub instruction_sysvar: UncheckedAccount<'info>,

    pub associated_token_program: Program<'info, AssociatedToken>,

    pub token_program_y: Interface<'info, TokenInterface>,
    pub token_program_x: Interface<'info, TokenInterface>,
    pub token_program_lp: Interface<'info, TokenInterface>,
}

impl<'info> WithdrawWithIntrospection<'info> {
    pub fn withdraw_with_introseption(
        &mut self,
        amount: u64,
        min_x: u64,
        min_y: u64,
    ) -> Result<()> {
        require!(amount > 0, AmmErrorCode::InvalidAmount);
        require!(!self.config.locked, AmmErrorCode::PoolLocked);
        
        self.verify_burn(amount)?;

        let amounts = ConstantProduct::xy_withdraw_amounts_from_l(
            self.vault_x.amount,
            self.vault_y.amount,
            self.mint_lp.supply,
            amount,
            6,
        )
        .unwrap();


        let (x, y) = (amounts.x, amounts.y);

        require!(x >= min_x && y >= min_y, AmmErrorCode::InvalidAmount);

        self.withdraw_token(true, x)?;
        self.withdraw_token(false, y)
    }

    fn withdraw_token(&self, is_x: bool, amount: u64) -> Result<()> {
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

        let cpi_acc = TransferChecked {
            from,
            to,
            mint,
            authority: self.user.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(token_program.key(), cpi_acc);

        transfer_checked(cpi_ctx, amount, decimals)
    }

    fn verify_burn(&self, amount: u64) -> Result<()> {
        let ix = get_instruction_relative(1, &self.instruction_sysvar.to_account_info())
            .map_err(|_| error!(AmmErrorCode::MissingPriorInstruction))?;

        require!(
            &ix.data[..8] == BurnLp::DISCRIMINATOR,
            AmmErrorCode::UnexpectedDiscriminator
        );

        require_eq!(ix.program_id, crate::ID, AmmErrorCode::InvalidProgramId);

        require_eq!(ix.data.len(), 16, AmmErrorCode::InvalidDataLength);

        self.confirm_accounts(ix.accounts)?;

        let mint_amount = u64::from_le_bytes(
            ix.data[8..16]
                .try_into()
                .map_err(|_| AmmErrorCode::InvalidDataLength)?,
        );
        require_eq!(mint_amount, amount, AmmErrorCode::InvalidAmount);

        Ok(())
    }

    fn confirm_accounts(&self, accounts: Vec<AccountMeta>) -> Result<()> {
        require!(accounts.len() == 8, AmmErrorCode::InvalidAccountsLength);

        let expected_accounts: [Pubkey; 3] =
            [self.user.key(), self.config.key(), self.mint_lp.key()];

        for (index, key) in expected_accounts.iter().enumerate() {
            require_keys_eq!(accounts[index].pubkey, *key, AmmErrorCode::InvalidKey);
        }
        Ok(())
    }
}
