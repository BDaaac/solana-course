use anchor_lang::prelude::*;

/// Данные аккаунта-счётчика. Anchor добавляет перед ними 8-байтовый discriminator.
#[account]
#[derive(InitSpace)]
pub struct Counter {
    pub authority: Pubkey,
    pub count: u64,
    /// Сколько раз счётчик меняли
    pub total_updates: u64,
    pub bump: u8,
}
