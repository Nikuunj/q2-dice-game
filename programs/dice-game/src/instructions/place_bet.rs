use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};

use crate::{
    constants::{MAX_ROLL, MINI_BET_LAMPORTS, MINI_ROLL},
    error::ErrorCode,
    state::Bet,
};

#[derive(Accounts)]
#[instruction(seed: u128)]
pub struct PlaceBet<'info> {
    #[account(mut)]
    pub player: Signer<'info>,

    /// CHECK: Validated with seeds
    pub house: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"vault", house.key().as_ref()],
        bump
    )]
    pub vault: SystemAccount<'info>,

    #[account(
        init,
        payer = player,
        seeds = [b"bet", vault.key().as_ref(), player.key().as_ref(), seed.to_le_bytes().as_ref()],
        space = Bet::INIT_SPACE + Bet::DISCRIMINATOR.len(),
        bump
    )]
    pub bet: Account<'info, Bet>,

    pub system_program: Program<'info, System>,
}

impl<'info> PlaceBet<'info> {
    pub fn place_bet(
        &mut self,
        seed: u128,
        roll: u8,
        amount: u64,
        bumps: &PlaceBetBumps,
    ) -> Result<()> {
        require!(amount >= MINI_BET_LAMPORTS, ErrorCode::MinimumBet);
        require!(roll >= MINI_ROLL, ErrorCode::MinimumRoll);
        require!(roll <= MAX_ROLL, ErrorCode::MaxRoll);

        self.bet.set_inner(Bet {
            player: self.player.key(),
            seed,
            slot: Clock::get()?.slot,
            amount,
            roll,
            bump: bumps.bet,
        });
        Ok(())
    }

    pub fn deposit(&mut self, amount: u64) -> Result<()> {
        transfer(
            CpiContext::new(
                self.system_program.key(),
                Transfer {
                    from: self.player.to_account_info(),
                    to: self.vault.to_account_info(),
                },
            ),
            amount,
        )
    }
}
