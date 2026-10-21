# solcli: мини-симулятор Solana на Rust

**Неделя 3. Rust для Solana-разработчика.**
Задание: небольшая CLI-программа на Rust с конструкциями, которые понадобятся в Solana-разработке.

`solcli` — консольный "блокчейн в миниатюре". Он повторяет модель аккаунтов Solana:
адреса в base58, балансы в лампортах, System Program, комиссия 5000 лампортов,
атомарные транзакции, preflight-симуляция, история транзакций с подписями.
Внешних зависимостей нет, только стандартная библиотека Rust.

## Запуск

```bash
cargo build --release
cargo test

cargo run -- keygen alice
cargo run -- keygen bob
cargo run -- airdrop 2 alice
cargo run -- transfer alice bob 0.5
cargo run -- accounts
cargo run -- history
cargo run -- account alice
cargo run -- tx <первые символы подписи>
cargo run -- transfer bob alice 10                     # preflight отклонит, комиссия не спишется
cargo run -- transfer bob alice 10 --skip-preflight    # упадёт on-chain, комиссия спишется
```

Состояние хранится в `ledger.bin` в текущей папке (путь меняется переменной `SOLCLI_LEDGER`).

## Пример работы

```
> solcli transfer alice bob 0.5
Transfer 0.5 SOL alice -> bob
Signature: 2mJGa2YyyBcDiaqpZGhogkrpg58oFkTM5vUd9nLasG5nX3pmu17Yt4NjySxKGk4Pa4GurAnjHvFihcfBX6VGG5rT
Slot: 2 | Fee: 0.000005 SOL
Balances: alice = 1.499995 SOL, bob = 0.5 SOL
Status: Ok

> solcli transfer bob alice 10
Error: transaction simulation failed: insufficient funds: need 10 SOL, available 0.499995 SOL
(the transaction was NOT sent, no fee was charged)

> solcli transfer bob alice 10 --skip-preflight
...
Error: transaction landed on-chain but failed: insufficient funds: need 10 SOL, available 0.499995 SOL
(the fee was still charged)

> solcli history
SLOT  SIGNATURE        STATUS  FEE (SOL)  INSTRUCTION
   3  5HNCJJ...E1r5wE  Failed   0.000005  Transfer 10 SOL bob -> alice
      -> insufficient funds: need 10 SOL, available 0.499995 SOL
   2  2mJGa2...VGG5rT  Ok       0.000005  Transfer 0.5 SOL alice -> bob
   1  2gbkD8...ayzFZA  Ok              0  Airdrop 2 SOL -> alice

> solcli tx 2mJGa2
Signature:   2mJGa2YyyBcDiaqpZGhogkrpg58oFkTM5vUd9nLasG5nX3pmu17Yt4NjySxKGk4Pa4GurAnjHvFihcfBX6VGG5rT
Slot:        2
Status:      Ok
Fee:         0.000005 SOL
Program:     11111111111111111111111111111111 (System Program)
Instruction: Transfer (tag byte = 1)
  from:      8FaUm3F9HrgUAMZCfZ6UTfXejdwAJTxkTk4fba16VdCc (alice)
  to:        CTNVLq92hTWycPovppCxDNfLUMTqVD78Z3BqxdZM89LZ (bob)
  lamports:  500000000 (0.5 SOL)

Instruction data, 73 bytes:
  0000: 01 6b bd cb 50 58 79 d8 59 2f 5a d5 81 e1 24 13
  ...
  0040: 66 00 65 cd 1d 00 00 00 00
```

Тесты: `cargo test` — 45 passed, 0 failed.

## Команды

| Команда | Что делает | Аналог в Solana |
|---|---|---|
| `keygen <label>` | создаёт адрес и сохраняет его под меткой | `solana-keygen new` |
| `airdrop <SOL> <account>` | начисляет тестовые SOL (до 5 SOL, без комиссии) | `solana airdrop` |
| `transfer <from> <to> <SOL> [--skip-preflight]` | перевод с комиссией 0.000005 SOL | `solana transfer` |
| `balance <account>` | баланс | `solana balance` |
| `account <account>` | поля аккаунта + сырые байты | `solana account` |
| `accounts` | все аккаунты, общее предложение, сожжённые комиссии | — |
| `history [N]` | последние транзакции | история в Explorer |
| `tx <signature>` | разбор транзакции и байтов instruction data | Solana Explorer |
| `reset` | стереть всё состояние | `solana-test-validator --reset` |

## Какие конструкции Rust используются и зачем они в Solana

