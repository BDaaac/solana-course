use anchor_lang::prelude::*;
use anchor_spl::memo::Memo;
use anchor_spl::token::Token;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{Mint, TokenAccount};
use raydium_clmm_cpi::{
    cpi,
    states::{PersonalPositionState, PoolState, TickArrayState},
};

use crate::VaultState;

/// Неделя 13. Сбор комиссий: decrease_liquidity_v2 с liquidity = 0
/// не уменьшает позицию, а переводит накопленные комиссии на счета vault.
#[derive(Accounts)]
pub struct CollectFees<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        seeds = [b"vault", vault_state.token_mint.as_ref()],
        bump = vault_state.bump,
        constraint = vault_state.admin == admin.key() @ crate::VaultError::Unauthorized,
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(mut, token::authority = vault_state)]
    pub recipient_token_account_0: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut, token::authority = vault_state)]
    pub recipient_token_account_1: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(mut)]
    pub pool_state: AccountLoader<'info, PoolState>,

    #[account(
        token::mint = personal_position.nft_mint,
        token::authority = vault_state,
    )]
    pub position_nft_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(mut, constraint = personal_position.pool_id == pool_state.key())]
    pub personal_position: Box<Account<'info, PersonalPositionState>>,

    #[account(mut)]
    pub token_vault_0: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    pub token_vault_1: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    pub tick_array_lower: AccountLoader<'info, TickArrayState>,
    #[account(mut)]
    pub tick_array_upper: AccountLoader<'info, TickArrayState>,

    pub vault_0_mint: Box<InterfaceAccount<'info, Mint>>,
    pub vault_1_mint: Box<InterfaceAccount<'info, Mint>>,

    /// CHECK: проверяется адресом
    #[account(address = raydium_clmm_cpi::id())]
    pub clmm_program: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub token_program_2022: Program<'info, Token2022>,
    pub memo_program: Program<'info, Memo>,
}

pub fn handler<'a, 'b, 'c: 'info, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, CollectFees<'info>>,
) -> Result<()> {
    let remaining = ctx.remaining_accounts.to_vec();
    let mint = ctx.accounts.vault_state.token_mint;
    let bump = ctx.accounts.vault_state.bump;
    let seeds: &[&[&[u8]]] = &[&[b"vault", mint.as_ref(), &[bump]]];

    let accounts = cpi::accounts::DecreaseLiquidityV2 {
        nft_owner: ctx.accounts.vault_state.to_account_info(),
        nft_account: ctx.accounts.position_nft_account.to_account_info(),
        personal_position: ctx.accounts.personal_position.to_account_info(),
        pool_state: ctx.accounts.pool_state.to_account_info(),
        protocol_position: ctx.accounts.personal_position.to_account_info(),
        token_vault_0: ctx.accounts.token_vault_0.to_account_info(),
        token_vault_1: ctx.accounts.token_vault_1.to_account_info(),
        tick_array_lower: ctx.accounts.tick_array_lower.to_account_info(),
        tick_array_upper: ctx.accounts.tick_array_upper.to_account_info(),
        recipient_token_account_0: ctx.accounts.recipient_token_account_0.to_account_info(),
        recipient_token_account_1: ctx.accounts.recipient_token_account_1.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
        token_program_2022: ctx.accounts.token_program_2022.to_account_info(),
        memo_program: ctx.accounts.memo_program.to_account_info(),
        vault_0_mint: ctx.accounts.vault_0_mint.to_account_info(),
        vault_1_mint: ctx.accounts.vault_1_mint.to_account_info(),
    };
    let cpi_ctx =
        CpiContext::new_with_signer(ctx.accounts.clmm_program.to_account_info(), accounts, seeds)
            .with_remaining_accounts(remaining);

    cpi::decrease_liquidity_v2(cpi_ctx, 0, 0, 0)?;
    msg!("Fees collected");
    Ok(())
}
