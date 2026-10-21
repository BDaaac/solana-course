use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Mint, MintTo, Token, TokenAccount, Transfer};

pub mod instructions;
use instructions::*;

declare_id!("8wmtpwX7UZbwEUhNFobRdMRmtyRsUgtWaFwV5GBff8J6");

#[program]
pub mod simple_vault {
    use super::*;

    // ───────────────────────── Неделя 8: каркас ─────────────────────────

    /// Создаёт VaultState, хранилище токенов и share-mint (все три — PDA).
    pub fn initialize_vault(ctx: Context<InitializeVault>) -> Result<()> {
        let vault = &mut ctx.accounts.vault_state;
        vault.admin = ctx.accounts.admin.key();
        vault.token_mint = ctx.accounts.token_mint.key();
        vault.vault_token_account = ctx.accounts.vault_token_account.key();
        vault.share_mint = ctx.accounts.share_mint.key();
        vault.total_tokens = 0;
        vault.total_shares = 0;
        vault.paused = false;
        vault.bump = ctx.bumps.vault_state;
        msg!("Vault initialized for mint: {}", vault.token_mint);
        Ok(())
    }

    /// Личная карточка пользователя: PDA ["user_card", vault, user].
    /// Если vault на паузе — карточку создать нельзя.
    pub fn create_user_card(ctx: Context<CreateUserCard>) -> Result<()> {
        require!(!ctx.accounts.vault_state.paused, VaultError::VaultPaused);
        let card = &mut ctx.accounts.user_card;
        card.vault = ctx.accounts.vault_state.key();
        card.owner = ctx.accounts.user.key();
        card.deposited = 0;
        card.created_at = Clock::get()?.unix_timestamp;
        card.bump = ctx.bumps.user_card;
        msg!("User card created for: {}", card.owner);
        Ok(())
    }

    /// Закрывает карточку и возвращает ренту владельцу (только если вклад выведен).
    pub fn close_user_card(_ctx: Context<CloseUserCard>) -> Result<()> {
        Ok(())
    }

    /// Админ ставит vault на паузу / снимает с паузы.
    pub fn set_paused(ctx: Context<AdminOnly>, paused: bool) -> Result<()> {
        ctx.accounts.vault_state.paused = paused;
        Ok(())
    }

    // ───────────────────────── Неделя 9: депозит ─────────────────────────

    /// shares = amount (первый вкладчик), далее amount * total_shares / total_tokens.
    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        require!(amount > 0, VaultError::ZeroAmount);
        require!(!ctx.accounts.vault_state.paused, VaultError::VaultPaused);

        let vault = &ctx.accounts.vault_state;
        let shares = if vault.total_shares == 0 || vault.total_tokens == 0 {
            amount
        } else {
            (amount as u128)
                .checked_mul(vault.total_shares as u128)
                .and_then(|v| v.checked_div(vault.total_tokens as u128))
                .and_then(|v| u64::try_from(v).ok())
                .ok_or(VaultError::MathOverflow)?
        };
        require!(shares > 0, VaultError::SharesTooSmall);