| Конструкция | Где в коде | Где это встретится в Solana |
|---|---|---|
| `struct`, newtype `Pubkey([u8; 32])` | `pubkey.rs`, `ledger.rs` | `Pubkey`, `Account`, структуры данных аккаунтов |
| `enum` с данными + `match` | `Instruction`, `TxStatus`, `Command` | инструкции программы, `SystemInstruction` |
| `Result<T, E>`, свой enum ошибок, `?`, `From` | `error.rs` | `ProgramResult`, `ProgramError`, `#[error_code]` в Anchor |
| `Option<T>`, `?` на Option | `base58::decode`, `fee_payer()` | `accounts.get()`, `checked_*` |
| Трейты, методы по умолчанию, generics с ограничениями | `trait Pack`, `impl<T: Pack> Pack for Vec<T>` | `BorshSerialize` / `BorshDeserialize`, `AnchorSerialize` |
| Const generics | `impl<const N: usize> Pack for [u8; N]` | массивы фиксированной длины в данных аккаунтов |
| Сериализация little-endian, `&mut &[u8]` | `pack.rs` | Borsh, instruction data, данные аккаунтов |
| `split_first()` для тега инструкции | `instruction.rs` | `Instruction::unpack` в native-программах |
| Lifetimes (`'a`) | `pack::take<'a>` | `AccountInfo<'a>`, `Context<'_, '_, '_, 'info, T>` в Anchor |
| Ownership и borrowing, `&mut` | `execute_instruction`, `credit` | изменение `lamports`/`data` через `AccountInfo` |
| `checked_add` / `checked_sub` против переполнения | `ledger.rs`, `amount.rs` | обязательная практика безопасности программ |
| `wrapping_*` (там, где переполнение — часть алгоритма) | `pubkey::expand_seed` | хэши, генераторы |
| Целочисленная арифметика вместо `f64` | `amount.rs` | лампорты, децималы SPL-токенов |
| `HashMap`, `BTreeMap`, Entry API | `ledger.rs` | хранилище аккаунтов, поиск по ключу |
| Итераторы и замыкания (`filter`, `map`, `try_fold`, `rev`, `take`) | `ledger.rs`, `main.rs` | обработка списков аккаунтов, клиентский код |
| Pattern matching по срезам, guard'ы | `Command::parse`, `find_transaction` | разбор `accounts: &[AccountInfo]` |
| `macro_rules!` | `impl_pack_for_int!`, `impl_base58_fmt!` | `declare_id!`, `msg!`, `require!` и макросы Anchor |
| Модули (`mod`) и видимость (`pub`) | 7 модулей | структура программы: `instruction`, `processor`, `state`, `error` |
| `#[derive(...)]` | почти везде | `#[derive(BorshSerialize, BorshDeserialize)]`, `#[derive(Accounts)]` |
| Unit-тесты `#[cfg(test)]` | в каждом модуле | тестирование программ |
| `#![forbid(unsafe_code)]`, `overflow-checks = true` | `main.rs`, `Cargo.toml` | настройки безопасности, как в шаблоне Anchor |

## Как это связано с устройством Solana

- **Кошелёк и аккаунт — разные вещи.** `keygen` только создаёт адрес. On-chain аккаунт появляется
  при первом поступлении лампортов, как у System Program.
- **Instruction data — это просто байты.** Клиент сериализует `Instruction` в `Vec<u8>`
  (1 байт тега + поля), "рантайм" получает `&[u8]` и декодирует обратно. Команда `tx` показывает эти байты.
- **Транзакции атомарны.** Инструкция выполняется на копии состояния. При ошибке всё откатывается,
  кроме комиссии.
- **Preflight.** По умолчанию падающая транзакция отклоняется ещё на симуляции, без комиссии.
  С `--skip-preflight` она попадает в блок, падает и всё равно платит 5000 лампортов.
- **Детерминизм.** Одно и то же состояние всегда даёт одни и те же байты (ключи HashMap сортируются, как в Borsh).
- **Формат данных совместим с Borsh** для используемых типов: `u64` little-endian, `bool` 1 байт, `String`/`Vec` = длина `u32` + элементы.

## Структура

```
src/
  main.rs         CLI: разбор аргументов, команды, вывод
  ledger.rs       аккаунты, выполнение транзакций, история, сохранение в файл
  instruction.rs  enum инструкций и их байтовый формат
  pack.rs         трейт Pack (мини-Borsh) и реализации для базовых типов
  pubkey.rs       Pubkey и Signature, генерация адресов
  base58.rs       кодирование и декодирование base58
  amount.rs       SOL <-> лампорты без float
  error.rs        enum ошибок
```

## Упрощения по сравнению с настоящей Solana

Адреса и подписи генерируются псевдослучайно, без ed25519. Хранилище локальное (файл), сети и валидаторов нет.
В одной транзакции одна инструкция. Комиссия сжигается полностью, rent не моделируется.
