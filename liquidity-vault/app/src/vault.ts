// Клиент программы vault для браузера.
import { AnchorProvider, Program, type Idl } from "@coral-xyz/anchor";
import type { AnchorWallet } from "@solana/wallet-adapter-react";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  createAssociatedTokenAccountIdempotentInstruction,
  getAssociatedTokenAddressSync,
} from "@solana/spl-token";
import { Connection, PublicKey, Transaction } from "@solana/web3.js";
import BN from "bn.js";
import idl from "./idl.json";
import { TOKEN_MINT } from "./config";

export const tokenMint = new PublicKey(TOKEN_MINT);
export const programId = new PublicKey((idl as { address: string }).address);

const pda = (seed: string, ...extra: PublicKey[]) =>
  PublicKey.findProgramAddressSync([Buffer.from(seed), ...extra.map((k) => k.toBuffer())], programId)[0];

export const vaultState = pda("vault", tokenMint);
export const vaultTokens = pda("vault_tokens", tokenMint);
export const shareMint = pda("share_mint", tokenMint);
export const userCardOf = (user: PublicKey) => pda("user_card", vaultState, user);

export function getProgram(connection: Connection, wallet: AnchorWallet) {
  const provider = new AnchorProvider(connection, wallet, { commitment: "confirmed" });
  return new Program(idl as Idl, provider);
}

export interface VaultView {
  admin: string;
  paused: boolean;
  totalTokens: bigint;
  totalShares: bigint;
  userTokens: bigint;
  userShares: bigint;
  hasCard: boolean;
  deposited: bigint;
}

export async function loadView(connection: Connection, wallet: AnchorWallet): Promise<VaultView> {
  const program = getProgram(connection, wallet);
  const vs = await (program.account as any).vaultState.fetch(vaultState);
  const tokAta = getAssociatedTokenAddressSync(tokenMint, wallet.publicKey);
  const shAta = getAssociatedTokenAddressSync(shareMint, wallet.publicKey);
  const amount = async (a: PublicKey) => {
    try {
      return BigInt((await connection.getTokenAccountBalance(a)).value.amount);
    } catch {
      return 0n;
    }
  };
  const card = await (program.account as any).userCard.fetchNullable(userCardOf(wallet.publicKey));
  return {
    admin: vs.admin.toBase58(),
    paused: vs.paused,
    totalTokens: BigInt(vs.totalTokens.toString()),
    totalShares: BigInt(vs.totalShares.toString()),
    userTokens: await amount(tokAta),
    userShares: await amount(shAta),
    hasCard: !!card,
    deposited: card ? BigInt(card.deposited.toString()) : 0n,
  };
}

const userAccounts = (user: PublicKey) => ({
  vaultState,
  userCard: userCardOf(user),
  vaultTokenAccount: vaultTokens,
  shareMint,
  userTokenAccount: getAssociatedTokenAddressSync(tokenMint, user),
  userShareAccount: getAssociatedTokenAddressSync(shareMint, user),
  user,
  tokenProgram: TOKEN_PROGRAM_ID,
});

/** Депозит: при необходимости создаёт ATA и карточку — всё одной транзакцией. */
export async function deposit(connection: Connection, wallet: AnchorWallet, amount: bigint, hasCard: boolean) {
  const program = getProgram(connection, wallet);
  const u = wallet.publicKey;
  const tx = new Transaction().add(
    createAssociatedTokenAccountIdempotentInstruction(u, getAssociatedTokenAddressSync(shareMint, u), u, shareMint, TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID),
  );
  if (!hasCard) {
    tx.add(await program.methods.createUserCard().accountsPartial({ vaultState, userCard: userCardOf(u), user: u }).instruction());
  }
  tx.add(await program.methods.deposit(new BN(amount.toString())).accountsPartial(userAccounts(u)).instruction());
  return (program.provider as AnchorProvider).sendAndConfirm(tx);
}

export async function withdraw(connection: Connection, wallet: AnchorWallet, shares: bigint) {
  const program = getProgram(connection, wallet);
  return program.methods.withdraw(new BN(shares.toString())).accountsPartial(userAccounts(wallet.publicKey)).rpc();
}

export async function setPaused(connection: Connection, wallet: AnchorWallet, paused: boolean) {
  const program = getProgram(connection, wallet);
  return program.methods.setPaused(paused).accountsPartial({ vaultState, admin: wallet.publicKey }).rpc();
}

export async function syncBalance(connection: Connection, wallet: AnchorWallet) {
  const program = getProgram(connection, wallet);
  return program.methods
    .syncBalance()
    .accountsPartial({ vaultState, vaultTokenAccount: vaultTokens, admin: wallet.publicKey })
    .rpc();
}