        // 1. Перевод токенов user -> vault: подписывает пользователь.
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_token_account.to_account_info(),
                    to: ctx.accounts.vault_token_account.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            amount,
        )?;

        // 2. Выпуск shares: подписывает PDA vault.
        let mint_key = ctx.accounts.vault_state.token_mint;
        let bump = [ctx.accounts.vault_state.bump];
        let seeds: &[&[u8]] = &[b"vault", mint_key.as_ref(), &bump];
        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.share_mint.to_account_info(),
                    to: ctx.accounts.user_share_account.to_account_info(),
                    authority: ctx.accounts.vault_state.to_account_info(),
                },
                &[seeds],
            ),
            shares,
        )?;

        // 3. Учёт.
        let vault = &mut ctx.accounts.vault_state;
        vault.total_tokens = vault.total_tokens.checked_add(amount).ok_or(VaultError::MathOverflow)?;
        vault.total_shares = vault.total_shares.checked_add(shares).ok_or(VaultError::MathOverflow)?;
        let card = &mut ctx.accounts.user_card;
        card.deposited = card.deposited.checked_add(amount).ok_or(VaultError::MathOverflow)?;

        msg!("Deposit: {} tokens -> {} shares", amount, shares);
        Ok(())
    }

    // ───────────────────────── Неделя 10: вывод ─────────────────────────

    /// tokens = shares * total_tokens / total_shares.
    /// Burn подписывает пользователь, transfer — PDA vault.
    pub fn withdraw(ctx: Context<Withdraw>, shares_amount: u64) -> Result<()> {
        require!(shares_amount > 0, VaultError::ZeroAmount);
        let vault = &ctx.accounts.vault_state;
        require!(vault.total_shares > 0, VaultError::VaultEmpty);

        let tokens_out = (shares_amount as u128)
            .checked_mul(vault.total_tokens as u128)
            .and_then(|v| v.checked_div(vault.total_shares as u128))
            .and_then(|v| u64::try_from(v).ok())
            .ok_or(VaultError::MathOverflow)?;
        require!(tokens_out > 0, VaultError::WithdrawTooSmall);
        // Пока часть средств в позиции, в хранилище может не хватать токенов.
        require!(
            ctx.accounts.vault_token_account.amount >= tokens_out,
            VaultError::InsufficientLiquidity
        );

        // 1. Burn shares: подписывает пользователь.
        token::burn(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Burn {
                    mint: ctx.accounts.share_mint.to_account_info(),
                    from: ctx.accounts.user_share_account.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            shares_amount,
        )?;

        // 2. Transfer vault -> user: подписывает PDA vault.
        let mint_key = ctx.accounts.vault_state.token_mint;
        let bump = [ctx.accounts.vault_state.bump];
        let seeds: &[&[u8]] = &[b"vault", mint_key.as_ref(), &bump];
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.vault_token_account.to_account_info(),
                    to: ctx.accounts.user_token_account.to_account_info(),
                    authority: ctx.accounts.vault_state.to_account_info(),
                },
                &[seeds],
            ),
            tokens_out,
        )?;

        let vault = &mut ctx.accounts.vault_state;
        vault.total_tokens = vault.total_tokens.checked_sub(tokens_out).ok_or(VaultError::MathOverflow)?;
        vault.total_shares = vault.total_shares.checked_sub(shares_amount).ok_or(VaultError::MathOverflow)?;
        let card = &mut ctx.accounts.user_card;
        card.deposited = card.deposited.saturating_sub(tokens_out);

        msg!("Withdraw: {} shares -> {} tokens", shares_amount, tokens_out);
        Ok(())
    }

    /// После закрытия позиции и обратного обмена весь капитал снова в базовом токене.
    /// Админ фиксирует новую сумму: total_tokens = реальный баланс хранилища.
    /// Так заработанные комиссии увеличивают цену одного share.
    pub fn sync_balance(ctx: Context<SyncBalance>) -> Result<()> {
        let vault = &mut ctx.accounts.vault_state;
        vault.total_tokens = ctx.accounts.vault_token_account.amount;
        msg!("Synced total_tokens={}", vault.total_tokens);
        Ok(())
    }

    // ───────────────────────── Неделя 12: swap ─────────────────────────

    pub fn swap_via_raydium<'a, 'b, 'c: 'info, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, SwapViaRaydium<'info>>,
        amount_in: u64,
        min_amount_out: u64,
        sqrt_price_limit_x64: u128,
    ) -> Result<()> {
        instructions::swap::handler(ctx, amount_in, min_amount_out, sqrt_price_limit_x64)
    }

    // ───────────────────────── Неделя 13: позиция ─────────────────────────

    #[allow(clippy::too_many_arguments)]
    pub fn open_raydium_position<'a, 'b, 'c: 'info, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, OpenRaydiumPosition<'info>>,
        tick_lower_index: i32,
        tick_upper_index: i32,
        tick_array_lower_start_index: i32,
        tick_array_upper_start_index: i32,
        liquidity: u128,
        amount_0_max: u64,
        amount_1_max: u64,
    ) -> Result<()> {
        instructions::open_position::handler(
            ctx,
            tick_lower_index,
            tick_upper_index,
            tick_array_lower_start_index,
            tick_array_upper_start_index,
            liquidity,
            amount_0_max,
            amount_1_max,
        )
    }

    pub fn collect_fees<'a, 'b, 'c: 'info, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, CollectFees<'info>>,
    ) -> Result<()> {
        instructions::collect_fees::handler(ctx)
    }

    pub fn close_raydium_position<'a, 'b, 'c: 'info, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, CloseRaydiumPosition<'info>>,
        amount_0_min: u64,
        amount_1_min: u64,
    ) -> Result<()> {
        instructions::close_position::handler(ctx, amount_0_min, amount_1_min)
    }
}

// ───────────────────────────── Accounts ─────────────────────────────

