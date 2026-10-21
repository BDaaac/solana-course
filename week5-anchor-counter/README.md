# Неделя 5. Anchor Framework: Counter Program

**Задание:** создать Counter Program и взаимодействовать с ней через client.

## Программа (Anchor 1.2)

| Инструкция | Что делает |
|---|---|
| `initialize()` | создаёт PDA-счётчик `["counter", authority]` (`init`, payer, space через `InitSpace`) |
| `increment(step)` | +step, `checked_add` |
| `decrement(step)` | −step, ниже нуля нельзя (`CounterUnderflow`) |
| `reset()` | обнулить |

Что показывает код:

- `#[program]`, `#[derive(Accounts)]`, `#[account]`, `#[error_code]`, `require!`;
- ограничения `seeds` + `bump` + `has_one = authority`: менять счётчик может только владелец;
- у каждого кошелька свой счётчик (PDA от ключа владельца);
- поле `total_updates` хранит историю изменений.

**Тесты** (`programs/week5-anchor-counter/tests`, LiteSVM, 3 штуки): полный цикл, повторный `initialize`, попытка чужого кошелька изменить счётчик.

**Клиент** `client/client.ts` (`@anchor-lang/core` + IDL):
1. `initialize` (при повторном запуске `reset`).
2. `increment(5)`, `increment(3)`, `decrement(2)`: итог 6.
3. `decrement(1000)` отклоняется с ошибкой `CounterUnderflow`.
4. Состояние читается через `program.account.counter.fetch`.

Результаты с реальными транзакциями devnet — в `result.json`.

## Запуск

```bash
anchor build -- --arch v0   # или cargo build-sbf --arch v0
cargo test
solana program deploy target/deploy/week5_anchor_counter.so --program-id target/deploy/week5_anchor_counter-keypair.json
cd .. && npm run week5
```
