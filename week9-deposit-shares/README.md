# Неделя 9. Депозит и share-токены

**Задание:** пользователь вносит токены в vault и получает share-токены (свою долю). Запретить внесение нуля.

Код: инструкция `deposit` в [`lib.rs`](../liquidity-vault/programs/simple_vault/src/lib.rs), клиент — [`01-vault.ts`](../liquidity-vault/scripts/01-vault.ts).

## Как работает

1. Расчёт: первый вкладчик `shares = amount`; дальше `shares = amount × total_shares / total_tokens` (в `u128`, с `checked_*`; результат > 0, иначе `SharesTooSmall`).
2. Перевод токенов пользователь → хранилище vault: CPI `token::transfer`, **подписывает пользователь**.
3. Выпуск share-токенов на кошелёк пользователя: CPI `token::mint_to`, **подписывает PDA vault** (`invoke_signed` с seeds `["vault", token_mint, bump]`).
4. Учёт: `total_tokens`, `total_shares` и `deposited` в карточке пользователя.
5. `amount == 0` → `ZeroAmount`.

## Пример из методички — воспроизведён в devnet

| Шаг | Результат |
|---|---|
| Админ вносит 1 000 токенов | [tx](https://explorer.solana.com/tx/47bjnGjiABSmyLNTF92Xfq2oPVhFGYLydTLGVVkJziZWBhnyRks6a37k92NYyE3z3StwkwFs1Sj6g4aECLWwJohR?cluster=devnet) → 1 000 shares |
| В vault «заработали» +100: `sync_balance` | [tx](https://explorer.solana.com/tx/2Xj3xinApyGvrzBrTGTXfbYiD7m2ATaW2nmxk83ZVDdGXaQ7ZLzZHeSAsoqWZHAa2VxaEZcwCAHq94XA3B9aiUy?cluster=devnet) → 1 100 токенов / 1 000 shares |
| Пользователь вносит **110** токенов | [tx](https://explorer.solana.com/tx/3FUnVf7ZSbvC7V7pWbTQuzHJYun3KH3QiN6bwf172bKEsY3vLdGSyHyWyPy2mAdmty64AQqapJsP2xcZ88ynyTaL?cluster=devnet) → ровно **100** shares |

Проверки: депозит нуля → `ZeroAmount`; `sync_balance` не админом → `Unauthorized`.

**Программы в транзакции депозита (Explorer):** моя программа vault → внутри два вызова Token Program (`transfer`, `mint_to`).

## Проверьте себя

- **Почему поздний вкладчик может получить share-токенов меньше, чем внёс токенов?** Share-токен «подорожал»: в котле 1 100 токенов на 1 000 shares, 1 share = 1,1 токена. 110 токенов дают 100 shares. Иначе новичок забрал бы часть дохода, заработанного до него.
- **Почему перевод подписывает пользователь, а выпуск share-токенов — программа?** Двигаются *его* токены — нужна его подпись. Mint authority принадлежит PDA vault — подписать может только программа.
- **Какие программы в транзакции?** Vault (`8wmtpwX7…`) → Token Program (`TokenkegQ…`, два inner-вызова: `transfer`, `mint_to`).
