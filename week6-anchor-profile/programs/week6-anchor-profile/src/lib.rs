//! Неделя 6: децентрализованный on-chain профиль на PDA.
//!
//! Архитектура аккаунтов:
//! - `Profile`  PDA ["profile", authority]  — один профиль на кошелёк;
//! - `Username` PDA ["username", username]  — реестр имён: гарантирует уникальность
//!   username (второй `init` с тем же сидом упадёт, адрес уже занят).
//!
//! Никаких приватных ключей у PDA нет: подписывать за них может только программа.
use anchor_lang::prelude::*;

declare_id!("ABcoiQozKZMZ7uiUdha1ip8Z6zDpekTP9iifUeyGcsS1");

pub const PROFILE_SEED: &[u8] = b"profile";
pub const USERNAME_SEED: &[u8] = b"username";
pub const MAX_USERNAME: usize = 32;
pub const MAX_BIO: usize = 200;
pub const MAX_AVATAR_URL: usize = 128;

#[program]
pub mod week6_anchor_profile {
    use super::*;

    pub fn create_profile(ctx: Context<CreateProfile>, username: String, bio: String, avatar_url: String) -> Result<()> {
        validate_username(&username)?;
        validate_text(&bio, &avatar_url)?;
        let now = Clock::get()?.unix_timestamp;

        let profile = &mut ctx.accounts.profile;
        profile.authority = ctx.accounts.authority.key();
        profile.username = username.clone();
        profile.bio = bio;
        profile.avatar_url = avatar_url;
        profile.created_at = now;
        profile.updated_at = now;
        profile.bump = ctx.bumps.profile;

        let record = &mut ctx.accounts.username_record;
        record.authority = ctx.accounts.authority.key();
        record.bump = ctx.bumps.username_record;

        emit!(ProfileCreated { authority: profile.authority, username });
        msg!("Profile @{} created", profile.username);
        Ok(())
    }

    pub fn update_profile(ctx: Context<UpdateProfile>, bio: String, avatar_url: String) -> Result<()> {
        validate_text(&bio, &avatar_url)?;
        let profile = &mut ctx.accounts.profile;
        profile.bio = bio;
        profile.avatar_url = avatar_url;
        profile.updated_at = Clock::get()?.unix_timestamp;
        msg!("Profile @{} updated", profile.username);
        Ok(())
    }

    /// Закрывает оба аккаунта: rent возвращается владельцу, username освобождается.
    pub fn delete_profile(ctx: Context<DeleteProfile>) -> Result<()> {
        msg!("Profile @{} deleted", ctx.accounts.profile.username);
        Ok(())
    }
}

// ---------- Аккаунты (данные) ----------

#[account]
#[derive(InitSpace)]
pub struct Profile {
    pub authority: Pubkey,
    #[max_len(MAX_USERNAME)]
    pub username: String,
    #[max_len(MAX_BIO)]
    pub bio: String,
    #[max_len(MAX_AVATAR_URL)]
    pub avatar_url: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct Username {
    pub authority: Pubkey,
    pub bump: u8,
}

// ---------- Контексты инструкций ----------

#[derive(Accounts)]
#[instruction(username: String)]
pub struct CreateProfile<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init,
        payer = authority,
        space = 8 + Profile::INIT_SPACE,
        seeds = [PROFILE_SEED, authority.key().as_ref()],
        bump
    )]
    pub profile: Account<'info, Profile>,
    #[account(
        init,
        payer = authority,
        space = 8 + Username::INIT_SPACE,
        seeds = [USERNAME_SEED, username.as_bytes()],
        bump
    )]
    pub username_record: Account<'info, Username>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateProfile<'info> {
    pub authority: Signer<'info>,
    #[account(
        mut,
        seeds = [PROFILE_SEED, authority.key().as_ref()],
        bump = profile.bump,
        has_one = authority
    )]
    pub profile: Account<'info, Profile>,
}

#[derive(Accounts)]
pub struct DeleteProfile<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        mut,
        close = authority,
        seeds = [PROFILE_SEED, authority.key().as_ref()],
        bump = profile.bump,
        has_one = authority
    )]
    pub profile: Account<'info, Profile>,
    #[account(
        mut,
        close = authority,
        seeds = [USERNAME_SEED, profile.username.as_bytes()],
        bump = username_record.bump,
        has_one = authority
    )]
    pub username_record: Account<'info, Username>,
}

// ---------- События и ошибки ----------

#[event]
pub struct ProfileCreated {
    pub authority: Pubkey,
    pub username: String,
}

#[error_code]
pub enum ProfileError {
    #[msg("Username must be 3-32 characters: a-z, 0-9, _")]
    InvalidUsername,
    #[msg("Bio is too long (max 200 bytes)")]
    BioTooLong,
    #[msg("Avatar URL is too long (max 128 bytes)")]
    AvatarUrlTooLong,
}

fn validate_username(username: &str) -> Result<()> {
    let ok_len = (3..=MAX_USERNAME).contains(&username.len());
    let ok_chars = username.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_');
    require!(ok_len && ok_chars, ProfileError::InvalidUsername);
    Ok(())
}

fn validate_text(bio: &str, avatar_url: &str) -> Result<()> {
    require!(bio.len() <= MAX_BIO, ProfileError::BioTooLong);
    require!(avatar_url.len() <= MAX_AVATAR_URL, ProfileError::AvatarUrlTooLong);
    Ok(())
}
