# Неделя 2. Как на самом деле работает Solana: Transaction Detective

**Задание:** разобрать реальные Solana-транзакции и определить accounts, instructions и programs.

## Что сделано

- `analyze.ts` — библиотека разбора транзакции:
  - **accounts** с ролями: fee payer, signer, writable или read-only, program, из Address Lookup Table;
  - **instructions**, включая вложенные вызовы (CPI) из `innerInstructions`;
  - **programs** с названиями известных программ (System, SPL Token, Token-2022, ATA, Memo, Compute Budget, Jupiter, Raydium, Orca и другие);
  - изменения балансов SOL и токенов, compute units, логи.
- `detective.ts` — CLI для любой подписи: `npm run week2 -- <signature> [devnet|mainnet]`.
- `investigate.ts` — находит свежие реальные транзакции mainnet трёх типов (swap через Jupiter, Memo, создание ATA), разбирает их и пишет отчёт **`REPORT.md`** с выводами.

## Главные выводы

- Транзакция = список инструкций + список всех аккаунтов, которые они трогают. Аккаунты объявляются заранее с флагами signer/writable. Благодаря этому рантайм Sealevel выполняет транзакции без пересекающихся writable-аккаунтов параллельно.
- Программы тоже аккаунты (executable) и в транзакции всегда read-only.
- Сложные операции (swap) — это цепочка CPI: Jupiter вызывает DEX (Raydium, Orca), тот вызывает SPL Token.
- Versioned-транзакции (v0) через Address Lookup Tables вмещают больше аккаунтов.
- Compute Budget задаёт лимит compute units и приоритетную комиссию.

## Запуск

```bash
npm run week2:report                                  # отчёт по mainnet
npm run week2 -- <signature> devnet                   # разбор любой транзакции
```
