# Неделя 10. CPI и вывод из vault

**Задание:** share-токены сжигаются, vault возвращает токены пользователю через Token Program с подписью PDA.

Код: инструкция `withdraw` в [`lib.rs`](../liquidity-vault/programs/simple_vault/src/lib.rs).

## Как работает

1. Пользователь указывает, сколько share-токенов возвращает.
2. `tokens = shares × total_tokens / total_shares` (`u128`, `checked_*`; результат > 0, иначе `WithdrawTooSmall`).
3. `token::burn` share-токенов — **подписывает пользователь**.
4. `token::transfer` из хранилища пользователю — **подписывает PDA vault** (`invoke_signed`, seeds `["vault", token_mint, bump]`).
5. Учёт: `total_tokens`, `total_shares`, `deposited` в карточке.
6. Дополнительно: если в хранилище меньше токенов, чем положено (часть работает в пуле) — `InsufficientLiquidity`.

## Результат в devnet

Пользователь возвращает 50 share-токенов (из 100) при цене share 1,1 — [транзакция вывода](https://explorer.solana.com/tx/4WMcPsgQEm6Pr7avnmaGE8pANMH2ovqPfz3vjuiDM9wsf9GRhDEd8t9BgBfxtgS2q1SEMvRiGjksn9J2Jfy4S9Sg?cluster=devnet): получено **55 токенов** (скрипт сверяет сумму).

### Какие CPI-вызовы видны в транзакции вывода

Во вкладке *Inner Instructions* у инструкции `withdraw` программы vault — два вызова **Token Program**:

1. `Burn` — сжигание shares, authority = пользователь (его подпись «передана» дальше по цепочке);
2. `Transfer` — токены из хранилища vault пользователю, authority = PDA `VaultState` (подпись даёт сама программа через `invoke_signed`).

Проверено программно: `meta.innerInstructions` = `[burn, transfer]`.

## Проверьте себя

- **Что такое CPI и зачем он нужен?** Вызов одной программы из другой. Наша программа не умеет двигать токены сама — она просит Token Program. Так же vault вызывает Raydium (недели 12–13).
- **Когда подписывает пользователь, а когда программа?** Пользователь — когда двигаются его токены (burn shares, transfer из его счёта). Программа от имени PDA — когда двигаются токены vault (transfer из хранилища, mint_to).
- **Почему админ не может забрать токены своей подписью?** Authority хранилища — PDA, у которой нет ключа. Единственный путь — инструкции программы, а они выдают токены только против сожжённых shares.
