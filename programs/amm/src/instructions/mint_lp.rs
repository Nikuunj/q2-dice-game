use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{mint_to, Mint, MintTo, TokenAccount, TokenInterface},
};
use solana_instructions_sysvar::get_instruction_relative;

use crate::{Config, error::AmmErrorCode, instruction::DepositWithIntrospection};

#[derive(Accounts)]
pub struct MintLp<'info> {
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
        init_if_needed,
        payer = user,
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

impl<'info> MintLp<'info> {
    pub fn mint_lp(&mut self, amount: u64) -> Result<()> {
        require!(amount > 0, AmmErrorCode::InvalidAmount);
        require!(!self.config.locked, AmmErrorCode::PoolLocked);

        self.verify_deposit(amount)?;
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
            CpiContext::new_with_signer(self.token_program_lp.key(), cpi_acc, signer_seeds);

        mint_to(cpi_ctx, amount)
    }

    pub fn verify_deposit(&self, amount: u64) -> Result<()> {
        let ix = get_instruction_relative(-1, &self.instruction_sysvar.to_account_info())
            .map_err(|_| error!(AmmErrorCode::MissingPriorInstruction))?;

        require!(
            &ix.data[..8]
                == DepositWithIntrospection::DISCRIMINATOR,
            AmmErrorCode::UnexpectedDiscriminator
        );

        require_eq!(ix.program_id, crate::ID, AmmErrorCode::InvalidProgramId);

        require_eq!(ix.data.len(), 32, AmmErrorCode::InvalidDataLength);

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
        require!(accounts.len() >= 12, AmmErrorCode::InvalidAccountsLength);

        let expected_accounts: [Pubkey; 4] = [
            self.user.key(),
            self.config.mint_x,
            self.config.mint_y,
            self.config.key(),
        ];

        for (index, key) in expected_accounts.iter().enumerate() {
            require_keys_eq!(accounts[index].pubkey, *key, AmmErrorCode::InvalidKey);
        }

        Ok(())
    }
}
