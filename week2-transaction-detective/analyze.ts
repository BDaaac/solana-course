// Transaction Detective: разбирает транзакцию на accounts, instructions и programs.
import {
  Connection,
  ParsedInstruction,
  ParsedTransactionWithMeta,
  PartiallyDecodedInstruction,
} from "@solana/web3.js";

/** Известные программы, чтобы показывать имя, а не только адрес. */
export const KNOWN_PROGRAMS: Record<string, string> = {
  "11111111111111111111111111111111": "System Program",
  TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA: "SPL Token",
  TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb: "Token-2022 (Token Extensions)",
  ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL: "Associated Token Account",
  MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr: "Memo",
  Memo1UhkJRfHyvLMcVucJwxXeuD728EqVDDwQDxFMNo: "Memo v1",
  ComputeBudget111111111111111111111111111111: "Compute Budget",
  Vote111111111111111111111111111111111111111: "Vote Program",
  Stake11111111111111111111111111111111111111: "Stake Program",
  BPFLoaderUpgradeab1e11111111111111111111111: "BPF Upgradeable Loader",
  AddressLookupTab1e1111111111111111111111111: "Address Lookup Table",
  metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s: "Metaplex Token Metadata",
  JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4: "Jupiter Aggregator v6",
  whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc: "Orca Whirlpools",
  "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8": "Raydium AMM v4",
  CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK: "Raydium CLMM",
  pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA: "Pump.fun AMM",
  "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P": "Pump.fun",
  LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo: "Meteora DLMM",
};

export function programName(id: string): string {
  return KNOWN_PROGRAMS[id] ?? "Unknown program";
}

export interface AccountRow {
  index: number;
  address: string;
  roles: string[];
  solChange: number;
}

export interface InstructionRow {
  index: string;
  program: string;
  programId: string;
  type: string;
  accounts: string[];
  details: string;
}

export interface Analysis {
  signature: string;
  cluster: string;
  slot: number;
  blockTime: string;
  status: string;
  fee: number;
  computeUnits: number | null;
  version: string;
  signers: string[];
  accounts: AccountRow[];
  instructions: InstructionRow[];
  programs: { programId: string; name: string; calls: number }[];
  tokenChanges: { owner: string; mint: string; change: number }[];
  logs: string[];
}

function describeInstruction(ix: ParsedInstruction | PartiallyDecodedInstruction, index: string): InstructionRow {
  const programId = ix.programId.toBase58();
  if ("parsed" in ix) {
    const parsed = ix.parsed as { type?: string; info?: Record<string, unknown> } | string;
    const type = typeof parsed === "string" ? "memo" : parsed.type ?? "unknown";
    const info = typeof parsed === "string" ? { text: parsed } : parsed.info ?? {};
    const accounts = Object.entries(info)
      .filter(([, v]) => typeof v === "string" && (v as string).length >= 32 && (v as string).length <= 44)
      .map(([k, v]) => `${k}=${v}`);
    return { index, program: programName(programId), programId, type, accounts, details: JSON.stringify(info) };
  }
  // Программа неизвестна RPC-парсеру: показываем сырые данные
  return {
    index,
    program: programName(programId),
    programId,
    type: "raw",
    accounts: ix.accounts.map((a) => a.toBase58()),
    details: `data (base58, ${ix.data.length} chars): ${ix.data.slice(0, 60)}${ix.data.length > 60 ? "..." : ""}`,
  };
}

