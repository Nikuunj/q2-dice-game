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
    #[msg("Math error")]
    MathError,
    #[msg("Invalid data length")]
    Ed25519DataLength,
    #[msg("Invalid Header")]
    Ed25519Header,
    #[msg("Signature offset wrong")]
    Ed25519SignatureOffset,
    #[msg("Signature must be one")]
    Ed25519SignatureMustBeOne,
    #[msg("Ed25519 message wrong")]
    Ed25519Message,
    #[msg("Ed25519 pubkey is wrong")]
    Ed25519Pubkey,
    #[msg("Ed25519 acocunt is wrong")]
    Ed25519Accounts,
    #[msg("Ed25519 program is wrong")]
    Ed25519Program,
    #[msg("Ed25519 signature Invalid")]
    Ed25519Signature,
}
