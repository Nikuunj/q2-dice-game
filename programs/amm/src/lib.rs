pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("8NVA8HLgC9e3G4VgdfEAxPGeomNZgHFQGaAXDZsgYjHV");

#[program]
pub mod amm {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, seed: u64, fee: u16, authority: Option<Pubkey>) -> Result<()> {
        ctx.accounts.init(seed, fee, authority, ctx.bumps)
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64, max_x: u64, max_y: u64) -> Result<()> {
        ctx.accounts.deposit(amount, max_x, max_y)
    }
    pub fn deposit_with_introspection(
        ctx: Context<DepositWithIntrospection>,
        amount: u64,
        max_x: u64,
        max_y: u64,
    ) -> Result<()> {
        ctx.accounts.deposit_with_introseption(amount, max_x, max_y)
    }

    pub fn swap(ctx: Context<Swap>, is_x: bool, amount: u64, min: u64) -> Result<()> {
        ctx.accounts.swap(is_x, amount, min)
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64, min_x: u64, min_y: u64) -> Result<()> {
        ctx.accounts.withdraw(amount, min_x, min_y)
    }

    pub fn withdraw_with_introspection(
        ctx: Context<WithdrawWithIntrospection>,
        amount: u64,
        min_x: u64,
        min_y: u64,
    ) -> Result<()> {
        ctx.accounts.withdraw_with_introseption(amount, min_x, min_y)
    }

    pub fn burn_lp(ctx: Context<BurnLp>, amount: u64) -> Result<()> {
        ctx.accounts.burn_lp(amount)
    }
    pub fn mint_lp(ctx: Context<MintLp>, amount: u64) -> Result<()> {
        ctx.accounts.mint_lp(amount)
    }
}
