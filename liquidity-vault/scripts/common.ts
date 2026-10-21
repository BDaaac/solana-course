// Общие помощники клиентов liquidity-vault (devnet).
import { readFileSync, writeFileSync, existsSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import anchor from "@coral-xyz/anchor";
import type { Program as ProgramT } from "@coral-xyz/anchor";
const { AnchorProvider, Program, Wallet } = anchor;
import { Keypair, PublicKey } from "@solana/web3.js";
import { devnet, loadWallet } from "../../common/solana.js";

const here = dirname(fileURLToPath(import.meta.url));
export const ROOT = join(here, "..");
export const STATE_PATH = join(ROOT, "state.json");

export type State = Record<string, any>;

export function loadState(): State {
  return existsSync(STATE_PATH) ? JSON.parse(readFileSync(STATE_PATH, "utf8")) : {};
}

export function saveState(patch: State): State {
  const next = { ...loadState(), ...patch };
  writeFileSync(STATE_PATH, JSON.stringify(next, null, 2) + "\n");
  return next;
}

export function appendLog(key: string, entry: unknown): void {
  const s = loadState();
  s[key] = [...(s[key] ?? []), entry];
  writeFileSync(STATE_PATH, JSON.stringify(s, null, 2) + "\n");
}

export function setup() {
  const connection = devnet();
  const admin: Keypair = loadWallet();
  const provider = new AnchorProvider(connection, new Wallet(admin), { commitment: "confirmed" });
  anchor.setProvider(provider);
  const idl = JSON.parse(readFileSync(join(ROOT, "idl", "simple_vault.json"), "utf8"));
  const program = new Program(idl, provider) as ProgramT<any>;
  return { connection, admin, provider, program };
}

export function vaultPdas(programId: PublicKey, tokenMint: PublicKey) {
  const f = (seed: string) => PublicKey.findProgramAddressSync([Buffer.from(seed), tokenMint.toBuffer()], programId)[0];
  const vaultState = f("vault");
  return {
    vaultState,
    vaultTokens: f("vault_tokens"),
    shareMint: f("share_mint"),
    userCard: (user: PublicKey) =>
      PublicKey.findProgramAddressSync(
        [Buffer.from("user_card"), vaultState.toBuffer(), user.toBuffer()],
        programId,
      )[0],
  };
}

export const link = (sig: string) => `https://explorer.solana.com/tx/${sig}?cluster=devnet`;
export const addr = (a: string | PublicKey) => `https://explorer.solana.com/address/${a.toString()}?cluster=devnet`;

/** Ожидаем, что вызов упадёт с ошибкой, содержащей `needle`. */
export async function expectFail(label: string, fn: () => Promise<unknown>, needle: string): Promise<string> {
  try {
    await fn();
  } catch (e: any) {
    const text = `${e?.message ?? e} ${(e?.logs ?? []).join(" ")}`;
    if (text.includes(needle)) {
      console.log(`  ✓ ${label}: отклонено (${needle})`);
      return needle;
    }
    throw new Error(`${label}: ошибка не та: ${text.slice(0, 400)}`);
  }
  throw new Error(`${label}: должно было упасть`);
}