export function analyze(tx: ParsedTransactionWithMeta, signature: string, cluster: string): Analysis {
  const meta = tx.meta;
  if (!meta) throw new Error("transaction has no meta");
  const keys = tx.transaction.message.accountKeys;
  const programIds = new Set(
    tx.transaction.message.instructions.map((ix) => ix.programId.toBase58()),
  );

  const accounts: AccountRow[] = keys.map((key, i) => {
    const address = key.pubkey.toBase58();
    const roles: string[] = [];
    if (i === 0) roles.push("fee payer");
    if (key.signer) roles.push("signer");
    roles.push(key.writable ? "writable" : "read-only");
    if (programIds.has(address)) roles.push("program");
    if (key.source === "lookupTable") roles.push("from lookup table");
    return { index: i, address, roles, solChange: (meta.postBalances[i] - meta.preBalances[i]) / 1e9 };
  });

  const instructions: InstructionRow[] = [];
  const calls = new Map<string, number>();
  const count = (id: string) => calls.set(id, (calls.get(id) ?? 0) + 1);
  tx.transaction.message.instructions.forEach((ix, i) => {
    instructions.push(describeInstruction(ix, `${i + 1}`));
    count(ix.programId.toBase58());
    // Inner instructions = CPI: программа вызывает другую программу
    const inner = meta.innerInstructions?.find((group) => group.index === i);
    inner?.instructions.forEach((innerIx, j) => {
      instructions.push(describeInstruction(innerIx, `${i + 1}.${j + 1} (CPI)`));
      count(innerIx.programId.toBase58());
    });
  });

  const tokenChanges = (meta.postTokenBalances ?? []).map((post) => {
    const pre = meta.preTokenBalances?.find((p) => p.accountIndex === post.accountIndex);
    return {
      owner: post.owner ?? keys[post.accountIndex].pubkey.toBase58(),
      mint: post.mint,
      change: (post.uiTokenAmount.uiAmount ?? 0) - (pre?.uiTokenAmount.uiAmount ?? 0),
    };
  }).filter((c) => c.change !== 0);

  return {
    signature,
    cluster,
    slot: tx.slot,
    blockTime: tx.blockTime ? new Date(tx.blockTime * 1000).toISOString() : "unknown",
    status: meta.err ? `Failed: ${JSON.stringify(meta.err)}` : "Success",
    fee: meta.fee,
    computeUnits: meta.computeUnitsConsumed ?? null,
    version: String(tx.version ?? "legacy"),
    signers: accounts.filter((a) => a.roles.includes("signer")).map((a) => a.address),
    accounts,
    instructions,
    programs: [...calls].map(([programId, n]) => ({ programId, name: programName(programId), calls: n })),
    tokenChanges,
    logs: meta.logMessages ?? [],
  };
}

export async function fetchAndAnalyze(connection: Connection, signature: string, cluster: string): Promise<Analysis> {
  const tx = await connection.getParsedTransaction(signature, { maxSupportedTransactionVersion: 0, commitment: "confirmed" });
  if (!tx) throw new Error(`transaction ${signature} not found on ${cluster}`);
  return analyze(tx, signature, cluster);
}

export function toMarkdown(a: Analysis, title: string): string {
  const explorer = `https://explorer.solana.com/tx/${a.signature}${a.cluster === "mainnet" ? "" : `?cluster=${a.cluster}`}`;
  const lines = [
    `### ${title}`,
    "",
    `- Signature: [\`${a.signature.slice(0, 20)}...\`](${explorer})`,
    `- Cluster: ${a.cluster}, slot ${a.slot}, ${a.blockTime}`,
    `- Status: ${a.status}; fee ${a.fee} lamports; compute units ${a.computeUnits ?? "n/a"}; version ${a.version}`,
    `- Signers: ${a.signers.map((s) => `\`${s}\``).join(", ")}`,
    "",
    "**Accounts**",
    "",
    "| # | Address | Roles | SOL change |",
    "|---|---|---|---|",
    ...a.accounts.map((acc) => `| ${acc.index} | \`${acc.address}\` | ${acc.roles.join(", ")} | ${acc.solChange} |`),
    "",
    "**Instructions**",
    "",
    "| # | Program | Type | Details |",
    "|---|---|---|---|",
    ...a.instructions.map(
      (ix) => `| ${ix.index} | ${ix.program} | ${ix.type} | ${ix.details.replace(/\|/g, "\\|").slice(0, 160)} |`,
    ),
    "",
    "**Programs**",
    "",
    ...a.programs.map((p) => `- ${p.name} \`${p.programId}\`: ${p.calls} call(s)`),
  ];
  if (a.tokenChanges.length) {
    lines.push("", "**Token balance changes**", "");
    a.tokenChanges.forEach((c) => lines.push(`- owner \`${c.owner}\`, mint \`${c.mint}\`: ${c.change > 0 ? "+" : ""}${c.change}`));
  }
  return lines.join("\n");
}
