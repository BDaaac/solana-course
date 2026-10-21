//! Мини-аналог Borsh: формата, которым Solana-программы и Anchor
//! сериализуют данные аккаунтов и инструкций.
//!
//! Для используемых типов байты совпадают с Borsh:
//! целые числа little-endian, bool = 1 байт, String/Vec = длина u32 + элементы.

use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;

use crate::error::{LedgerError, Result};

/// Тип, который умеет превращаться в байты и обратно.
/// Аналог пары трейтов `BorshSerialize` + `BorshDeserialize`.
pub trait Pack: Sized {
    /// Дописывает байтовое представление в конец буфера.
    fn pack(&self, out: &mut Vec<u8>);

    /// Читает значение из начала среза и сдвигает срез вперёд
    /// (тот же приём с `&mut &[u8]`, что в `BorshDeserialize::deserialize`).
    fn unpack(input: &mut &[u8]) -> Result<Self>;

    /// Метод по умолчанию: реализовывать его в каждом типе не нужно.
    fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        self.pack(&mut out);
        out
    }

    /// Строгая десериализация: лишние байты в конце считаются ошибкой.
    fn from_bytes(mut data: &[u8]) -> Result<Self> {
        let value = Self::unpack(&mut data)?;
        if !data.is_empty() {
            return Err(LedgerError::CorruptedData(format!(
                "{} unexpected trailing bytes",
                data.len()
            )));
        }
        Ok(value)
    }
}

/// Откусывает `n` байт от начала среза.
///
/// Лайфтайм `'a` говорит компилятору: возвращаемый кусок живёт столько же,
/// сколько исходные данные, а не сколько временная ссылка `input`.
pub fn take<'a>(input: &mut &'a [u8], n: usize) -> Result<&'a [u8]> {
    let data: &'a [u8] = *input;
    if data.len() < n {
        return Err(LedgerError::CorruptedData(format!(
            "expected {n} more bytes, got {}",
            data.len()
        )));
    }
    let (head, tail) = data.split_at(n);
    *input = tail;
    Ok(head)
}

fn pack_len(len: usize, out: &mut Vec<u8>) {
    // В Borsh длина коллекции — u32; больше 4 млрд элементов здесь не бывает
    let len = u32::try_from(len).expect("collection is too large to serialize");
    len.pack(out);
}

fn unpack_len(input: &mut &[u8]) -> Result<usize> {
    Ok(u32::unpack(input)? as usize)
}

/// Декларативный макрос: одна и та же реализация для нескольких целых типов.
macro_rules! impl_pack_for_int {
    ($($int:ty),*) => {$(
        impl Pack for $int {
            fn pack(&self, out: &mut Vec<u8>) {
                out.extend_from_slice(&self.to_le_bytes());
            }

            fn unpack(input: &mut &[u8]) -> Result<Self> {
                let bytes = take(input, std::mem::size_of::<$int>())?;
                let array = bytes
                    .try_into()
                    .map_err(|_| LedgerError::CorruptedData("bad integer".into()))?;
                Ok(<$int>::from_le_bytes(array))
            }
        }
    )*};
}

impl_pack_for_int!(u8, u32, u64);

impl Pack for bool {
    fn pack(&self, out: &mut Vec<u8>) {
        out.push(u8::from(*self));
    }

    fn unpack(input: &mut &[u8]) -> Result<Self> {
        match u8::unpack(input)? {
            0 => Ok(false),
            1 => Ok(true),
            other => Err(LedgerError::CorruptedData(format!("invalid bool byte {other}"))),
        }
    }
}

impl Pack for String {
    fn pack(&self, out: &mut Vec<u8>) {
        pack_len(self.len(), out);
        out.extend_from_slice(self.as_bytes());
    }

    fn unpack(input: &mut &[u8]) -> Result<Self> {
        let len = unpack_len(input)?;
        let bytes = take(input, len)?;
        String::from_utf8(bytes.to_vec())
            .map_err(|_| LedgerError::CorruptedData("invalid UTF-8 string".into()))
    }
}

/// Const generics: одна реализация для массивов любой длины N
/// (нужно для Pubkey = [u8; 32] и Signature = [u8; 64]).
impl<const N: usize> Pack for [u8; N] {
    fn pack(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(self);
    }

    fn unpack(input: &mut &[u8]) -> Result<Self> {
        let mut array = [0u8; N];
        array.copy_from_slice(take(input, N)?);
        Ok(array)
    }
}

