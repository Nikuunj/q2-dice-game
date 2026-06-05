pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("GKQtwjgpGFQXyEmFHxarsmDqpgbPwq9VoaVydVsv91Kn");

#[program]
pub mod dice_game {
    use super::*;
}
