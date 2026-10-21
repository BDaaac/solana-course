//! Base58: кодировка, в которой Solana показывает адреса и подписи.
//! Алфавит Bitcoin: без символов 0, O, I, l, чтобы их не путали при чтении.

const ALPHABET: &[u8; 58] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

/// Кодирует байты в base58-строку.
pub fn encode(input: &[u8]) -> String {
    // Каждый ведущий нулевой байт кодируется отдельным символом '1'
    let zeros = input.iter().take_while(|&&byte| byte == 0).count();

    // Большое число в системе счисления 58; цифры хранятся от младшей к старшей
    let mut digits: Vec<u8> = Vec::with_capacity(input.len() * 138 / 100 + 1);
    for &byte in &input[zeros..] {
        let mut carry = u32::from(byte);
        for digit in digits.iter_mut() {
            carry += u32::from(*digit) << 8;
            *digit = (carry % 58) as u8;
            carry /= 58;
        }
        while carry > 0 {
            digits.push((carry % 58) as u8);
            carry /= 58;
        }
    }

    let mut out = String::with_capacity(zeros + digits.len());
    out.extend(std::iter::repeat('1').take(zeros));
    out.extend(digits.iter().rev().map(|&d| char::from(ALPHABET[usize::from(d)])));
    out
}

/// Декодирует base58-строку. Возвращает `None`, если есть символ не из алфавита.
pub fn decode(input: &str) -> Option<Vec<u8>> {
    let zeros = input.bytes().take_while(|&c| c == b'1').count();

    // Большое число в системе счисления 256; байты от младшего к старшему
    let mut bytes: Vec<u8> = Vec::with_capacity(input.len());
    for c in input.bytes().skip(zeros) {
        // `?` работает и с Option: неизвестный символ -> сразу return None
        let mut carry = ALPHABET.iter().position(|&a| a == c)? as u32;
        for byte in bytes.iter_mut() {
            carry += u32::from(*byte) * 58;
            *byte = (carry & 0xff) as u8;
            carry >>= 8;
        }
        while carry > 0 {
            bytes.push((carry & 0xff) as u8);
            carry >>= 8;
        }
    }

    let mut out = vec![0u8; zeros];
    out.extend(bytes.iter().rev());
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_reference_vector() {
        assert_eq!(encode(b"Hello World!"), "2NEpo7TZRRrLZSi2U");
        assert_eq!(decode("2NEpo7TZRRrLZSi2U").unwrap(), b"Hello World!");
    }

    #[test]
    fn leading_zero_bytes_become_ones() {
        assert_eq!(encode(&[0; 32]), "1".repeat(32));
        assert_eq!(decode(&"1".repeat(32)).unwrap(), vec![0; 32]);
        assert_eq!(encode(&[0, 0, 1]), "112");
    }

    #[test]
    fn roundtrip_and_empty_input() {
        let data = [0, 0, 1, 2, 3, 255, 254, 0, 128];
        assert_eq!(decode(&encode(&data)).unwrap(), data);
        assert_eq!(encode(&[]), "");
        assert_eq!(decode("").unwrap(), Vec::<u8>::new());
    }

    #[test]
    fn rejects_characters_outside_alphabet() {
        for bad in ["0", "O", "I", "l", "abc!", "кириллица"] {
            assert_eq!(decode(bad), None, "input: {bad}");
        }
    }
}
