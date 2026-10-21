// CLI: npm run week2 -- <signature> [devnet|mainnet]
import { Connection } from "@solana/web3.js";
import { DEVNET_URL, MAINNET_URL } from "../common/solana.js";
import { fetchAndAnalyze } from "./analyze.js";

async function main() {
  const [signature, cluster = "devnet"] = process.argv.slice(2);
  if (!signature) {
    console.error("Usage: npm run week2 -- <signature> [devnet|mainnet]");
    process.exit(1);
  }
  const connection = new Connection(cluster === "mainnet" ? MAINNET_URL : DEVNET_URL, "confirmed");
  const a = await fetchAndAnalyze(connection, signature, cluster);

  console.log(`Signature: ${a.signature}`);
  console.log(`Slot ${a.slot} | ${a.blockTime} | ${a.status} | fee ${a.fee} lamports | CU ${a.computeUnits}`);
  console.log("\nACCOUNTS");
  console.table(a.accounts.map((acc) => ({ address: acc.address, roles: acc.roles.join(", "), sol: acc.solChange })));
  console.log("INSTRUCTIONS");
  console.table(a.instructions.map((ix) => ({ "#": ix.index, program: ix.program, type: ix.type })));
  console.log("PROGRAMS");
  a.programs.forEach((p) => console.log(`  ${p.name.padEnd(30)} ${p.programId} x${p.calls}`));
  if (a.tokenChanges.length) {
    console.log("\nTOKEN CHANGES");
    console.table(a.tokenChanges);
  }
  console.log("\nLOGS");
  a.logs.forEach((l) => console.log(`  ${l}`));
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
