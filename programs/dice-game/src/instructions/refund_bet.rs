use anchor_lang::{prelude::*, system_program::{transfer, Transfer}};

use crate::state::Bet;

#[derive(Accounts)]
pub struct RefundBet<'info> {
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
        mut, 
        has_one = player,
        close = player,
        seeds = [b"bet", vault.key().as_ref(), player.key().as_ref(), bet.seed.to_le_bytes().as_ref()],
        bump = bet.bump
    )]
    pub bet: Account<'info, Bet>,

    pub system_program: Program<'info, System>,
}

impl<'info> RefundBet<'info> {
    pub fn refund_bet(&mut self, bumps: &RefundBetBumps) -> Result<()> {
        let house_key =  self.house.key();
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"vault",
            house_key.as_ref(),
            &[bumps.vault]
        ]];

        transfer(CpiContext::new_with_signer(self.system_program.key(), Transfer {
            from: self.vault.to_account_info(),
            to: self.player.to_account_info()
        }, signer_seeds), self.bet.amount)
    }
}
