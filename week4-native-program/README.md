# Неделя 4. Первая Solana Program (native, без Anchor)

**Задание:** создать и задеплоить первую Solana Program в Devnet.

## Программа `week4_note`

On-chain заметка в аккаунте, которым владеет программа.

| Инструкция | Тег | Что делает |
|---|---|---|
| `Initialize { text }` | 0 | записывает первую заметку и запоминает автора |
| `Update { text }` | 1 | меняет текст; может только автор |

Что показывает код (`program/src/lib.rs`):

- `entrypoint!` и сигнатура `process_instruction(program_id, accounts, instruction_data)`;
- instruction data в формате Borsh: 1 байт тега + поля;
- проверки безопасности: owner аккаунта == program_id, аккаунт writable, автор подписал, автор совпадает;
- свои коды ошибок (`ProgramError::Custom`), `checked_add`, `msg!`-логи;
- 3 unit-теста на формат данных.

Клиент `client/client.ts` делает четыре шага:
1. Создаёт аккаунт (`SystemProgram.createAccount`, owner = программа) и вызывает `Initialize` в той же транзакции.
2. Вызывает `Update`.
3. Проверяет, что чужой ключ не может изменить заметку: получает ошибку `custom program error: 0x2`.
4. Читает данные аккаунта и декодирует их вручную.

Program ID, ссылки на транзакции и итоговое состояние лежат в `result.json`.

## Сборка и деплой

```bash
cd program
cargo build-sbf --arch v0
cargo test
solana program deploy target/deploy/week4_note.so --program-id target/deploy/week4_note-keypair.json
cd ../.. && npm run week4
```
