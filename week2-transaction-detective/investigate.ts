// Разбирает несколько реальных транзакций mainnet разных типов
// и пишет отчёт REPORT.md: accounts, instructions, programs.
import { writeFileSync } from "node:fs";
import { Connection, PublicKey } from "@solana/web3.js";
import { MAINNET_URL } from "../common/solana.js";
import { Analysis, fetchAndAnalyze, toMarkdown } from "./analyze.js";

// Для каждого случая берём свежую успешную транзакцию, где участвует программа
const CASES = [
  { title: "Случай 1. Обмен токенов через Jupiter", address: "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4" },
  { title: "Случай 2. Memo-транзакция", address: "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr" },
  { title: "Случай 3. Создание Associated Token Account", address: "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL" },
];

const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

async function pickTransaction(connection: Connection, address: string): Promise<Analysis> {
  const signatures = await connection.getSignaturesForAddress(new PublicKey(address), { limit: 25 });
  for (const s of signatures.filter((s) => !s.err)) {
    try {
      const a = await fetchAndAnalyze(connection, s.signature, "mainnet");
      // Для наглядности берём не слишком огромную транзакцию
      if (a.accounts.length <= 30) return a;
    } catch {
      await sleep(1500); // публичный RPC ограничивает частоту запросов
    }
  }
  throw new Error(`no suitable transaction for ${address}`);
}

function conclusions(a: Analysis): string {
  const cpi = a.instructions.filter((ix) => ix.index.includes("CPI")).length;
  const writable = a.accounts.filter((acc) => acc.roles.includes("writable")).length;
  return [
    "",
    "**Выводы**",
    "",
    `- В транзакции ${a.accounts.length} аккаунтов: ${a.signers.length} подписант(ов), ${writable} writable, остальные read-only.`,
    `- Верхнеуровневых инструкций: ${a.instructions.length - cpi}, вложенных вызовов (CPI): ${cpi}.`,
    `- Задействованные программы: ${a.programs.map((p) => p.name).join(", ")}.`,
    `- Комиссию ${a.fee} лампортов заплатил fee payer \`${a.accounts[0].address}\`.`,
  ].join("\n");
}

async function main() {
  const connection = new Connection(MAINNET_URL, "confirmed");
  const sections: string[] = [];
  for (const c of CASES) {
    console.log(`Ищу: ${c.title}`);
    const a = await pickTransaction(connection, c.address);
    console.log(`  -> ${a.signature}`);
    sections.push(toMarkdown(a, c.title) + "\n" + conclusions(a));
    await sleep(1500);
  }
  const report = [
    "# Transaction Detective: разбор реальных транзакций Solana (mainnet)",
    "",
    `Сгенерировано скриптом \`investigate.ts\` ${new Date().toISOString()}.`,
    "",
    "Роли аккаунтов: **fee payer** платит комиссию, **signer** подписал транзакцию,",
    "**writable** будет изменён, **read-only** только читается, **program** — исполняемый аккаунт.",
    "Это нужно рантайму Sealevel: транзакции с непересекающимися writable-аккаунтами идут параллельно.",
    "",
    ...sections.flatMap((s) => [s, ""]),
  ].join("\n");
  writeFileSync("week2-transaction-detective/REPORT.md", report);
  console.log("Saved week2-transaction-detective/REPORT.md");
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
