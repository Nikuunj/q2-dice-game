use anchor_lang::prelude::*;

#[error_code]
pub enum AmmErrorCode {
    #[msg("Custom error message")]
    CustomError,
}
