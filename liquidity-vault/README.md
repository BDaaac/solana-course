# Liquidity Vault

Учебный DeFi-vault на Solana (Anchor 0.31.1, devnet). Пользователи вносят токены и получают share-токены — долю в общем котле.
Администратор обменивает токены через Raydium и размещает их в CLMM-пуле в выбранном ценовом диапазоне, собирает комиссии и
переставляет позицию. За основу взят шаблон [`0xnurda/vault-dev` (ветка `students-vault-card`)](https://github.com/0xnurda/vault-dev/tree/students-vault-card).

- **Программа (devnet):** [`8wmtpwX7UZbwEUhNFobRdMRmtyRsUgtWaFwV5GBff8J6`](https://explorer.solana.com/address/8wmtpwX7UZbwEUhNFobRdMRmtyRsUgtWaFwV5GBff8J6?cluster=devnet)
- **Токен vault (VLT):** [`DDuiXekcXiGckNegNd8vWmbirpPYq41EDTAy9fBHD3sf`](https://explorer.solana.com/address/DDuiXekcXiGckNegNd8vWmbirpPYq41EDTAy9fBHD3sf?cluster=devnet)
- **Второй токен пула (QTE):** [`JE7TnWNzdpSjefeW9tzp7qJiEqheyktowzr5hxsP6VxT`](https://explorer.solana.com/address/JE7TnWNzdpSjefeW9tzp7qJiEqheyktowzr5hxsP6VxT?cluster=devnet)
- **CLMM-пул Raydium:** [`Bvzzmc49b2iGy8f3FHTEGbFkmFjZVpQwF7TdCuznJsDg`](https://explorer.solana.com/address/Bvzzmc49b2iGy8f3FHTEGbFkmFjZVpQwF7TdCuznJsDg?cluster=devnet) (tickSpacing 10, комиссия 0,05 %)

Ключей и seed-фраз в репозитории нет. Только devnet.

## Схема аккаунтов

| Аккаунт | Адрес / seeds | Владелец | Что хранит |
|---|---|---|---|
| `VaultState` | PDA `["vault", token_mint]` — [`7DDcUz…ZLk8`](https://explorer.solana.com/address/7DDcUzSKoxNEkkWFRpMJbgBnBTCorQJ9mRS1ZHP6ZLk8?cluster=devnet) | программа vault | admin, token_mint, адреса хранилища и share-mint, `total_tokens`, `total_shares`, `paused`, bump |
| Хранилище токенов | PDA `["vault_tokens", token_mint]` — [`C3tukc…gxQb`](https://explorer.solana.com/address/C3tukcbMpsArb47sF3QrXBjm9MLrL6HQPk5VuKVHgxQb?cluster=devnet) | token-account, authority = `VaultState` | внесённые токены |
| Share-mint | PDA `["share_mint", token_mint]` — [`6ixhmV…YTpG`](https://explorer.solana.com/address/6ixhmV9PSMTzHL2q8vZtr8BCTAsmcNGkrrdEsgKuYTpG?cluster=devnet) | mint authority = `VaultState` (6 decimals) | квитанции-доли |
| `UserCard` | PDA `["user_card", vault, user]` | программа vault | vault, owner, `deposited`, `created_at`, bump |
| Хранилище токена B | token-account с authority = `VaultState` — [`AsehvJ…ceqP4`](https://explorer.solana.com/address/AsehvJhviQgnPrZyymJUTci26mkULjKF9bTtKYBceqP4?cluster=devnet) | token-account | второй токен пула (после swap) |
| NFT позиции Raydium | Token-2022 NFT, на счёте `VaultState` | Raydium + vault | право на позицию в CLMM-пуле |

```
 пользователь ──deposit──▶ хранилище токенов ◀── vault PDA (подпись seeds)
      │                          │
      └──◀── share-токены ◀── mint (authority = vault PDA)
                                 │
          admin ──swap/open/collect/close──▶ CPI в Raydium CLMM ──▶ пул Raydium
```

Адрес карточки вычисляется из адреса пользователя и vault, поэтому её всегда можно найти без списка пользователей.
Никто, включая администратора, не может забрать токены подписью: хранилищем и share-mint распоряжается только программа (PDA).

## Инструкции

| Инструкция | Кто вызывает | Неделя | Что делает |
|---|---|---|---|
| `initialize_vault` | admin | 8 | создаёт VaultState, хранилище и share-mint |
| `create_user_card` / `close_user_card` | пользователь | 8 | карточка PDA; закрыть можно, только если вклад выведен; при паузе создать нельзя |
| `set_paused` | admin | 8 | пауза vault |
| `deposit(amount)` | пользователь | 9 | перевод токенов (подпись пользователя) + выпуск shares (подпись PDA); ноль запрещён |
| `withdraw(shares)` | пользователь | 10 | burn shares (подпись пользователя) + перевод токенов (подпись PDA) |
| `sync_balance` | admin | 13 | `total_tokens` = реальный баланс хранилища (после возврата капитала из пула) |
| `swap_via_raydium(amount_in, min_amount_out, …)` | admin | 12 | swap через CLMM-пул, `min_amount_out > 0` обязателен |
| `open_raydium_position(…)` | admin | 13 | открывает позицию в диапазоне; NFT принадлежит vault |
| `collect_fees` | admin | 13 | забирает комиссии позиции (decrease_liquidity с liquidity = 0) |
| `close_raydium_position(min0, min1)` | admin | 13 | выводит всю ликвидность с минимумом и сжигает NFT |

**Формулы.** Первый депозит: `shares = amount`. Далее `shares = amount × total_shares / total_tokens`.
Вывод: `tokens = shares × total_tokens / total_shares`. Вычисления в `u128` с `checked_*`.

**Упрощение учебной версии.** Пока средства в позиции, вывод пользователей ограничен балансом хранилища (`InsufficientLiquidity`).
Администратор закрывает позицию, при необходимости меняет токен B обратно в A, вызывает `sync_balance` — и комиссии удорожают share.

## Запуск

Требования: Rust, Solana CLI (Agave 3.x), Anchor 0.31.1, Node 22. Сборка (`Cargo.lock` с закреплёнными версиями лежит в репозитории):

```bash
cargo build-sbf --manifest-path programs/simple_vault/Cargo.toml
solana program deploy target/deploy/simple_vault.so --program-id target/deploy/simple_vault-keypair.json
anchor idl build -p simple_vault -o idl/simple_vault.json
```

Клиенты (корень репозитория, кошелёк `~/.config/solana/id.json`, devnet; каждый скрипт — интеграционный тест с проверками и отрицательными сценариями):

```bash
npm install
npm run vault:basic   # недели 8–10: каркас, депозит, вывод
npm run vault:pool    # неделя 11: токены, CLMM-пул, ручная позиция
npm run vault:swap    # неделя 12: swap через vault + отклонённый из-за slippage
npm run vault:position # неделя 13: позиция, комиссии, закрытие, ребалансировка
```

Результаты прогона — в [`state.json`](state.json). Веб-интерфейс — [`app/`](app/) (`npm install && npm run dev`).

## Задания по неделям

- [Неделя 8 — каркас vault](../week8-vault-skeleton/README.md)
- [Неделя 9 — депозит и share-токены](../week9-deposit-shares/README.md)
- [Неделя 10 — CPI и вывод](../week10-cpi-withdraw/README.md)
- [Неделя 11 — AMM, CLMM-пул Raydium](../week11-clmm-pool/README.md)
- [Неделя 12 — swap через Raydium](../week12-swap/README.md)
- [Неделя 13 — позиция в CLMM-пуле](../week13-clmm-position/README.md)
- [Неделя 14 — Capstone: интерфейс и README](../week14-capstone/README.md)
- [Неделя 15 — Demo Day](../week15-demo-day/README.md)
