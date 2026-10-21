use anchor_lang::prelude::*;

use crate::{constants::*, error::ErrorCode, state::Counter};

/// Общий набор аккаунтов для increment / decrement / reset.
/// `has_one = authority` + seeds гарантируют, что менять счётчик может только владелец.
#[derive(Accounts)]
pub struct Update<'info> {
    #[account(
        mut,
        seeds = [COUNTER_SEED, authority.key().as_ref()],
        bump = counter.bump,
        has_one = authority
    )]
    pub counter: Account<'info, Counter>,
    pub authority: Signer<'info>,
}

pub fn handle_increment(ctx: Context<Update>, step: u64) -> Result<()> {
    require!(step > 0, ErrorCode::ZeroStep);
    let counter = &mut ctx.accounts.counter;
    counter.count = counter.count.checked_add(step).ok_or(ErrorCode::CounterOverflow)?;
    bump_updates(counter)?;
    msg!("Counter +{} = {}", step, counter.count);
    Ok(())
}

pub fn handle_decrement(ctx: Context<Update>, step: u64) -> Result<()> {
    require!(step > 0, ErrorCode::ZeroStep);
    let counter = &mut ctx.accounts.counter;
    counter.count = counter.count.checked_sub(step).ok_or(ErrorCode::CounterUnderflow)?;
    bump_updates(counter)?;
    msg!("Counter -{} = {}", step, counter.count);
    Ok(())
}

pub fn handle_reset(ctx: Context<Update>) -> Result<()> {
    let counter = &mut ctx.accounts.counter;
    counter.count = 0;
    bump_updates(counter)?;
    msg!("Counter reset");
    Ok(())
}

fn bump_updates(counter: &mut Counter) -> Result<()> {
    counter.total_updates = counter.total_updates.checked_add(1).ok_or(ErrorCode::CounterOverflow)?;
    Ok(())
}
