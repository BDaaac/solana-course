use anchor_lang::prelude::*;

/// Сид PDA счётчика: ["counter", authority]. У каждого пользователя свой счётчик.
#[constant]
pub const COUNTER_SEED: &[u8] = b"counter";
