// Общие помощники для всех клиентов курса.
import { readFileSync, writeFileSync, existsSync, mkdirSync } from "node:fs";
import { homedir } from "node:os";
import { dirname, join } from "node:path";
import {
  Connection,
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram,
  Transaction,
  sendAndConfirmTransaction,
} from "@solana/web3.js";

export const DEVNET_URL = process.env.RPC_URL ?? "https://api.devnet.solana.com";
export const MAINNET_URL = process.env.MAINNET_RPC_URL ?? "https://api.mainnet-beta.solana.com";

export function devnet(): Connection {
  return new Connection(DEVNET_URL, "confirmed");
}

/** Кошелёк Solana CLI: ~/.config/solana/id.json (массив из 64 байт секретного ключа). */
export function loadWallet(path = join(homedir(), ".config", "solana", "id.json")): Keypair {
  const secret = JSON.parse(readFileSync(path, "utf8")) as number[];
  return Keypair.fromSecretKey(Uint8Array.from(secret));
}

/** Загружает кошелёк из файла или создаёт новый и сохраняет его. */
export function loadOrCreateKeypair(path: string): { keypair: Keypair; created: boolean } {
  if (existsSync(path)) return { keypair: loadWallet(path), created: false };
  const keypair = Keypair.generate();
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, JSON.stringify(Array.from(keypair.secretKey)));
  return { keypair, created: true };
}

export function explorer(kind: "tx" | "address", value: string, cluster = "devnet"): string {
  return `https://explorer.solana.com/${kind}/${value}?cluster=${cluster}`;
}

export function sol(lamports: number | bigint): string {
  return `${Number(lamports) / LAMPORTS_PER_SOL} SOL`;
}

/**
 * Пополняет адрес до `minLamports`: сначала пробует devnet-airdrop,
 * если faucet отказал (лимиты), переводит SOL с основного кошелька `funder`.
 */
export async function ensureBalance(
  connection: Connection,
  target: PublicKey,
  minLamports: number,
  funder?: Keypair,
): Promise<void> {
  const balance = await connection.getBalance(target);
  if (balance >= minLamports) return;
  const missing = minLamports - balance;
  try {
    const signature = await connection.requestAirdrop(target, missing);
    await connection.confirmTransaction(signature, "confirmed");
    console.log(`Airdrop ${sol(missing)} -> ${target.toBase58()}: ${signature}`);
    return;
  } catch (err) {
    if (!funder) throw new Error(`Airdrop failed and no funder wallet given: ${(err as Error).message}`);
    console.log(`Airdrop недоступен (${(err as Error).message.split("\n")[0]}), перевожу с основного кошелька`);
  }
  const tx = new Transaction().add(
    SystemProgram.transfer({ fromPubkey: funder.publicKey, toPubkey: target, lamports: missing }),
  );
  const signature = await sendAndConfirmTransaction(connection, tx, [funder]);
  console.log(`Funded ${sol(missing)} -> ${target.toBase58()}: ${signature}`);
}

export function saveJson(path: string, data: unknown): void {
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(
    path,
    JSON.stringify(data, (_, v) => (typeof v === "bigint" ? v.toString() : v), 2) + "\n",
  );
  console.log(`Saved ${path}`);
}
