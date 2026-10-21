# Solana-курс: задания недель 1–15

Все программы задеплоены в **Devnet**, все клиенты запускались на реальной сети. Недели 8–15 — сквозной проект **Liquidity Vault** (код в [`liquidity-vault/`](liquidity-vault/README.md)).
Ссылки ниже открываются в Solana Explorer.

| Неделя | Тема | Результат |
|---|---|---|
| 1 | Первая транзакция | [перевод + memo](https://explorer.solana.com/tx/FdSTfbQvF54s7exsL8rdAo3a1oMZk2ppUX3HMG8zW1fJywheuAB8QaTg8DFgQHUXwxqs7cKLtiAPEhJqAzqwp1Q?cluster=devnet), разбор в `week1-first-transaction/result.json` |
| 2 | Transaction Detective | разбор 3 реальных транзакций mainnet (Jupiter swap, Memo, ATA): `week2-transaction-detective/REPORT.md` |
| 3 | Rust CLI | `solcli`: мини-симулятор Solana, 45 unit-тестов |
| 4 | Native Solana Program | Program ID [`6oQ1riuo8Pnvo3abyGvy6MHz4BFP6Hjw5JpQAtU2rRmT`](https://explorer.solana.com/address/6oQ1riuo8Pnvo3abyGvy6MHz4BFP6Hjw5JpQAtU2rRmT?cluster=devnet) |
| 5 | Anchor Counter | Program ID [`7nnPuXhhqA7S9CwG3zu5qLapAys3fRViLWx8MLYRHryv`](https://explorer.solana.com/address/7nnPuXhhqA7S9CwG3zu5qLapAys3fRViLWx8MLYRHryv?cluster=devnet) |
| 6 | On-chain профиль на PDA | Program ID [`ABcoiQozKZMZ7uiUdha1ip8Z6zDpekTP9iifUeyGcsS1`](https://explorer.solana.com/address/ABcoiQozKZMZ7uiUdha1ip8Z6zDpekTP9iifUeyGcsS1?cluster=devnet) |
| 7 | Токен на Token-2022 + metadata | Mint [`EGAq25Rh7Ry28cpXfsj2rkHm8qYMRQ55G1UuZNYDrvFX`](https://explorer.solana.com/address/EGAq25Rh7Ry28cpXfsj2rkHm8qYMRQ55G1UuZNYDrvFX?cluster=devnet) (SCT) |
| 8 | Каркас Liquidity Vault (РК1) | [`week8-vault-skeleton`](week8-vault-skeleton/README.md): программа [`8wmtpwX7…8J6`](https://explorer.solana.com/address/8wmtpwX7UZbwEUhNFobRdMRmtyRsUgtWaFwV5GBff8J6?cluster=devnet), PDA vault / хранилище / share-mint / карточка |
| 9 | Депозит и share-токены | [`week9-deposit-shares`](week9-deposit-shares/README.md): [депозит](https://explorer.solana.com/tx/3FUnVf7ZSbvC7V7pWbTQuzHJYun3KH3QiN6bwf172bKEsY3vLdGSyHyWyPy2mAdmty64AQqapJsP2xcZ88ynyTaL?cluster=devnet) (110 токенов → 100 shares) |
| 10 | CPI и вывод | [`week10-cpi-withdraw`](week10-cpi-withdraw/README.md): [вывод](https://explorer.solana.com/tx/4WMcPsgQEm6Pr7avnmaGE8pANMH2ovqPfz3vjuiDM9wsf9GRhDEd8t9BgBfxtgS2q1SEMvRiGjksn9J2Jfy4S9Sg?cluster=devnet) (burn + transfer с подписью PDA) |
| 11 | AMM и CLMM-пул Raydium | [`week11-clmm-pool`](week11-clmm-pool/README.md): пул [`Bvzzmc49…JsDg`](https://explorer.solana.com/address/Bvzzmc49b2iGy8f3FHTEGbFkmFjZVpQwF7TdCuznJsDg?cluster=devnet) |
| 12 | Swap через Raydium | [`week12-swap`](week12-swap/README.md): [успех](https://explorer.solana.com/tx/eHr8eY8eGFkuD6UGTnQwF4FpkzCNAWtPqeSNS53HJEskmppPYWYTMeZGTKZ3UtZYzphiSr59ZgiJAbgqTCcVvab?cluster=devnet) и [отказ по slippage](https://explorer.solana.com/tx/2kKYkBxm2FoAcHozSzU8uTQj9BdTB9SWvEARgh6GVAu6PQzcJnDaU358BWoMXzLVkAYVTorDHbfsKwQspTKFipJB?cluster=devnet) |
| 13 | Позиция в CLMM-пуле | [`week13-clmm-position`](week13-clmm-position/README.md): открытие, комиссии, закрытие, ребалансировка |
| 14 | Capstone | [`week14-capstone`](week14-capstone/README.md): веб-интерфейс [`liquidity-vault/app`](liquidity-vault/app) |
| 15 | Demo Day (РК2) | [`week15-demo-day`](week15-demo-day/README.md): сценарий выступления, ответы жюри |

Подробности по каждой неделе — в `weekN-*/README.md`, реальные подписи и состояние аккаунтов — в `weekN-*/result.json`.

## Проверки

| Что | Результат |
|---|---|
| Неделя 3, `cargo test` | 45 passed |
| Неделя 4, `cargo test` | 3 passed |
| Неделя 5, `cargo test` (LiteSVM) | 3 passed |
| Неделя 6, `cargo test` (LiteSVM) | 3 passed |
| TypeScript, `tsc --noEmit` | без ошибок |
| Негативные сценарии в devnet | чужой ключ не меняет заметку (нед. 4); `decrement` ниже нуля → `CounterUnderflow` (нед. 5); повторный username → отказ (нед. 6) |

## Стек

Solana CLI (Agave) 4.3, Anchor 1.2 (`@anchor-lang/core`), Rust 1.89+ (недели 1–7); недели 8–15: Anchor 0.31.1, Agave 3.0, `@raydium-io/raydium-sdk-v2`, `@solana/web3.js` 1.x, `@solana/spl-token`, Node 22.
Программы собраны `cargo build-sbf --arch v0`.

## Запуск клиентов

```bash
npm install
npm run week1        # первая транзакция
npm run week2:report # отчёт по mainnet
npm run week4        # native-программа
npm run week5        # counter
npm run week6        # профиль
npm run week7        # токен
npm run vault:basic  # недели 8–10
npm run vault:pool   # неделя 11
npm run vault:swap   # неделя 12
npm run vault:position # неделя 13
```
Клиенты используют кошелёк `~/.config/solana/id.json` и devnet.
