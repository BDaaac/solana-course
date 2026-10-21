# Неделя 6. Accounts, PDAs и архитектура программ

**Задание:** создать децентрализованный on-chain профиль с использованием PDA.

## Архитектура аккаунтов

| Аккаунт | PDA seeds | Назначение |
|---|---|---|
| `Profile` | `["profile", authority]` | один профиль на кошелёк: username, bio, avatar_url, created_at, updated_at |
| `Username` | `["username", username]` | реестр имён: уникальность username |

Уникальность имени обеспечивает сам адрес. Если имя занято, PDA `["username", name]` уже существует, и второй `init` падает. Отдельная проверка не нужна.

## Инструкции

- `create_profile(username, bio, avatar_url)` — создаёт оба PDA, валидирует имя (3–32 символа: a-z, 0-9, _) и длины строк, эмитит событие `ProfileCreated`.
- `update_profile(bio, avatar_url)` — только владелец (`seeds` + `has_one`).
- `delete_profile()` — `close = authority`: оба аккаунта закрываются, rent возвращается, имя освобождается.

Размер аккаунта считается через `#[derive(InitSpace)]` и `#[max_len]`, текущее время берётся из `Clock` sysvar.

**Тесты** (LiteSVM, 3 штуки): создание, обновление, удаление; уникальность и валидация имени; один профиль на кошелёк; чужой кошелёк не может изменить профиль.

**Клиент** `client/client.ts`:
1. Вычисляет PDA на клиенте (`findProgramAddressSync`).
2. Создаёт и обновляет профиль.
3. Проверяет, что другой кошелёк не может занять то же имя.
4. Читает профиль, а список всех профилей получает через `program.account.profile.all()`.

Результаты — в `result.json`.

## Запуск

```bash
cargo build-sbf --arch v0 --manifest-path programs/week6-anchor-profile/Cargo.toml
cargo test
solana program deploy target/deploy/week6_anchor_profile.so --program-id target/deploy/week6_anchor_profile-keypair.json
cd .. && npm run week6
```
