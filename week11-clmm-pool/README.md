# Неделя 11. AMM и концентрированная ликвидность (Raydium CLMM)

**Задание:** создать два своих тестовых токена и свой CLMM-пул Raydium в devnet, открыть вручную одну позицию вокруг цены.

Скрипт: [`02-pool.ts`](../liquidity-vault/scripts/02-pool.ts) (`npm run vault:pool`), на основе `@raydium-io/raydium-sdk-v2` (devnet, программа CLMM `DRayAUgENGQBKVaX8owNhgzkEDyoHTGVEGHVJT1E9pfH`).

## Адреса

| Что | Адрес |
|---|---|
| Токен A (VLT, он же токен vault) | [`DDuiXekc…D3sf`](https://explorer.solana.com/address/DDuiXekcXiGckNegNd8vWmbirpPYq41EDTAy9fBHD3sf?cluster=devnet) |
| Токен B (QTE) | [`JE7TnWNz…6VxT`](https://explorer.solana.com/address/JE7TnWNzdpSjefeW9tzp7qJiEqheyktowzr5hxsP6VxT?cluster=devnet) |
| CLMM-пул | [`Bvzzmc49…JsDg`](https://explorer.solana.com/address/Bvzzmc49b2iGy8f3FHTEGbFkmFjZVpQwF7TdCuznJsDg?cluster=devnet), [создание](https://explorer.solana.com/tx/3J64Az1AHqmmbbLbtvJhYi23zcyqihh8dUo1J6frmU8w35AGEzXSXtzHYwxwF5u7pDRBMkjmAN69faZua8xkjMV2?cluster=devnet) |
| AmmConfig | `FZdkW5jiYsjTnCVqFqPrxrQisQkCYrohd7ArZhoKnM8q` (tickSpacing 10, комиссия 0,05 %) |
| Ручная позиция (NFT) | [`DXaf3hr5…8bGhG`](https://explorer.solana.com/address/DXaf3hr5mj3yTWHvB2xJyj4ugTLExX678TXeHms8bGhG?cluster=devnet), [tx](https://explorer.solana.com/tx/2U49oUtrX8s5Q2xSdMes37pH75G4zCjphpAZviaXrQn4bLWYHvt9Wpq68SwpkrQk65sT4LLcg7Fg1upd6u95H7E3?cluster=devnet) |

Токен A в паре — token0 пула (адрес mint меньше), поэтому цена пула = токенов B за 1 токен A.

## Цена и диапазон

- **Начальная цена пула:** 1 (1 VLT = 1 QTE).
- **Диапазон ручной позиции:** 0,9 … 1,1 (тики −1060 … 960), то есть ±10 %. Внесено 50 000 VLT и ≈ 55 072 QTE.
- **Почему такой диапазон:** это тестовая пара «стейбл к стейблу» с целевой ценой 1. Цена будет колебаться в пределах нескольких процентов, поэтому ±10 % — запас, при котором позиция почти всегда в диапазоне и обеспечивает глубокую ликвидность около 1. Это «подушка» пула: поверх неё vault открывает более узкие позиции (±5 %) на неделе 13. Если сделать диапазон уже, то на тех же деньгах доход был бы выше, но цена чаще выходила бы из диапазона.

## Проверьте себя

- **Чем концентрированная ликвидность отличается от простого пула?** В простом пуле (x·y = k) ликвидность размазана по всем ценам от 0 до ∞, и большая часть денег никогда не используется. В CLMM провайдер сам выбирает диапазон и вкладывает деньги только в него, поэтому та же сумма даёт больше глубины и комиссий.
- **Что происходит с позицией, когда цена выходит за диапазон?** Она перестаёт зарабатывать, а весь капитал оказывается в одном токене. На неделе 13 это видно в цифрах: цена упала ниже диапазона, и позиция из 80 A + 84,2 B превратилась в 166,46 A.
- **Почему узкий диапазон зарабатывает больше, но требует больше внимания?** Те же деньги дают большую ликвидность в узкой полосе, значит больше доля комиссий — но цена быстрее покидает полосу, и позицию надо переставлять (ребалансировка).
