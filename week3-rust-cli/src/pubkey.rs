//! Адреса и подписи: 32-байтовый Pubkey и 64-байтовая Signature.

use std::collections::hash_map::DefaultHasher;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::base58;
use crate::error::{LedgerError, Result};
use crate::pack::Pack;

pub const PUBKEY_BYTES: usize = 32;
pub const SIGNATURE_BYTES: usize = 64;

/// Адрес аккаунта: ровно 32 байта, как `solana_program::pubkey::Pubkey`.
///
/// Newtype над массивом: компилятор не даст перепутать адрес с любыми
/// другими 32 байтами. `Copy`, потому что 32 байта дёшево копировать
/// (в Solana `Pubkey` тоже `Copy`). `Hash` + `Ord` нужны, чтобы
/// использовать адрес как ключ в HashMap/BTreeMap.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Pubkey([u8; PUBKEY_BYTES]);

/// ID System Program: 32 нулевых байта, в base58 это "111...1" (32 единицы).
pub const SYSTEM_PROGRAM_ID: Pubkey = Pubkey::new([0; PUBKEY_BYTES]);

impl Pubkey {
    pub const fn new(bytes: [u8; PUBKEY_BYTES]) -> Self {
        Self(bytes)
    }

    /// Псевдослучайный адрес из seed.
    ///
    /// В настоящей Solana адрес кошелька — публичный ключ ed25519
    /// (его создаёт `solana-keygen new`); здесь обходимся без криптографии.
    pub fn new_unique(seed: u64) -> Self {
        Self(expand_seed(seed))
    }

    /// Короткая запись для таблиц: "7xKX...gAsU".
    pub fn short(&self) -> String {
        shorten(&self.to_string(), 4)
    }
}

/// Подпись транзакции: 64 байта, в base58 около 88 символов.
/// Именно по ней транзакцию ищут в Solana Explorer.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Signature([u8; SIGNATURE_BYTES]);

impl Signature {
    /// Детерминированная "подпись" из байтов транзакции и номера слота.
    /// (Настоящая — ed25519-подпись fee payer'а над сообщением транзакции.)
    pub fn derive(message: &[u8], slot: u64) -> Self {
        let mut hasher = DefaultHasher::new();
        message.hash(&mut hasher);
        slot.hash(&mut hasher);
        Self(expand_seed(hasher.finish()))
    }

    pub fn short(&self) -> String {
        shorten(&self.to_string(), 6)
    }
}

/// Одинаковые Display/Debug для обоих типов генерируем макросом
/// (в Anchor похожую работу делают `#[derive(...)]`-макросы).
macro_rules! impl_base58_fmt {
    ($($name:ident),*) => {$(
        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                // pad(), а не write_str(): так работают ширина и выравнивание {:<44}
                f.pad(&base58::encode(&self.0))
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}({})", stringify!($name), self)
            }
        }

        impl Pack for $name {
            fn pack(&self, out: &mut Vec<u8>) {
                self.0.pack(out);
            }

            fn unpack(input: &mut &[u8]) -> Result<Self> {
                Ok(Self(Pack::unpack(input)?))
            }
        }
    )*};
}

impl_base58_fmt!(Pubkey, Signature);

/// Позволяет писать `"TokenkegQ...".parse::<Pubkey>()`.
impl FromStr for Pubkey {
    type Err = LedgerError;

    fn from_str(s: &str) -> Result<Self> {
        let invalid = || LedgerError::InvalidPubkey(s.to_string());
        let bytes = base58::decode(s).ok_or_else(invalid)?;
        // Vec<u8> -> [u8; 32]: не получится, если длина не 32
        let bytes: [u8; PUBKEY_BYTES] = bytes.try_into().map_err(|_| invalid())?;
        Ok(Self(bytes))
    }
}

/// Энтропия для keygen: текущее время + PID процесса + метка.
pub fn random_seed(label: &str) -> u64 {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default();
    let mut hasher = DefaultHasher::new();
    (nanos, std::process::id(), label).hash(&mut hasher);
    hasher.finish()
}

/// Растягивает 64-битный seed в N псевдослучайных байт (алгоритм SplitMix64).
/// Здесь нужны wrapping_*: переполнение — часть алгоритма, а не ошибка.
/// В подсчёте балансов, наоборот, используется только checked_*.
fn expand_seed<const N: usize>(mut seed: u64) -> [u8; N] {
    let mut out = [0u8; N];
    for chunk in out.chunks_mut(8) {
        seed = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = seed;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^= z >> 31;
        chunk.copy_from_slice(&z.to_le_bytes()[..chunk.len()]);
    }
    out
}

fn shorten(s: &str, keep: usize) -> String {
    if s.len() <= keep * 2 + 3 {
        return s.to_string();
    }
    // base58 — только ASCII, поэтому срезать строку по байтам безопасно
    format!("{}...{}", &s[..keep], &s[s.len() - keep..])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_program_id_is_32_ones() {
        assert_eq!(SYSTEM_PROGRAM_ID.to_string(), "11111111111111111111111111111111");
    }

    #[test]
    fn parses_real_solana_address() {
        let token_program: Pubkey = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".parse().unwrap();
        assert_eq!(token_program.to_string(), "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
        assert_eq!(token_program.short(), "Toke...Q5DA");
    }

    #[test]
    fn rejects_wrong_length_and_bad_characters() {
        // валидный base58, но это 12 байт, а не 32
        assert_eq!(
            "2NEpo7TZRRrLZSi2U".parse::<Pubkey>(),
            Err(LedgerError::InvalidPubkey("2NEpo7TZRRrLZSi2U".into()))
        );
        assert!("0OIl".parse::<Pubkey>().is_err());
    }

    #[test]
    fn unique_keys_are_deterministic_per_seed() {
        assert_eq!(Pubkey::new_unique(1), Pubkey::new_unique(1));
        assert_ne!(Pubkey::new_unique(1), Pubkey::new_unique(2));
    }

    #[test]
    fn signature_is_64_bytes() {
        let text = Signature::derive(b"tx", 1).to_string();
        assert!(text.len() <= 88, "{text}");
        assert_eq!(base58::decode(&text).unwrap().len(), SIGNATURE_BYTES);
        assert_ne!(Signature::derive(b"tx", 1), Signature::derive(b"tx", 2));
    }

    #[test]
    fn display_respects_width() {
        assert_eq!(format!("[{:>34}]", SYSTEM_PROGRAM_ID), format!("[  {}]", "1".repeat(32)));
        assert_eq!(format!("{:?}", SYSTEM_PROGRAM_ID), format!("Pubkey({})", "1".repeat(32)));
    }
}
