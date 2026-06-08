use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Burn, Mint, TokenAccount, TokenInterface, burn},
};
use solana_instructions_sysvar::get_instruction_relative;

use crate::{Config, error::AmmErrorCode, instruction::WithdrawWithIntrospection};

#[derive(Accounts)]
pub struct BurnLp<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
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
        associated_token::mint = mint_lp,
        associated_token::authority = user,
        associated_token::token_program = token_program_lp
    )]
    pub user_lp: Box<InterfaceAccount<'info, TokenAccount>>,

    /// CHECK: This should be safe
    #[account(
        address= solana_sdk_ids::sysvar::instructions::ID
    )]
    pub instruction_sysvar: UncheckedAccount<'info>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program_lp: Interface<'info, TokenInterface>,
    system_program: Program<'info, System>,
}

impl<'info> BurnLp<'info> {
    pub fn burn_lp(&mut self, amount: u64) -> Result<()> {
        require!(amount > 0, AmmErrorCode::InvalidAmount);
        require!(!self.config.locked, AmmErrorCode::PoolLocked);

        self.verify_withdraw(amount)?;
        let cpi_acc = Burn {
            mint: self.mint_lp.to_account_info(),
            from: self.user_lp.to_account_info(),
            authority: self.config.to_account_info(),
        };

        let signer_seeds: &[&[&[u8]]] = &[&[
            b"config",
            &self.config.seed.to_le_bytes(),
            &[self.config.config_bump],
        ]];

        let cpi_ctx =
            CpiContext::new_with_signer(self.token_program_lp.key(), cpi_acc, signer_seeds);

        burn(cpi_ctx, amount)
    }

    pub fn verify_withdraw(&self, amount: u64) -> Result<()> {
        let ix = get_instruction_relative(-1, &self.instruction_sysvar.to_account_info())
            .map_err(|_| error!(AmmErrorCode::MissingPriorInstruction))?;

        require!(
            &ix.data[..8]
                == WithdrawWithIntrospection::DISCRIMINATOR,
            AmmErrorCode::UnexpectedDiscriminator
        );

        require_eq!(ix.program_id, crate::ID, AmmErrorCode::InvalidProgramId);

        require_eq!(ix.data.len(), 32, AmmErrorCode::InvalidDataLength);

        self.confirm_accounts(ix.accounts)?;

        let burn_amount = u64::from_le_bytes(
            ix.data[8..16]
                .try_into()
                .map_err(|_| AmmErrorCode::InvalidDataLength)?,
        );
        require_eq!(burn_amount, amount, AmmErrorCode::InvalidAmount);

        Ok(())
    }

    fn confirm_accounts(&self, accounts: Vec<AccountMeta>) -> Result<()> {
        require!(accounts.len() == 14, AmmErrorCode::InvalidAccountsLength);

        let expected_accounts: [Pubkey; 5] = [
            self.user.key(),
            self.config.mint_x,
            self.config.mint_y,
            self.config.key(),
            self.mint_lp.key()
        ];

        for (index, key) in expected_accounts.iter().enumerate() {
            require_keys_eq!(accounts[index].pubkey, *key, AmmErrorCode::InvalidKey);
        }

        Ok(())
    }
}
