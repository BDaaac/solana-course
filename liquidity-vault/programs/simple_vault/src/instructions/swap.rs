use anchor_lang::prelude::*;
use anchor_spl::memo::Memo;
use anchor_spl::token::Token;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{Mint, TokenAccount};
use raydium_clmm_cpi::{
    cpi,
    states::{AmmConfig, ObservationState, PoolState},
};

use crate::VaultState;

/// Неделя 12. Vault обменивает токены через CLMM-пул Raydium.
/// Запускать может только admin; обмен подписывает PDA vault_state.
#[derive(Accounts)]
pub struct SwapViaRaydium<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        seeds = [b"vault", vault_state.token_mint.as_ref()],
        bump = vault_state.bump,
        constraint = vault_state.admin == admin.key() @ crate::VaultError::Unauthorized,
    )]
    pub vault_state: Account<'info, VaultState>,

    /// Откуда списываем (токен-аккаунт, принадлежащий vault PDA)
    #[account(mut, token::authority = vault_state)]
    pub input_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// Куда получаем (второе хранилище vault)
    #[account(mut, token::authority = vault_state)]
    pub output_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    pub amm_config: Box<Account<'info, AmmConfig>>,
    #[account(mut)]
    pub pool_state: AccountLoader<'info, PoolState>,
    #[account(mut)]
    pub input_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    pub output_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    pub observation_state: AccountLoader<'info, ObservationState>,

    pub input_vault_mint: Box<InterfaceAccount<'info, Mint>>,
    pub output_vault_mint: Box<InterfaceAccount<'info, Mint>>,

    /// CHECK: проверяется адресом
    #[account(address = raydium_clmm_cpi::id())]
    pub clmm_program: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub token_program_2022: Program<'info, Token2022>,
    pub memo_program: Program<'info, Memo>,
}

/// `min_amount_out` — защита от slippage: если пул отдаст меньше, Raydium откатит обмен.
pub fn handler<'a, 'b, 'c: 'info, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, SwapViaRaydium<'info>>,
    amount_in: u64,
    min_amount_out: u64,
    sqrt_price_limit_x64: u128,
) -> Result<()> {
    require!(amount_in > 0, crate::VaultError::ZeroAmount);
    require!(min_amount_out > 0, crate::VaultError::NoSlippageProtection);

    let remaining = ctx.remaining_accounts.to_vec();
    let mint = ctx.accounts.vault_state.token_mint;
    let bump = ctx.accounts.vault_state.bump;
    let seeds: &[&[&[u8]]] = &[&[b"vault", mint.as_ref(), &[bump]]];

    let accounts = cpi::accounts::SwapSingleV2 {
        payer: ctx.accounts.vault_state.to_account_info(),
        amm_config: ctx.accounts.amm_config.to_account_info(),
        pool_state: ctx.accounts.pool_state.to_account_info(),
        input_token_account: ctx.accounts.input_token_account.to_account_info(),
        output_token_account: ctx.accounts.output_token_account.to_account_info(),
        input_vault: ctx.accounts.input_vault.to_account_info(),
        output_vault: ctx.accounts.output_vault.to_account_info(),
        observation_state: ctx.accounts.observation_state.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
        token_program_2022: ctx.accounts.token_program_2022.to_account_info(),
        memo_program: ctx.accounts.memo_program.to_account_info(),
        input_vault_mint: ctx.accounts.input_vault_mint.to_account_info(),
        output_vault_mint: ctx.accounts.output_vault_mint.to_account_info(),
    };
    let cpi_ctx =
        CpiContext::new_with_signer(ctx.accounts.clmm_program.to_account_info(), accounts, seeds)
            .with_remaining_accounts(remaining);

    cpi::swap_v2(cpi_ctx, amount_in, min_amount_out, sqrt_price_limit_x64, true)?;
    msg!("Swap: in={} min_out={}", amount_in, min_amount_out);
    Ok(())
}
