// Неделя 1: создать кошелёк, получить тестовый SOL, отправить первую
// транзакцию и разобрать её так, как это делает Solana Explorer.
import {
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram,
  Transaction,
  TransactionInstruction,
  sendAndConfirmTransaction,
} from "@solana/web3.js";
import { devnet, ensureBalance, explorer, loadOrCreateKeypair, loadWallet, saveJson, sol } from "../common/solana.js";

const MEMO_PROGRAM_ID = new PublicKey("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr");
const DIR = "week1-first-transaction";

async function main() {
  const connection = devnet();

  // 1. Кошелёк: пара ключей ed25519. Публичный ключ = адрес, секретный хранится в файле.
  const { keypair: wallet, created } = loadOrCreateKeypair(`${DIR}/wallet.json`);
  console.log(`${created ? "Создан новый" : "Загружен"} кошелёк: ${wallet.publicKey.toBase58()}`);

  // 2. Тестовые SOL из devnet faucet
  const mainWallet = loadWallet();
  await ensureBalance(connection, wallet.publicKey, 0.2 * LAMPORTS_PER_SOL, mainWallet);
  const balanceBefore = await connection.getBalance(wallet.publicKey);
  console.log(`Баланс: ${sol(balanceBefore)}`);

  // 3. Первая транзакция: перевод 0.05 SOL + memo-инструкция с текстом
  const recipient = mainWallet.publicKey;
  const memo = "Week 1: my first Solana transaction";
  const transaction = new Transaction()
    .add(SystemProgram.transfer({ fromPubkey: wallet.publicKey, toPubkey: recipient, lamports: 0.05 * LAMPORTS_PER_SOL }))
    .add(new TransactionInstruction({ programId: MEMO_PROGRAM_ID, keys: [], data: Buffer.from(memo, "utf8") }));

  const signature = await sendAndConfirmTransaction(connection, transaction, [wallet]);
  console.log(`\nТранзакция подтверждена: ${signature}`);
  console.log(explorer("tx", signature));

  // 4. Разбор транзакции: те же данные, что показывает Explorer
  const tx = await connection.getParsedTransaction(signature, { commitment: "confirmed", maxSupportedTransactionVersion: 0 });
  if (!tx || !tx.meta) throw new Error("transaction not found");
  const keys = tx.transaction.message.accountKeys;

  const accounts = keys.map((key, i) => ({
    address: key.pubkey.toBase58(),
    signer: key.signer,
    writable: key.writable,
    preBalance: sol(tx.meta!.preBalances[i]),
    postBalance: sol(tx.meta!.postBalances[i]),
    change: sol(tx.meta!.postBalances[i] - tx.meta!.preBalances[i]),
  }));
  const instructions = tx.transaction.message.instructions.map((ix) => ({
    programId: ix.programId.toBase58(),
    program: "program" in ix ? ix.program : "unknown",
    parsed: "parsed" in ix ? ix.parsed : null,
  }));

  console.log(`\nSlot: ${tx.slot}`);
  console.log(`Время блока: ${new Date((tx.blockTime ?? 0) * 1000).toISOString()}`);
  console.log(`Комиссия: ${sol(tx.meta.fee)}`);
  console.log(`Статус: ${tx.meta.err ? "Failed" : "Success"}`);
  console.log("\nАккаунты:");
  console.table(accounts);
  console.log("Инструкции:");
  instructions.forEach((ix, i) => console.log(`  #${i + 1} ${ix.program} (${ix.programId}):`, JSON.stringify(ix.parsed)));
  console.log("\nЛоги программ:");
  tx.meta.logMessages?.forEach((line) => console.log(`  ${line}`));

  saveJson(`${DIR}/result.json`, {
    wallet: wallet.publicKey.toBase58(),
    walletExplorer: explorer("address", wallet.publicKey.toBase58()),
    signature,
    explorer: explorer("tx", signature),
    slot: tx.slot,
    blockTime: new Date((tx.blockTime ?? 0) * 1000).toISOString(),
    fee: sol(tx.meta.fee),
    status: tx.meta.err ? "Failed" : "Success",
    accounts,
    instructions,
    logs: tx.meta.logMessages,
  });
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
