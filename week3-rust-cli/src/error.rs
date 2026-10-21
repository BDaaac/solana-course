//! Единый тип ошибок программы.

use std::fmt;

use crate::amount::format_sol;

/// Все ошибки в одном enum — так же устроены `ProgramError` в solana-program
/// и `#[error_code]` в Anchor. Варианты могут нести данные для сообщения.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LedgerError {
    Usage(String),
    InvalidAmount(String),
    InvalidPubkey(String),
    UnknownAccount(String),
    InvalidLabel(String),
    LabelAlreadyExists(String),
    AccountNotFound(String),
    InsufficientFunds { needed: u64, available: u64 },
    InsufficientFundsForFee { fee: u64, available: u64 },
    AirdropLimitExceeded { requested: u64, max: u64 },
    SelfTransfer,
    InvalidAccountOwner,
    ArithmeticOverflow,
    InvalidInstructionData,
    CorruptedData(String),
    /// Box нужен, потому что enum не может содержать сам себя напрямую
    /// (размер типа был бы бесконечным).
    PreflightFailed(Box<LedgerError>),
    TransactionFailed(String),
    TransactionNotFound(String),
    Io(String),
}

/// Сокращение, как `ProgramResult` в Solana.
pub type Result<T> = std::result::Result<T, LedgerError>;

impl fmt::Display for LedgerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage(msg) => write!(f, "{msg}\nRun `solcli help` to see available commands"),
            Self::InvalidAmount(msg) => write!(f, "invalid amount {msg}"),
            Self::InvalidPubkey(s) => write!(f, "invalid public key `{s}`"),
            Self::UnknownAccount(s) => {
                write!(f, "`{s}` is neither a wallet label nor a valid base58 public key")
            }
            Self::InvalidLabel(s) => {
                write!(f, "invalid label `{s}`: use 1-32 characters [a-zA-Z0-9_-], not starting with `-`")
            }
            Self::LabelAlreadyExists(s) => write!(f, "label `{s}` already exists"),
            Self::AccountNotFound(pubkey) => {
                write!(f, "account {pubkey} not found on-chain (it has never been funded)")
            }
            Self::InsufficientFunds { needed, available } => write!(
                f,
                "insufficient funds: need {} SOL, available {} SOL",
                format_sol(*needed),
                format_sol(*available)
            ),
            Self::InsufficientFundsForFee { fee, available } => write!(
                f,
                "insufficient funds for fee: fee {} SOL, available {} SOL",
                format_sol(*fee),
                format_sol(*available)
            ),
            Self::AirdropLimitExceeded { requested, max } => write!(
                f,
                "airdrop limit exceeded: requested {} SOL, max {} SOL per request",
                format_sol(*requested),
                format_sol(*max)
            ),
            Self::SelfTransfer => write!(f, "cannot transfer to the same account"),
            Self::InvalidAccountOwner => {
                write!(f, "source account is not owned by the System Program")
            }
            Self::ArithmeticOverflow => write!(f, "arithmetic overflow"),
            Self::InvalidInstructionData => write!(f, "invalid instruction data"),
            Self::CorruptedData(msg) => write!(f, "corrupted data: {msg}"),
            Self::PreflightFailed(inner) => write!(
                f,
                "transaction simulation failed: {inner}\n(the transaction was NOT sent, no fee was charged)"
            ),
            Self::TransactionFailed(reason) => write!(
                f,
                "transaction landed on-chain but failed: {reason}\n(the fee was still charged)"
            ),
            Self::TransactionNotFound(s) => write!(f, "transaction `{s}` not found"),
            Self::Io(msg) => write!(f, "I/O error: {msg}"),
        }
    }
}

impl std::error::Error for LedgerError {}

/// Благодаря этому `?` сам превращает `io::Error` в `LedgerError`.
impl From<std::io::Error> for LedgerError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn messages_show_amounts_in_sol() {
        let err = LedgerError::InsufficientFunds { needed: 1_000_000_000, available: 500_000_000 };
        assert_eq!(err.to_string(), "insufficient funds: need 1 SOL, available 0.5 SOL");
    }

    #[test]
    fn io_errors_convert_with_question_mark() {
        fn read_missing() -> Result<Vec<u8>> {
            Ok(std::fs::read("definitely/missing/file.bin")?)
        }
        assert!(matches!(read_missing(), Err(LedgerError::Io(_))));
    }
}
