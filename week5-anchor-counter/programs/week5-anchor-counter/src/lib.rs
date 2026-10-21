//! Неделя 5: Counter Program на Anchor.
pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("7nnPuXhhqA7S9CwG3zu5qLapAys3fRViLWx8MLYRHryv");

#[program]
pub mod week5_anchor_counter {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        instructions::initialize::handle_initialize(ctx)
    }

    pub fn increment(ctx: Context<Update>, step: u64) -> Result<()> {
        instructions::update::handle_increment(ctx, step)
    }

    pub fn decrement(ctx: Context<Update>, step: u64) -> Result<()> {
        instructions::update::handle_decrement(ctx, step)
    }

    pub fn reset(ctx: Context<Update>) -> Result<()> {
        instructions::update::handle_reset(ctx)
    }
}
