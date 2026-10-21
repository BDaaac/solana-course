# Неделя 12. Swap через Raydium

**Задание:** обменять токены из vault через CLMM-пул Raydium с ограничением на минимальную сумму получения (slippage).

Код: [`instructions/swap.rs`](../liquidity-vault/programs/simple_vault/src/instructions/swap.rs), клиент [`03-swap.ts`](../liquidity-vault/scripts/03-swap.ts) (`npm run vault:swap`).

## Как устроено

- Подключён крейт [`raydium-clmm-cpi`](https://github.com/raydium-io/raydium-cpi) (ветка `anchor-0.31.1`, feature `devnet`).
- У vault появилось второе хранилище — токен-аккаунт токена B с authority = PDA `VaultState` ([`AsehvJ…ceqP4`](https://explorer.solana.com/address/AsehvJhviQgnPrZyymJUTci26mkULjKF9bTtKYBceqP4?cluster=devnet)).
- `swap_via_raydium(amount_in, min_amount_out, sqrt_price_limit)`:
  - только admin (`Unauthorized` для остальных);
  - `min_amount_out > 0` обязателен (`NoSlippageProtection`);
  - CPI в `swap_v2` Raydium, плательщик/владелец входного счёта — PDA `VaultState`, который подписывает через `invoke_signed`;
  - tick-массивы пула и bitmap передаются как `remaining_accounts`.
- Перед обменом клиент считает ожидаемую сумму (`PoolUtils.computeAmountOutFormat`) и ставит минимум на 1 % ниже.

## Результаты в devnet

Обмен 100 VLT → QTE:

| Параметр | Значение |
|---|---|
| Ожидалось | 99,940637 QTE |
| Минимум (−1 %) | 98,941230 QTE |
| Получено | 99,940637 QTE |
| Успешный обмен | [tx](https://explorer.solana.com/tx/eHr8eY8eGFkuD6UGTnQwF4FpkzCNAWtPqeSNS53HJEskmppPYWYTMeZGTKZ3UtZYzphiSr59ZgiJAbgqTCcVvab?cluster=devnet) |
| Отклонённый из-за slippage (минимум завышен в 2 раза) | [tx](https://explorer.solana.com/tx/2kKYkBxm2FoAcHozSzU8uTQj9BdTB9SWvEARgh6GVAu6PQzcJnDaU358BWoMXzLVkAYVTorDHbfsKwQspTKFipJB?cluster=devnet) — ошибка Raydium `6018 TooLittleOutputReceived` |
| Цена пула до → после | 1 → 0,99981 |

Дополнительно: попытка swap не админом → `Unauthorized`.

## Проверьте себя

- **Что такое slippage и как от него защищаются?** Разница между ожидаемой и фактической ценой обмена: пока транзакция идёт в сеть, цена может сдвинуться. Защита — минимум на выходе (`min_amount_out`): если пул отдаёт меньше, обмен откатывается.
- **Почему цена сдвигается сильнее, если около текущей цены мало ликвидности?** Обмен «проходит» по тикам, пока не наберёт нужную сумму; при малой ликвидности на тик приходится меньше объёма, и цена уходит дальше.
- **Почему обмен подписывает программа, а не администратор?** Токены принадлежат PDA vault, а не кошельку админа. Админ лишь авторизует операцию (проверка в программе), а подпись над токенами ставит программа.
