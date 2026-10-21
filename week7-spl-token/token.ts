// Неделя 7: собственный токен на Token-2022 (Token Extensions).
// Выпуск mint'а с расширениями MetadataPointer + TokenMetadata (метаданные хранятся
// прямо в аккаунте mint, без Metaplex), mint, transfer и чтение через client.
import {
  Keypair,
  LAMPORTS_PER_SOL,
  SystemProgram,
  Transaction,
  sendAndConfirmTransaction,
} from "@solana/web3.js";
import {
  ExtensionType,
  LENGTH_SIZE,
  TOKEN_2022_PROGRAM_ID,
  TYPE_SIZE,
  createInitializeMetadataPointerInstruction,
  createInitializeMintInstruction,
  getAccount,
  getMint,
  getMintLen,
  getOrCreateAssociatedTokenAccount,
  getTokenMetadata,
  mintTo,
  transferChecked,
} from "@solana/spl-token";
import {
  TokenMetadata,
  createInitializeInstruction,
  createUpdateFieldInstruction,
  pack,
} from "@solana/spl-token-metadata";
import { devnet, ensureBalance, explorer, loadWallet, saveJson } from "../common/solana.js";

const DIR = "week7-spl-token";
const DECIMALS = 6;
const SUPPLY = 1_000_000n; // целых токенов
const TRANSFER_AMOUNT = 1_500n; // целых токенов

async function main() {
  const connection = devnet();
  const payer = loadWallet();
  await ensureBalance(connection, payer.publicKey, 0.1 * LAMPORTS_PER_SOL);
  const unit = 10n ** BigInt(DECIMALS);
  const log: Record<string, string> = {};

  // 1. Mint-аккаунт с метаданными внутри
  const mint = Keypair.generate();
  const metadata: TokenMetadata = {
    mint: mint.publicKey,
    updateAuthority: payer.publicKey,
    name: "Solana Course Token",
    symbol: "SCT",
    uri: "https://raw.githubusercontent.com/solana-developers/opos-asset/main/assets/DeveloperPortal/metadata.json",
    additionalMetadata: [["course", "Solana week 7"]],
  };
  const mintLen = getMintLen([ExtensionType.MetadataPointer]);
  const metadataLen = TYPE_SIZE + LENGTH_SIZE + pack(metadata).length;
  // Место под метаданные выделяется при их инициализации, но rent платим сразу за весь размер
  const lamports = await connection.getMinimumBalanceForRentExemption(mintLen + metadataLen);

  const createMintTx = new Transaction().add(
    SystemProgram.createAccount({
      fromPubkey: payer.publicKey,
      newAccountPubkey: mint.publicKey,
      space: mintLen,
      lamports,
      programId: TOKEN_2022_PROGRAM_ID,
    }),
    // Расширения инициализируются ДО самого mint'а
    createInitializeMetadataPointerInstruction(mint.publicKey, payer.publicKey, mint.publicKey, TOKEN_2022_PROGRAM_ID),
    createInitializeMintInstruction(mint.publicKey, DECIMALS, payer.publicKey, null, TOKEN_2022_PROGRAM_ID),
    createInitializeInstruction({
      programId: TOKEN_2022_PROGRAM_ID,
      metadata: mint.publicKey,
      updateAuthority: payer.publicKey,
      mint: mint.publicKey,
      mintAuthority: payer.publicKey,
      name: metadata.name,
      symbol: metadata.symbol,
      uri: metadata.uri,
    }),
    createUpdateFieldInstruction({
      programId: TOKEN_2022_PROGRAM_ID,
      metadata: mint.publicKey,
      updateAuthority: payer.publicKey,
      field: metadata.additionalMetadata[0][0],
      value: metadata.additionalMetadata[0][1],
    }),
  );
  log.createMint = await sendAndConfirmTransaction(connection, createMintTx, [payer, mint]);
  console.log(`Mint создан: ${mint.publicKey.toBase58()}\n  ${explorer("tx", log.createMint)}`);

  // 2. Выпуск: ATA владельца + mintTo
  const ownerAta = await getOrCreateAssociatedTokenAccount(
    connection, payer, mint.publicKey, payer.publicKey, false, "confirmed", undefined, TOKEN_2022_PROGRAM_ID,
  );
  log.mintTo = await mintTo(
    connection, payer, mint.publicKey, ownerAta.address, payer, SUPPLY * unit, [], undefined, TOKEN_2022_PROGRAM_ID,
  );
  console.log(`Выпущено ${SUPPLY} SCT на ${ownerAta.address.toBase58()}\n  ${explorer("tx", log.mintTo)}`);

  // 3. Transfer новому владельцу (для него создаётся свой ATA)
  const recipient = Keypair.generate().publicKey;
  const recipientAta = await getOrCreateAssociatedTokenAccount(
    connection, payer, mint.publicKey, recipient, false, "confirmed", undefined, TOKEN_2022_PROGRAM_ID,
  );
  // transferChecked проверяет decimals: защищает от ошибки в порядке величины
  log.transfer = await transferChecked(
    connection, payer, ownerAta.address, mint.publicKey, recipientAta.address, payer,
    TRANSFER_AMOUNT * unit, DECIMALS, [], undefined, TOKEN_2022_PROGRAM_ID,
  );
  console.log(`Переведено ${TRANSFER_AMOUNT} SCT -> ${recipient.toBase58()}\n  ${explorer("tx", log.transfer)}`);

  // 4. Чтение состояния через client
  const mintInfo = await getMint(connection, mint.publicKey, "confirmed", TOKEN_2022_PROGRAM_ID);
  const onchainMetadata = await getTokenMetadata(connection, mint.publicKey, "confirmed", TOKEN_2022_PROGRAM_ID);
  const ownerBalance = (await getAccount(connection, ownerAta.address, "confirmed", TOKEN_2022_PROGRAM_ID)).amount;
  const recipientBalance = (await getAccount(connection, recipientAta.address, "confirmed", TOKEN_2022_PROGRAM_ID)).amount;

  const summary = {
    mint: mint.publicKey.toBase58(),
    mintExplorer: explorer("address", mint.publicKey.toBase58()),
    program: "Token-2022 (TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb)",
    decimals: mintInfo.decimals,
    supply: (mintInfo.supply / unit).toString(),
    mintAuthority: mintInfo.mintAuthority?.toBase58(),
    metadata: onchainMetadata && {
      name: onchainMetadata.name,
      symbol: onchainMetadata.symbol,
      uri: onchainMetadata.uri,
      additionalMetadata: onchainMetadata.additionalMetadata,
    },
    balances: {
      [`owner ${payer.publicKey.toBase58()}`]: (ownerBalance / unit).toString(),
      [`recipient ${recipient.toBase58()}`]: (recipientBalance / unit).toString(),
    },
    transactions: Object.fromEntries(Object.entries(log).map(([k, sig]) => [k, explorer("tx", sig)])),
  };
  console.log("\nСостояние токена:");
  console.log(JSON.stringify(summary, null, 2));
  saveJson(`${DIR}/result.json`, summary);
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
