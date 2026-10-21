// Неделя 12: vault обменивает токены через наш CLMM-пул Raydium (CPI, подпись PDA, защита от slippage).
import BN from "bn.js";
import { ComputeBudgetProgram, Keypair, PublicKey, SystemProgram, Transaction, sendAndConfirmTransaction } from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  TOKEN_2022_PROGRAM_ID,
  createAssociatedTokenAccountIdempotent,
  getAssociatedTokenAddressSync,
  getAccount,
} from "@solana/spl-token";
import { Raydium, PoolUtils, DEVNET_PROGRAM_ID, DEV_API_URLS, getPdaExBitmapAccount } from "@raydium-io/raydium-sdk-v2";
import { expectFail, link, loadState, saveState, setup, vaultPdas } from "./common.js";

const MEMO = new PublicKey("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr");
const CLMM = DEVNET_PROGRAM_ID.CLMM_PROGRAM_ID;

export async function loadPool(raydium: Raydium, poolId: string) {
  const data = await raydium.clmm.getPoolInfoFromRpc(poolId);
  return data;
}

async function main() {
  const { connection, admin, program } = setup();
  const state = loadState();
  const tokenA = new PublicKey(state.tokenMint);
  const tokenB = new PublicKey(state.quoteMint);
  const pda = vaultPdas(program.programId, tokenA);

  const raydium = await Raydium.load({
    owner: admin, connection, cluster: "devnet",
    disableFeatureCheck: true, disableLoadToken: true, blockhashCommitment: "finalized", urlConfigs: DEV_API_URLS,
  });

  // Второе хранилище vault — токен B, владелец vault PDA.
  const vaultQuote = await createAssociatedTokenAccountIdempotent(
    connection, admin, tokenB, pda.vaultState, undefined, TOKEN_PROGRAM_ID, undefined, true,
  );
  console.log("Vault quote account (токен B):", vaultQuote.toBase58());

  const { poolInfo, poolKeys, computePoolInfo, tickData } = await loadPool(raydium, state.poolId);
  const priceBefore = String(poolInfo.price);
  const epochInfo = await raydium.fetchEpochInfo();

  const amountIn = new BN(100_000_000); // 100 токенов A
  const quote = PoolUtils.computeAmountOutFormat({
    poolInfo: computePoolInfo,
    tickArrayCache: tickData[state.poolId],
    amountIn,
    tokenOut: poolInfo.mintB,
    slippage: 0.01, // минимум на 1% ниже ожидаемого
    epochInfo,
  });
  const expectedOut = quote.amountOut.amount.raw;
  const minOut = quote.minAmountOut.amount.raw;
  console.log(`Ожидаем ${expectedOut.toString()}, минимум (−1%) ${minOut.toString()}, price impact ${quote.priceImpact.toString()}`);

  const exBitmap = getPdaExBitmapAccount(CLMM, new PublicKey(state.poolId)).publicKey;
  const remaining = [exBitmap, ...quote.remainingAccounts].map((pubkey) => ({ pubkey, isSigner: false, isWritable: true }));

  const accounts = {
    admin: admin.publicKey,
    vaultState: pda.vaultState,
    inputTokenAccount: pda.vaultTokens,
    outputTokenAccount: vaultQuote,
    ammConfig: new PublicKey(poolKeys.config.id),
    poolState: new PublicKey(state.poolId),
    inputVault: new PublicKey(poolKeys.vault.A),
    outputVault: new PublicKey(poolKeys.vault.B),
    observationState: computePoolInfo.observationId,
    inputVaultMint: tokenA,
    outputVaultMint: tokenB,
    clmmProgram: CLMM,
    tokenProgram: TOKEN_PROGRAM_ID,
    tokenProgram2022: TOKEN_2022_PROGRAM_ID,
    memoProgram: MEMO,
  };
  const cu = ComputeBudgetProgram.setComputeUnitLimit({ units: 600_000 });
  const swap = (amount: BN, min: BN, signer?: Keypair) => {
    let b = program.methods
      .swapViaRaydium(amount, min, new BN(0))
      .accountsPartial(signer ? { ...accounts, admin: signer.publicKey } : accounts)
      .remainingAccounts(remaining)
      .preInstructions([cu]);
    if (signer) b = b.signers([signer]);
    return b.rpc();
  };

  // 1. Отклонённый обмен: минимум в 2 раза выше ожидаемого — Raydium откатывает транзакцию.
  const rejected = await expectFail("swap с завышенным минимумом", () => swap(amountIn, expectedOut.muln(2)), "TooLittleOutputReceived")
    .catch(async (e) => { console.log("  (проверяем текст ошибки)", e.message); throw e; });

  // Для README нужна подпись неуспешной транзакции: отправим её без preflight.
  let rejectedSig = "";
  try {
    const ix = await program.methods.swapViaRaydium(amountIn, expectedOut.muln(2), new BN(0))
      .accountsPartial(accounts).remainingAccounts(remaining).instruction();
    const tx = new Transaction().add(cu, ix);
    const sig = await connection.sendTransaction(tx, [admin], { skipPreflight: true });
    rejectedSig = sig;
    await connection.confirmTransaction(sig, "confirmed");
    console.log("Отклонённый обмен (on-chain, failed):", link(sig));
  } catch (e) { console.log("rejected send error", (e as Error).message); }

  // 2. Не админ.
  const stranger = Keypair.generate();
  await sendAndConfirmTransaction(connection, new Transaction().add(
    SystemProgram.transfer({ fromPubkey: admin.publicKey, toPubkey: stranger.publicKey, lamports: 10_000_000 })), [admin]);
  await expectFail("swap не админом", () => swap(amountIn, minOut, stranger), "Unauthorized");

  // 3. Успешный обмен.
  const sig = await swap(amountIn, minOut);
  console.log("swap (успех):", link(sig));
  const got = (await getAccount(connection, vaultQuote)).amount;
  const after = await loadPool(raydium, state.poolId);
  const priceAfter = String(after.poolInfo.price);
  console.log(`Получено токена B: ${got}; цена пула: ${priceBefore} -> ${priceAfter}`);

  saveState({
    swap: {
      vaultQuoteAccount: vaultQuote.toBase58(),
      amountIn: amountIn.toString(),
      expectedOut: expectedOut.toString(),
      minAmountOut: minOut.toString(),
      received: got.toString(),
      priceBefore,
      priceAfter,
      successTx: sig,
      rejectedTx: rejectedSig,
      rejected,
    },
  });
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
