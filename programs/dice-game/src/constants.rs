use anchor_lang::prelude::*;

#[constant]
pub const SEED: &str = "anchor";

pub const MINI_BET_LAMPORTS: u64 = 10_000_000;

pub const MINI_ROLL: u8 = 1;
pub const MAX_ROLL: u8 = 99;
pub const HOUSE_EDGE_BPS: u16 = 150;
