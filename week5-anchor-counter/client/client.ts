// Клиент недели 5: взаимодействие с Counter Program на devnet через Anchor TS.
import BN from "bn.js";
import { PublicKey } from "@solana/web3.js";
import { anchorProgram } from "../../common/anchor.js";
import { explorer, saveJson } from "../../common/solana.js";
import type { Week5AnchorCounter } from "../target/types/week5_anchor_counter.js";

const DIR = "week5-anchor-counter";

async function main() {
  const { program, provider } = anchorProgram<Week5AnchorCounter>(`${DIR}/target/idl/week5_anchor_counter.json`);
  const authority = provider.wallet.publicKey;
  // Тот же PDA, что вычисляет программа: seeds = ["counter", authority]
  const [counter] = PublicKey.findProgramAddressSync([Buffer.from("counter"), authority.toBuffer()], program.programId);
  console.log(`Program: ${program.programId.toBase58()}\nCounter PDA: ${counter.toBase58()}`);

  const txs: Record<string, string> = {};
  const state = async (label: string) => {
    const c = await program.account.counter.fetch(counter);
    console.log(`${label.padEnd(16)} count=${c.count.toString()} updates=${c.totalUpdates.toString()}`);
    return c;
  };

  if (!(await provider.connection.getAccountInfo(counter))) {
    txs.initialize = await program.methods.initialize().accounts({ authority }).rpc();
  } else {
    txs.reset = await program.methods.reset().accounts({ authority }).rpc();
  }
  await state("after init");

  txs.increment5 = await program.methods.increment(new BN(5)).accounts({ authority }).rpc();
  await state("increment(5)");
  txs.increment3 = await program.methods.increment(new BN(3)).accounts({ authority }).rpc();
  await state("increment(3)");
  txs.decrement2 = await program.methods.decrement(new BN(2)).accounts({ authority }).rpc();
  const final = await state("decrement(2)");

  // Ошибка программы: уйти ниже нуля нельзя
  let underflow = "";
  try {
    await program.methods.decrement(new BN(1000)).accounts({ authority }).rpc();
  } catch (err) {
    underflow = (err as Error).message.split("\n")[0];
    console.log(`decrement(1000) отклонён: ${underflow}`);
  }

  Object.entries(txs).forEach(([k, sig]) => console.log(`${k.padEnd(12)} ${explorer("tx", sig)}`));
  saveJson(`${DIR}/result.json`, {
    programId: program.programId.toBase58(),
    programExplorer: explorer("address", program.programId.toBase58()),
    counterPda: counter.toBase58(),
    finalState: {
      authority: final.authority.toBase58(),
      count: final.count.toString(),
      totalUpdates: final.totalUpdates.toString(),
    },
    transactions: Object.fromEntries(Object.entries(txs).map(([k, s]) => [k, explorer("tx", s)])),
    underflowError: underflow,
  });
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
