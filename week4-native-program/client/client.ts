// Клиент недели 4: создаёт аккаунт заметки, вызывает Initialize и Update,
// читает и декодирует данные аккаунта.
import { readFileSync } from "node:fs";
import {
  Keypair,
  PublicKey,
  SystemProgram,
  Transaction,
  TransactionInstruction,
  sendAndConfirmTransaction,
} from "@solana/web3.js";
import { devnet, explorer, loadWallet, saveJson } from "../../common/solana.js";

const DIR = "week4-native-program";
const NOTE_SPACE = 32 + 4 + 200 + 4;

const programKeypair = Keypair.fromSecretKey(
  Uint8Array.from(JSON.parse(readFileSync(`${DIR}/program/target/deploy/week4_note-keypair.json`, "utf8"))),
);
const PROGRAM_ID = programKeypair.publicKey;

/** Borsh-кодирование NoteInstruction: [tag u8][len u32 LE][utf8 bytes] */
function encode(tag: 0 | 1, text: string): Buffer {
  const bytes = Buffer.from(text, "utf8");
  const out = Buffer.alloc(1 + 4 + bytes.length);
  out.writeUInt8(tag, 0);
  out.writeUInt32LE(bytes.length, 1);
  bytes.copy(out, 5);
  return out;
}

/** Декодирование Note: author [32] + string + updates u32 */
function decodeNote(data: Buffer) {
  const author = new PublicKey(data.subarray(0, 32));
  const len = data.readUInt32LE(32);
  const text = data.subarray(36, 36 + len).toString("utf8");
  const updates = data.readUInt32LE(36 + len);
  return { author: author.toBase58(), text, updates };
}

function noteInstruction(note: PublicKey, author: PublicKey, data: Buffer) {
  return new TransactionInstruction({
    programId: PROGRAM_ID,
    keys: [
      { pubkey: note, isSigner: false, isWritable: true },
      { pubkey: author, isSigner: true, isWritable: false },
    ],
    data,
  });
}

async function main() {
  const connection = devnet();
  const payer = loadWallet();
  const note = Keypair.generate();
  console.log(`Program: ${PROGRAM_ID.toBase58()}`);

  // 1. Создаём аккаунт (владелец — наша программа) и сразу инициализируем заметку
  const rent = await connection.getMinimumBalanceForRentExemption(NOTE_SPACE);
  const initTx = new Transaction().add(
    SystemProgram.createAccount({
      fromPubkey: payer.publicKey,
      newAccountPubkey: note.publicKey,
      lamports: rent,
      space: NOTE_SPACE,
      programId: PROGRAM_ID,
    }),
    noteInstruction(note.publicKey, payer.publicKey, encode(0, "Hello from week 4!")),
  );
  const initSig = await sendAndConfirmTransaction(connection, initTx, [payer, note]);
  console.log(`Initialize: ${explorer("tx", initSig)}`);

  // 2. Обновляем текст
  const updateSig = await sendAndConfirmTransaction(
    connection,
    new Transaction().add(noteInstruction(note.publicKey, payer.publicKey, encode(1, "Updated: my first Solana program works"))),
    [payer],
  );
  console.log(`Update:     ${explorer("tx", updateSig)}`);

  // 3. Чужой автор не может изменить заметку: ожидаем ошибку программы
  const stranger = Keypair.generate();
  let strangerError = "";
  try {
    await sendAndConfirmTransaction(
      connection,
      new Transaction({ feePayer: payer.publicKey }).add(noteInstruction(note.publicKey, stranger.publicKey, encode(1, "hack"))),
      [payer, stranger],
    );
  } catch (err) {
    strangerError = (err as Error).message.split("\n")[0];
    console.log(`Update от чужого ключа отклонён: ${strangerError}`);
  }

  // 4. Читаем состояние
  const info = await connection.getAccountInfo(note.publicKey, "confirmed");
  if (!info) throw new Error("note account not found");
  const decoded = decodeNote(info.data);
  const tx = await connection.getTransaction(updateSig, { commitment: "confirmed", maxSupportedTransactionVersion: 0 });
  console.log("Заметка on-chain:", decoded);

  saveJson(`${DIR}/result.json`, {
    programId: PROGRAM_ID.toBase58(),
    programExplorer: explorer("address", PROGRAM_ID.toBase58()),
    noteAccount: note.publicKey.toBase58(),
    owner: info.owner.toBase58(),
    note: decoded,
    transactions: { initialize: explorer("tx", initSig), update: explorer("tx", updateSig) },
    updateLogs: tx?.meta?.logMessages,
    unauthorizedUpdate: strangerError || "unexpectedly succeeded",
  });
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
