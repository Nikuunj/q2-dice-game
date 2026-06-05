use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Custom error message")]
    CustomError,
    #[msg("Amount less then minimum bet aomunt")]
    MinimumBet,
    #[msg("Minimum roll overlflow")]
    MinimumRoll,
    #[msg("Max Roll overlflow")]
    MaxRoll,
}
