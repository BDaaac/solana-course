# Неделя 8. Каркас Liquidity Vault (РК1)

**Задание:** собрать каркас vault: аккаунт настроек, PDA пользователя («карточка»), share-токен, хранилище токенов. Задеплоить в devnet.

Начато с шаблона [`0xnurda/vault-dev`](https://github.com/0xnurda/vault-dev/tree/students-vault-card) (ветка `students-vault-card`: `initialize_vault` + `create_user_card`).
Дальше выполнены практические задания из README шаблона. Код — [`../liquidity-vault/programs/simple_vault/src/lib.rs`](../liquidity-vault/programs/simple_vault/src/lib.rs).

## Схема аккаунтов

| Аккаунт | Seeds | Владеет | Хранит |
|---|---|---|---|
| `VaultState` | `["vault", token_mint]` | программа | admin, token_mint, адреса хранилища и share-mint, `total_tokens`, `total_shares`, `paused`, bump |
| Хранилище токенов | `["vault_tokens", token_mint]` | PDA `VaultState` (authority) | внесённые токены |
| Share-mint | `["share_mint", token_mint]` | PDA `VaultState` (mint authority) | share-токены, 6 decimals |
| `UserCard` | `["user_card", vault, user]` | программа | vault, owner, `deposited`, `created_at`, bump |

## Что добавлено к шаблону

1. `created_at` в `UserCard` (`Clock::get()?.unix_timestamp`).
2. `close_user_card` — закрывает карточку и возвращает ренту; нельзя, пока есть вклад (`CardHasDeposit`).
3. Флаг `paused` + `set_paused` (только admin); при паузе нельзя создавать карточки и вносить токены (`VaultPaused`).
4. Счётчики `total_tokens` / `total_shares` в `VaultState` (понадобятся с недели 9).

## Результат в devnet

- Программа: [`8wmtpwX7…8J6`](https://explorer.solana.com/address/8wmtpwX7UZbwEUhNFobRdMRmtyRsUgtWaFwV5GBff8J6?cluster=devnet)
- [`VaultState`](https://explorer.solana.com/address/7DDcUzSKoxNEkkWFRpMJbgBnBTCorQJ9mRS1ZHP6ZLk8?cluster=devnet), [хранилище](https://explorer.solana.com/address/C3tukcbMpsArb47sF3QrXBjm9MLrL6HQPk5VuKVHgxQb?cluster=devnet), [share-mint](https://explorer.solana.com/address/6ixhmV9PSMTzHL2q8vZtr8BCTAsmcNGkrrdEsgKuYTpG?cluster=devnet)
- [`initialize_vault`](https://explorer.solana.com/tx/FetPv1sMuWkQcVpoHTbGbgJv3NSghaQ8Tx4yE36RbWVjbSjEdAbaDHWBpYBquDiT3fCxftAxNXcETZrqqbeQWWs?cluster=devnet)
- карточки: [admin](https://explorer.solana.com/address/BgBRFiaS2czXA5JgdKnTptxZnc6EcVyjgwDM4f1c3Dnp?cluster=devnet) ([tx](https://explorer.solana.com/tx/3p6zFbx1337LA1N2K4QuFNwTw8DRvE9JM6vqhmrrPhFkFU3DwCnNaqGG7UiQVuPGKAk6bes994T4pVBbr9V3FbXa?cluster=devnet)), [user](https://explorer.solana.com/address/4cRpxdeg9Apaw6oqdw4Gi9VRRNKFACxZdFJRXEaVJsxx?cluster=devnet) ([tx](https://explorer.solana.com/tx/UjkPEaXih2bMzuiydZSBTzRqbdoBf8DzCk1VEX5zBRWFe4kgtAFTX3AqN6JTuZFxcKB7FJifFNeBWMnxBhAyBFV?cluster=devnet))

Проверки (`npm run vault:basic`): повторная карточка того же пользователя → `already in use`; создание карточки при паузе → `VaultPaused`;
закрытие карточки с вкладом → `CardHasDeposit`.

## Проверьте себя

- **Почему share-токен выпускает программа, а не администратор?** Mint authority — PDA `VaultState`. Тогда выпуск возможен только внутри `deposit`, после реального перевода токенов; администратор не может напечатать себе доли и забрать чужие деньги.
- **Что такое PDA и почему у неё нет приватного ключа?** Адрес, выведенный из seeds и program id, который намеренно лежит *вне* кривой ed25519: подходящего приватного ключа не существует. Подписать за PDA может только программа через `invoke_signed` с теми же seeds.
- **Как найти карточку конкретного пользователя?** Пересчитать PDA: `findProgramAddress(["user_card", vault, user], program_id)`. Список пользователей хранить не нужно.