/// Generic-реализация с ограничением трейта: Vec<T> умеет сериализоваться,
/// если умеет его элемент T.
impl<T: Pack> Pack for Vec<T> {
    fn pack(&self, out: &mut Vec<u8>) {
        pack_len(self.len(), out);
        for item in self {
            item.pack(out);
        }
    }

    fn unpack(input: &mut &[u8]) -> Result<Self> {
        let len = unpack_len(input)?;
        // Не доверяем длине из входных данных: не резервируем память больше,
        // чем реально осталось байт (защита от "злого" ввода).
        let mut items = Vec::with_capacity(len.min(input.len()));
        for _ in 0..len {
            items.push(T::unpack(input)?);
        }
        Ok(items)
    }
}

impl<A: Pack, B: Pack> Pack for (A, B) {
    fn pack(&self, out: &mut Vec<u8>) {
        self.0.pack(out);
        self.1.pack(out);
    }

    fn unpack(input: &mut &[u8]) -> Result<Self> {
        Ok((A::unpack(input)?, B::unpack(input)?))
    }
}

impl<K: Pack + Eq + Hash + Ord, V: Pack> Pack for HashMap<K, V> {
    fn pack(&self, out: &mut Vec<u8>) {
        // Как в Borsh: сортируем ключи, чтобы одно и то же состояние всегда
        // давало одни и те же байты. Для блокчейна детерминизм обязателен.
        let mut entries: Vec<(&K, &V)> = self.iter().collect();
        entries.sort_by(|a, b| a.0.cmp(b.0));
        pack_len(entries.len(), out);
        for (key, value) in entries {
            key.pack(out);
            value.pack(out);
        }
    }

    fn unpack(input: &mut &[u8]) -> Result<Self> {
        let len = unpack_len(input)?;
        let mut map = HashMap::with_capacity(len.min(input.len()));
        for _ in 0..len {
            let key = K::unpack(input)?;
            let value = V::unpack(input)?;
            map.insert(key, value);
        }
        Ok(map)
    }
}

impl<K: Pack + Ord, V: Pack> Pack for BTreeMap<K, V> {
    fn pack(&self, out: &mut Vec<u8>) {
        pack_len(self.len(), out);
        for (key, value) in self {
            key.pack(out);
            value.pack(out);
        }
    }

    fn unpack(input: &mut &[u8]) -> Result<Self> {
        let len = unpack_len(input)?;
        let mut map = BTreeMap::new();
        for _ in 0..len {
            let key = K::unpack(input)?;
            let value = V::unpack(input)?;
            map.insert(key, value);
        }
        Ok(map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roundtrip<T: Pack + PartialEq + std::fmt::Debug>(value: T) {
        let bytes = value.to_bytes();
        assert_eq!(T::from_bytes(&bytes).unwrap(), value);
    }

    #[test]
    fn primitives_roundtrip() {
        roundtrip(0u8);
        roundtrip(u32::MAX);
        roundtrip(1_234_567_890_123u64);
        roundtrip(true);
        roundtrip(false);
        roundtrip(String::from("привет, Solana"));
        roundtrip([7u8; 32]);
    }

    #[test]
    fn collections_roundtrip() {
        roundtrip(vec![1u64, 2, 3]);
        roundtrip((String::from("a"), 5u64));
        roundtrip(BTreeMap::from([("x".to_string(), 1u64), ("y".to_string(), 2)]));
        roundtrip(HashMap::from([(1u64, true), (2, false), (3, true)]));
    }

    #[test]
    fn layout_is_borsh_compatible() {
        assert_eq!(1u64.to_bytes(), [1, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(String::from("hi").to_bytes(), [2, 0, 0, 0, b'h', b'i']);
        assert_eq!(vec![true, false].to_bytes(), [2, 0, 0, 0, 1, 0]);
    }

    #[test]
    fn rejects_truncated_invalid_and_trailing_data() {
        assert!(matches!(u64::from_bytes(&[1, 2, 3]), Err(LedgerError::CorruptedData(_))));
        assert!(matches!(bool::from_bytes(&[2]), Err(LedgerError::CorruptedData(_))));
        assert!(matches!(u8::from_bytes(&[1, 2]), Err(LedgerError::CorruptedData(_))));
        assert!(matches!(
            String::from_bytes(&[2, 0, 0, 0, 0xff, 0xfe]),
            Err(LedgerError::CorruptedData(_))
        ));
    }

    #[test]
    fn huge_length_prefix_does_not_allocate_or_panic() {
        // "Злой" ввод: длина u32::MAX, а данных нет
        let evil = [0xff, 0xff, 0xff, 0xff];
        assert!(Vec::<u64>::from_bytes(&evil).is_err());
        assert!(String::from_bytes(&evil).is_err());
    }
}
