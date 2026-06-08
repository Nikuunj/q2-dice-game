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

    pub fn deposit_with_introspection(
        ctx: Context<DepositWithIntrospection>,
        amount: u64,
        max_x: u64,
        max_y: u64,
    ) -> Result<()> {
        ctx.accounts.deposit_with_introseption(amount, max_x, max_y)
    }

    pub fn mint_lp(ctx: Context<MintLp>, amount: u64) -> Result<()> {
        ctx.accounts.mint_lp(amount)
    }
}
