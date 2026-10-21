//! Перевод SOL <-> lamports без чисел с плавающей точкой.

use crate::error::{LedgerError, Result};

/// 1 SOL = 1 000 000 000 lamports, как `LAMPORTS_PER_SOL` в solana-sdk.
pub const LAMPORTS_PER_SOL: u64 = 1_000_000_000;
const DECIMALS: usize = 9;

/// Парсит строку "1.5" в 1_500_000_000 lamports.
///
/// Никаких f64: в блокчейне деньги считают только целыми числами,
/// иначе 0.1 + 0.2 != 0.3 и балансы "теряют" лампорты на округлении.
pub fn parse_sol(input: &str) -> Result<u64> {
    let invalid = |reason: &str| LedgerError::InvalidAmount(format!("`{input}`: {reason}"));

    let (whole, frac) = input.split_once('.').unwrap_or((input, ""));
    if whole.is_empty() && frac.is_empty() {
        return Err(invalid("empty number"));
    }
    if !whole.bytes().chain(frac.bytes()).all(|b| b.is_ascii_digit()) {
        return Err(invalid("only digits and one decimal point are allowed"));
    }
    if frac.len() > DECIMALS {
        return Err(invalid("at most 9 decimal places (1 lamport = 0.000000001 SOL)"));
    }

    let whole: u64 = if whole.is_empty() {
        0
    } else {
        whole.parse().map_err(|_| invalid("number is too large"))?
    };
    // "5" -> "500000000": дополняем дробную часть нулями до 9 знаков
    let frac: u64 = format!("{:0<width$}", frac, width = DECIMALS)
        .parse()
        .map_err(|_| invalid("bad fractional part"))?;

    // checked_*: при переполнении получаем None вместо тихой ошибки
    whole
        .checked_mul(LAMPORTS_PER_SOL)
        .and_then(|lamports| lamports.checked_add(frac))
        .ok_or_else(|| invalid("number is too large"))
}

/// 1_500_000_000 -> "1.5", 5_000 -> "0.000005".
pub fn format_sol(lamports: u64) -> String {
    let whole = lamports / LAMPORTS_PER_SOL;
    let frac = lamports % LAMPORTS_PER_SOL;
    if frac == 0 {
        whole.to_string()
    } else {
        let frac = format!("{frac:09}");
        format!("{whole}.{}", frac.trim_end_matches('0'))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_whole_and_fractional_sol() {
        assert_eq!(parse_sol("1"), Ok(LAMPORTS_PER_SOL));
        assert_eq!(parse_sol("1.5"), Ok(1_500_000_000));
        assert_eq!(parse_sol("0.000000001"), Ok(1));
        assert_eq!(parse_sol(".25"), Ok(250_000_000));
        assert_eq!(parse_sol("2."), Ok(2 * LAMPORTS_PER_SOL));
        assert_eq!(parse_sol("18446744073.709551615"), Ok(u64::MAX));
    }

    #[test]
    fn rejects_garbage_too_precise_and_too_large() {
        let bad_inputs = [
            "", ".", "abc", "-1", "+1", "1.2.3", "1,5", " 1", "1e9", "0.0000000001", "18446744074",
        ];
        for bad in bad_inputs {
            assert!(matches!(parse_sol(bad), Err(LedgerError::InvalidAmount(_))), "input: {bad:?}");
        }
    }

    #[test]
    fn formats_lamports_as_sol() {
        assert_eq!(format_sol(0), "0");
        assert_eq!(format_sol(1), "0.000000001");
        assert_eq!(format_sol(5_000), "0.000005");
        assert_eq!(format_sol(1_500_000_000), "1.5");
        assert_eq!(format_sol(u64::MAX), "18446744073.709551615");
    }

    #[test]
    fn format_and_parse_roundtrip() {
        for lamports in [0, 1, 5_000, 1_500_000_000, u64::MAX] {
            assert_eq!(parse_sol(&format_sol(lamports)), Ok(lamports));
        }
    }
}
