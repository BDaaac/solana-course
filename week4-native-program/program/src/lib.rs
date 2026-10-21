//! Неделя 4: первая Solana-программа без фреймворков (native).
//!
//! Программа хранит заметку в аккаунте, которым владеет сама программа:
//! - `Initialize { text }` — записать первую заметку и запомнить автора;
//! - `Update { text }` — изменить текст (может только автор).

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};

/// Максимальная длина текста заметки в байтах.
pub const MAX_TEXT_LEN: usize = 200;
/// Размер аккаунта: author (32) + длина строки (4) + текст (200) + счётчик правок (4).
pub const NOTE_SPACE: usize = 32 + 4 + MAX_TEXT_LEN + 4;

/// Instruction data: первый байт — вариант enum, дальше поля (формат Borsh).
#[derive(BorshSerialize, BorshDeserialize, Debug, PartialEq)]
pub enum NoteInstruction {
    Initialize { text: String },
    Update { text: String },
}

/// Данные, которые лежат в аккаунте заметки.
#[derive(BorshSerialize, BorshDeserialize, Debug, Default, PartialEq)]
pub struct Note {
    pub author: [u8; 32],
    pub text: String,
    pub updates: u32,
}

#[repr(u32)]
pub enum NoteError {
    TextTooLong = 1,
    NotAuthor = 2,
}

impl From<NoteError> for ProgramError {
    fn from(e: NoteError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = NoteInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    let accounts_iter = &mut accounts.iter();
    let note_account = next_account_info(accounts_iter)?;
    let author = next_account_info(accounts_iter)?;

    // Проверки безопасности: без них любой мог бы подсунуть чужой аккаунт
    if note_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }
    if !note_account.is_writable {
        return Err(ProgramError::InvalidAccountData);
    }
    if !author.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Новый аккаунт заполнен нулями — это валидная пустая Note
    let mut note = Note::deserialize(&mut &note_account.data.borrow()[..])
        .map_err(|_| ProgramError::InvalidAccountData)?;

    match instruction {
        NoteInstruction::Initialize { text } => {
            if note.author != [0; 32] {
                return Err(ProgramError::AccountAlreadyInitialized);
            }
            check_len(&text)?;
            note.author = author.key.to_bytes();
            note.text = text;
            msg!("Hello, Solana! Note created by {}", author.key);
        }
        NoteInstruction::Update { text } => {
            if note.author != author.key.to_bytes() {
                return Err(NoteError::NotAuthor.into());
            }
            check_len(&text)?;
            note.text = text;
            note.updates = note.updates.checked_add(1).ok_or(ProgramError::ArithmeticOverflow)?;
            msg!("Note updated, total updates: {}", note.updates);
        }
    }

    msg!("Text: {}", note.text);
    note.serialize(&mut &mut note_account.data.borrow_mut()[..])
        .map_err(|_| ProgramError::AccountDataTooSmall)?;
    Ok(())
}

fn check_len(text: &str) -> ProgramResult {
    if text.len() > MAX_TEXT_LEN {
        return Err(NoteError::TextTooLong.into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instruction_layout_is_tag_plus_borsh_string() {
        let data = borsh::to_vec(&NoteInstruction::Update { text: "hi".into() }).unwrap();
        assert_eq!(data, vec![1, 2, 0, 0, 0, b'h', b'i']);
    }

    #[test]
    fn zeroed_account_is_empty_note() {
        let zeroed = vec![0u8; NOTE_SPACE];
        assert_eq!(Note::deserialize(&mut &zeroed[..]).unwrap(), Note::default());
    }

    #[test]
    fn max_note_fits_into_account() {
        let note = Note { author: [7; 32], text: "x".repeat(MAX_TEXT_LEN), updates: u32::MAX };
        assert_eq!(borsh::to_vec(&note).unwrap().len(), NOTE_SPACE);
    }
}
