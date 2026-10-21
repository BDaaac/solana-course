# Неделя 7. SPL Tokens и Token Extensions

**Задание:** создать собственный токен: выпуск, transfer, metadata и взаимодействие через client.

## Токен Solana Course Token (SCT)

Токен выпущен на **Token-2022** (Token Extensions) с двумя расширениями:

- **MetadataPointer** — mint указывает, где лежат метаданные (на самого себя);
- **TokenMetadata** — name, symbol, uri и дополнительное поле `course` хранятся прямо в аккаунте mint. Metaplex не нужен.

Что делает `token.ts`:

1. **Выпуск mint.** В одной транзакции:
   - `createAccount` с размером под расширения;
   - `initializeMetadataPointer`;
   - `initializeMint` (6 decimals);
   - `initialize` метаданных;
   - `updateField("course", ...)`.
2. **Mint.** Создаёт Associated Token Account владельца и выпускает 1 000 000 SCT.
3. **Transfer.** Создаёт ATA нового получателя и переводит 1 500 SCT через `transferChecked` (сверяет decimals).
4. **Client.** Читает mint (`getMint`), метаданные (`getTokenMetadata`) и балансы обоих ATA.

Адрес mint, метаданные, балансы и ссылки на все транзакции — в `result.json`.

## Запуск

```bash
npm run week7
```
