use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Counter overflow")]
    CounterOverflow,
    #[msg("Counter cannot go below zero")]
    CounterUnderflow,
    #[msg("Step must be greater than zero")]
    ZeroStep,
}