#[derive(Accounts)]
pub struct InitializeVault<'info> {
    #[account(
        init, payer = admin, space = VaultState::LEN,
        seeds = [b"vault", token_mint.key().as_ref()], bump,
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        init, payer = admin,
        token::mint = token_mint, token::authority = vault_state,
        seeds = [b"vault_tokens", token_mint.key().as_ref()], bump,
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    #[account(
        init, payer = admin,
        mint::decimals = 6, mint::authority = vault_state,
        seeds = [b"share_mint", token_mint.key().as_ref()], bump,
    )]
    pub share_mint: Account<'info, Mint>,

    pub token_mint: Account<'info, Mint>,

    #[account(mut)]
    pub admin: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct CreateUserCard<'info> {
    #[account(seeds = [b"vault", vault_state.token_mint.as_ref()], bump = vault_state.bump)]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        init, payer = user, space = UserCard::LEN,
        seeds = [b"user_card", vault_state.key().as_ref(), user.key().as_ref()], bump,
    )]
    pub user_card: Account<'info, UserCard>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CloseUserCard<'info> {
    #[account(seeds = [b"vault", vault_state.token_mint.as_ref()], bump = vault_state.bump)]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        mut, close = user,
        seeds = [b"user_card", vault_state.key().as_ref(), user.key().as_ref()],
        bump = user_card.bump,
        constraint = user_card.deposited == 0 @ VaultError::CardHasDeposit,
    )]
    pub user_card: Account<'info, UserCard>,

    #[account(mut)]
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct AdminOnly<'info> {
    #[account(
        mut,
        seeds = [b"vault", vault_state.token_mint.as_ref()], bump = vault_state.bump,
        has_one = admin @ VaultError::Unauthorized,
    )]
    pub vault_state: Account<'info, VaultState>,
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct SyncBalance<'info> {
    #[account(
        mut,
        seeds = [b"vault", vault_state.token_mint.as_ref()], bump = vault_state.bump,
        has_one = admin @ VaultError::Unauthorized,
    )]
    pub vault_state: Account<'info, VaultState>,
    #[account(address = vault_state.vault_token_account)]
    pub vault_token_account: Account<'info, TokenAccount>,
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut, seeds = [b"vault", vault_state.token_mint.as_ref()], bump = vault_state.bump)]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        mut, seeds = [b"user_card", vault_state.key().as_ref(), user.key().as_ref()],
        bump = user_card.bump,
    )]
    pub user_card: Account<'info, UserCard>,

    #[account(
        mut, address = vault_state.vault_token_account,
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    #[account(mut, address = vault_state.share_mint)]
    pub share_mint: Account<'info, Mint>,

    #[account(mut, token::mint = vault_state.token_mint, token::authority = user)]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(mut, token::mint = share_mint, token::authority = user)]
    pub user_share_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut, seeds = [b"vault", vault_state.token_mint.as_ref()], bump = vault_state.bump)]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        mut, seeds = [b"user_card", vault_state.key().as_ref(), user.key().as_ref()],
        bump = user_card.bump,
    )]
    pub user_card: Account<'info, UserCard>,

    #[account(mut, address = vault_state.vault_token_account)]
    pub vault_token_account: Account<'info, TokenAccount>,

    #[account(mut, address = vault_state.share_mint)]
    pub share_mint: Account<'info, Mint>,

    #[account(mut, token::mint = vault_state.token_mint, token::authority = user)]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(mut, token::mint = share_mint, token::authority = user)]
    pub user_share_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

// ───────────────────────────── State ─────────────────────────────

#[account]
#[derive(Default)]
pub struct VaultState {
    pub admin: Pubkey,
    pub token_mint: Pubkey,
    pub vault_token_account: Pubkey,
    pub share_mint: Pubkey,
    pub total_tokens: u64,
    pub total_shares: u64,
    pub paused: bool,
    pub bump: u8,
}

impl VaultState {
    pub const LEN: usize = 8 + 32 * 4 + 8 + 8 + 1 + 1;
}

#[account]
#[derive(Default)]
pub struct UserCard {
    pub vault: Pubkey,
    pub owner: Pubkey,
    pub deposited: u64,
    pub created_at: i64,
    pub bump: u8,
}

impl UserCard {
    pub const LEN: usize = 8 + 32 + 32 + 8 + 8 + 1;
}

// ───────────────────────────── Errors ─────────────────────────────

#[error_code]
pub enum VaultError {
    #[msg("Amount must be greater than zero")]
    ZeroAmount,
    #[msg("Shares would be zero - deposit too small")]
    SharesTooSmall,
    #[msg("Withdraw amount too small")]
    WithdrawTooSmall,
    #[msg("Vault is empty")]
    VaultEmpty,
    #[msg("Math overflow")]
    MathOverflow,
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("tick_lower must be less than tick_upper")]
    InvalidTickRange,
    #[msg("Vault is paused")]
    VaultPaused,
    #[msg("Card still has a deposit")]
    CardHasDeposit,
    #[msg("Not enough idle tokens in vault - close the Raydium position first")]
    InsufficientLiquidity,
    #[msg("min_amount_out must be greater than zero (slippage protection)")]
    NoSlippageProtection,
}
