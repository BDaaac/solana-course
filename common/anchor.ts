// Подключение Anchor-программы по IDL из target/idl.
import { readFileSync } from "node:fs";
import { AnchorProvider, Idl, Program, Wallet } from "@anchor-lang/core";
import { devnet, loadWallet } from "./solana.js";

export function anchorProgram<T extends Idl>(idlPath: string): { program: Program<T>; provider: AnchorProvider } {
  const idl = JSON.parse(readFileSync(idlPath, "utf8")) as T;
  const provider = new AnchorProvider(devnet(), new Wallet(loadWallet()), { commitment: "confirmed" });
  return { program: new Program<T>(idl, provider), provider };
}
