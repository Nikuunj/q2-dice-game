use anchor_lang::prelude::*;

#[error_code]
pub enum AmmErrorCode {
    #[msg("Custom error message")]
    CustomError,
    #[msg("Missing Prior instruction")]
    MissingPriorInstruction,
    #[msg("Invalid data len")]
    InvalidDataLength,
    #[msg("Invalid amount")]
    InvalidAmount,
    #[msg("Invalid program id")]
    InvalidProgramId,
    #[msg("Pool is lock, please contract auther")]
    PoolLocked,
    #[msg("Invalid key")]
    InvalidKey
}
