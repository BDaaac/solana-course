//! Инструкции нашей "System Program" и их байтовый формат.

use crate::error::{LedgerError, Result};
use crate::pack::Pack;
use crate::pubkey::Pubkey;

/// Enum с данными в вариантах — стандартный способ описать инструкции
/// программы (ср. `SystemInstruction` в solana-program).
///
/// В native-программах instruction_data приходит как `&[u8]`:
/// первый байт — тег варианта, дальше — поля. Здесь ровно так же.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    /// Тег 0: начислить тестовые SOL (как `solana airdrop` в devnet).
    Airdrop { to: Pubkey, lamports: u64 },
    /// Тег 1: перевод лампортов (как `SystemInstruction::Transfer`).
    Transfer { from: Pubkey, to: Pubkey, lamports: u64 },
}

impl Instruction {
    const AIRDROP_TAG: u8 = 0;
    const TRANSFER_TAG: u8 = 1;

    pub fn name(&self) -> &'static str {
        match self {
            Self::Airdrop { .. } => "Airdrop",
            Self::Transfer { .. } => "Transfer",
        }
    }

    /// Кто платит комиссию. Airdrop бесплатный: его "оплачивает" faucet.
    pub fn fee_payer(&self) -> Option<Pubkey> {
        match self {
            Self::Airdrop { .. } => None,
            Self::Transfer { from, .. } => Some(*from),
        }
    }

    /// Аккаунты, которые трогает инструкция. В Solana их перечисляют заранее
    /// (AccountMeta), поэтому рантайм Sealevel может выполнять транзакции
    /// с непересекающимися аккаунтами параллельно.
    pub fn accounts(&self) -> Vec<Pubkey> {
        match *self {
            Self::Airdrop { to, .. } => vec![to],
            Self::Transfer { from, to, .. } => vec![from, to],
        }
    }
}

impl Pack for Instruction {
    fn pack(&self, out: &mut Vec<u8>) {
        match self {
            Self::Airdrop { to, lamports } => {
                Self::AIRDROP_TAG.pack(out);
                to.pack(out);
                lamports.pack(out);
            }
            Self::Transfer { from, to, lamports } => {
                Self::TRANSFER_TAG.pack(out);
                from.pack(out);
                to.pack(out);
                lamports.pack(out);
            }
        }
    }

    fn unpack(input: &mut &[u8]) -> Result<Self> {
        // Классика native-программ: `let (tag, rest) = input.split_first()`
        let data: &[u8] = *input;
        let (&tag, rest) = data.split_first().ok_or(LedgerError::InvalidInstructionData)?;
        *input = rest;

        match tag {
            Self::AIRDROP_TAG => Ok(Self::Airdrop {
                to: Pubkey::unpack(input)?,
                lamports: u64::unpack(input)?,
            }),
            Self::TRANSFER_TAG => Ok(Self::Transfer {
                from: Pubkey::unpack(input)?,
                to: Pubkey::unpack(input)?,
                lamports: u64::unpack(input)?,
            }),
            _ => Err(LedgerError::InvalidInstructionData),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transfer_has_expected_byte_layout() {
        let from = Pubkey::new([1; 32]);
        let to = Pubkey::new([2; 32]);
        let bytes = Instruction::Transfer { from, to, lamports: 500 }.to_bytes();

        assert_eq!(bytes.len(), 1 + 32 + 32 + 8);
        assert_eq!(bytes[0], 1); // тег
        assert_eq!(&bytes[1..33], &[1; 32]); // from
        assert_eq!(&bytes[33..65], &[2; 32]); // to
        assert_eq!(&bytes[65..], &500u64.to_le_bytes()); // lamports, little-endian
    }

    #[test]
    fn instructions_roundtrip() {
        let key = Pubkey::new_unique(5);
        for ix in [
            Instruction::Airdrop { to: key, lamports: 42 },
            Instruction::Transfer { from: key, to: Pubkey::new_unique(6), lamports: u64::MAX },
        ] {
            assert_eq!(Instruction::from_bytes(&ix.to_bytes()), Ok(ix));
        }
    }

    #[test]
    fn rejects_bad_instruction_data() {
        assert_eq!(Instruction::from_bytes(&[]), Err(LedgerError::InvalidInstructionData));
        assert_eq!(Instruction::from_bytes(&[9]), Err(LedgerError::InvalidInstructionData));

        let bytes = Instruction::Airdrop { to: Pubkey::new_unique(1), lamports: 1 }.to_bytes();
        assert!(matches!(Instruction::from_bytes(&bytes[..10]), Err(LedgerError::CorruptedData(_))));
    }

    #[test]
    fn fee_payer_and_accounts() {
        let (a, b) = (Pubkey::new_unique(1), Pubkey::new_unique(2));
        let airdrop = Instruction::Airdrop { to: a, lamports: 1 };
        let transfer = Instruction::Transfer { from: a, to: b, lamports: 1 };
        assert_eq!(airdrop.fee_payer(), None);
        assert_eq!(transfer.fee_payer(), Some(a));
        assert_eq!(transfer.accounts(), vec![a, b]);
    }
}
