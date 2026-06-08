use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface::{Mint, TokenAccount, TokenInterface}};

use crate::state::Config;

#[derive(Accounts)]
#[instruction( seed: u64 )]
pub struct Initialize<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,

    #[account(mint::token_program = token_program_x)]
    pub mint_x: Box<InterfaceAccount<'info, Mint>>,
    #[account(mint::token_program = token_program_y)]
    pub mint_y: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        init,
        payer = initializer,
        seeds = [b"lp", config.key.as_ref()],
        bump,
        mint::decimals = 6,
        mint::authority = config,
        mint::token_program = token_program
        
    )]
    pub mint_lp: Box<InterfaceAccount<'info, Mint>>,

    
    #[account(
        init,
        payer= initializer,
        associated_token::mint= mint_x,
        associated_token::authority= config,
    )]
    pub vault_x: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init,
        payer= initializer,
        associated_token::mint= mint_y,
        associated_token::authority= config
    )]
    pub vault_y: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init,
        payer = initializer,
        seeds = [b"config", seed.to_le_bytes().as_ref()],
        space = Config::INIT_SPACE + Config::DISCRIMINATOR.len(),
        bump
    )]
    pub config: Box<Account<'info, Config>>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program_x: Interface<'info, TokenInterface>,
    pub token_program: Interface<'info, TokenInterface>,
    pub token_program_y: Interface<'info, TokenInterface>,
    pub system_program: Program<'info>

}

impl<'info> Initialize<'info> {
    pub fn init(
        &mut self,
        seed: u64,
        fee: u16,
        authority: Option<Pubkey>,
        bumps: InitializeBumps,
    ) -> Result<()> {
        self.config.set_inner(Config {
            seed,
            authority,
            mint_x: self.mint_x.key(),
            mint_y: self.mint_y.key(),
            fee,
            locked: false,
            config_bump: bumps.config,
            lp_bump: bumps.mint_lp,
        });

        Ok(())
    }
}