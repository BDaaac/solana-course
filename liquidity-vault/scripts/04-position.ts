// Неделя 13: vault открывает позицию в CLMM-пуле, собирает комиссии, закрывает, делает ребалансировку.
import BN from "bn.js";
import Decimal from "decimal.js";
import { ComputeBudgetProgram, Keypair, PublicKey, SYSVAR_RENT_PUBKEY, SystemProgram } from "@solana/web3.js";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_2022_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  getAccount,
  getAssociatedTokenAddressSync,
} from "@solana/spl-token";
import {
  DEVNET_PROGRAM_ID,
  DEV_API_URLS,
  PoolUtils,
  Raydium,
  TickUtils,
  TxVersion,
  getPdaExBitmapAccount,
  getPdaPersonalPositionAddress,
  getPdaTickArrayAddress,
} from "@raydium-io/raydium-sdk-v2";
import { link, loadState, saveState, setup, vaultPdas } from "./common.js";

const MEMO = new PublicKey("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr");
const CLMM = DEVNET_PROGRAM_ID.CLMM_PROGRAM_ID;
const CU = ComputeBudgetProgram.setComputeUnitLimit({ units: 800_000 });
const meta = (pubkey: PublicKey, isWritable = false) => ({ pubkey, isSigner: false, isWritable });

async function main() {
  const { connection, admin, program } = setup();
  const state = loadState();
  const tokenA = new PublicKey(state.tokenMint);
  const tokenB = new PublicKey(state.quoteMint);
  const poolId = new PublicKey(state.poolId);
  const pda = vaultPdas(program.programId, tokenA);
  const vaultQuote = new PublicKey(state.swap.vaultQuoteAccount);
  const exBitmap = getPdaExBitmapAccount(CLMM, poolId).publicKey;

  const raydium = await Raydium.load({
    owner: admin, connection, cluster: "devnet",
    disableFeatureCheck: true, disableLoadToken: true, blockhashCommitment: "finalized", urlConfigs: DEV_API_URLS,
  });
  const pool = () => raydium.clmm.getPoolInfoFromRpc(state.poolId);
  const bal = async () => ({
    a: (await getAccount(connection, pda.vaultTokens)).amount,
    b: (await getAccount(connection, vaultQuote)).amount,
  });
  const poolAccounts = async () => {
    const { poolKeys } = await pool();
    return {
      poolState: poolId,
      tokenVault0: new PublicKey(poolKeys.vault.A),
      tokenVault1: new PublicKey(poolKeys.vault.B),
      vault0Mint: tokenA,
      vault1Mint: tokenB,
    };
  };

  /** Открыть позицию из vault: диапазон задан ценами, база — токен A. */
  async function openPosition(lowerPrice: Decimal, upperPrice: Decimal, baseA: BN) {
    const { poolInfo } = await pool();
    const spacing = poolInfo.config.tickSpacing;
    const tickLower = TickUtils.getPriceAndTick({ poolInfo, price: lowerPrice, baseIn: true }).tick;
    const tickUpper = TickUtils.getPriceAndTick({ poolInfo, price: upperPrice, baseIn: true }).tick;
    const epochInfo = await raydium.fetchEpochInfo();
    const r = await PoolUtils.getLiquidityAmountOutFromAmountIn({
      poolInfo, slippage: 0.01, inputA: true, tickLower, tickUpper, amount: baseA, add: true, epochInfo, amountHasFee: false,
    });
    const lowerStart = TickUtils.getTickArrayStartIndexByTick(tickLower, spacing);
    const upperStart = TickUtils.getTickArrayStartIndexByTick(tickUpper, spacing);
    const nft = Keypair.generate();
    const accs = await poolAccounts();
    const before = await bal();
    const personalPosition = getPdaPersonalPositionAddress(CLMM, nft.publicKey).publicKey;
    const positionNftAccount = getAssociatedTokenAddressSync(nft.publicKey, pda.vaultState, true, TOKEN_2022_PROGRAM_ID);
    const tx = await program.methods
      .openRaydiumPosition(
        tickLower, tickUpper, lowerStart, upperStart,
        new BN(r.liquidity.toString()), r.amountSlippageA.amount, r.amountSlippageB.amount,
      )
      .accountsPartial({
        admin: admin.publicKey,
        vaultState: pda.vaultState,
        vaultTokenAccount: pda.vaultTokens,
        vaultWsolAccount: vaultQuote,
        wsolMint: tokenB,
        ...accs,
        positionNftMint: nft.publicKey,
        positionNftAccount,
        personalPosition,
        tickArrayLower: getPdaTickArrayAddress(CLMM, poolId, lowerStart).publicKey,
        tickArrayUpper: getPdaTickArrayAddress(CLMM, poolId, upperStart).publicKey,
        tickArrayBitmap: exBitmap,
        clmmProgram: CLMM,
        rent: SYSVAR_RENT_PUBKEY,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        tokenProgram2022: TOKEN_2022_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      })
      .preInstructions([CU])
      .signers([nft])
      .rpc();
    const after = await bal();
    const pos = {
      nftMint: nft.publicKey.toBase58(),
      personalPosition: personalPosition.toBase58(),
      positionNftAccount: positionNftAccount.toBase58(),
      tickLower, tickUpper, lowerStart, upperStart,
      priceLower: lowerPrice.toString(), priceUpper: upperPrice.toString(),
      priceAtOpen: String(poolInfo.price),
      liquidity: r.liquidity.toString(),
      depositedA: (before.a - after.a).toString(),
      depositedB: (before.b - after.b).toString(),
      tx,
    };
    console.log(`open_raydium_position: ${link(tx)}\n  диапазон ${pos.priceLower}…${pos.priceUpper}, внесено A=${pos.depositedA}, B=${pos.depositedB}`);
    return pos;
  }

  const positionAccounts = async (pos: any) => ({
    admin: admin.publicKey,
    vaultState: pda.vaultState,
    ...(await poolAccounts()),
    positionNftAccount: new PublicKey(pos.positionNftAccount),
    personalPosition: new PublicKey(pos.personalPosition),
    tickArrayLower: getPdaTickArrayAddress(CLMM, poolId, pos.lowerStart).publicKey,
    tickArrayUpper: getPdaTickArrayAddress(CLMM, poolId, pos.upperStart).publicKey,
    clmmProgram: CLMM,
    tokenProgram: TOKEN_PROGRAM_ID,
    tokenProgram2022: TOKEN_2022_PROGRAM_ID,
    memoProgram: MEMO,
  });

  async function collectFees(pos: any) {
    const before = await bal();
    const tx = await program.methods
      .collectFees()
      .accountsPartial({
        ...(await positionAccounts(pos)),
        recipientTokenAccount0: pda.vaultTokens,
        recipientTokenAccount1: vaultQuote,
      })
      .remainingAccounts([meta(exBitmap)])
      .preInstructions([CU])
      .rpc();
    const after = await bal();
    const fees = { a: (after.a - before.a).toString(), b: (after.b - before.b).toString(), tx };
    console.log(`collect_fees: ${link(tx)}\n  комиссии: A=${fees.a}, B=${fees.b}`);
    return fees;
  }

  async function closePosition(pos: any) {
    const { poolInfo } = await pool();
    const epochInfo = await raydium.fetchEpochInfo();
    const exp = await PoolUtils.getAmountsFromLiquidity({
      epochInfo, poolInfo, tickLower: pos.tickLower, tickUpper: pos.tickUpper,
      liquidity: new BN(pos.liquidity), slippage: 0.05, add: false,
    });
    const min0 = exp.amountSlippageA.amount, min1 = exp.amountSlippageB.amount; // минимум к получению (−5%)
    const before = await bal();
    const tx = await program.methods
      .closeRaydiumPosition(min0, min1)
      .accountsPartial({
        ...(await positionAccounts(pos)),
        vaultTokenAccount: pda.vaultTokens,
        vaultWsolAccount: vaultQuote,
        wsolMint: tokenB,
        positionNftMint: new PublicKey(pos.nftMint),
        vault0Mint: tokenA,
        vault1Mint: tokenB,
        systemProgram: SystemProgram.programId,
      })
      .remainingAccounts([meta(exBitmap)])
      .preInstructions([CU])
      .rpc();
    const after = await bal();
    const res = { returnedA: (after.a - before.a).toString(), returnedB: (after.b - before.b).toString(), min0: min0.toString(), min1: min1.toString(), tx };
    console.log(`close_raydium_position: ${link(tx)}\n  вернулось: A=${res.returnedA}, B=${res.returnedB} (минимум ${res.min0}/${res.min1})`);
    return res;
  }

  /** Сделка трейдера (кошелёк админа, не vault) — чтобы позиция зарабатывала комиссии. */
  async function traderSwap(inputA: boolean, amountIn: BN) {
    const { poolInfo, poolKeys, computePoolInfo, tickData } = await pool();
    const epochInfo = await raydium.fetchEpochInfo();
    const q = PoolUtils.computeAmountOutFormat({
      poolInfo: computePoolInfo, tickArrayCache: tickData[state.poolId], amountIn,
      tokenOut: inputA ? poolInfo.mintB : poolInfo.mintA, slippage: 0.05, epochInfo,
    });
    const { execute } = await raydium.clmm.swap({
      poolInfo, poolKeys, inputMint: inputA ? tokenA : tokenB, amountIn,
      amountOutMin: q.minAmountOut.amount.raw, observationId: computePoolInfo.observationId,
      ownerInfo: { useSOLBalance: false }, remainingAccounts: q.remainingAccounts, txVersion: TxVersion.V0,
    });
    await execute({ sendAndConfirm: true });
  }

  /** swap токена A -> B из vault (админ, CPI в Raydium, с защитой от slippage), как на неделе 12. */
  async function vaultSwapAtoB(amountIn: BN) {
    const { poolInfo, poolKeys, computePoolInfo, tickData } = await pool();
    const epochInfo = await raydium.fetchEpochInfo();
    const q = PoolUtils.computeAmountOutFormat({
      poolInfo: computePoolInfo, tickArrayCache: tickData[state.poolId], amountIn,
      tokenOut: poolInfo.mintB, slippage: 0.01, epochInfo,
    });
    const tx = await program.methods
      .swapViaRaydium(amountIn, q.minAmountOut.amount.raw, new BN(0))
      .accountsPartial({
        admin: admin.publicKey, vaultState: pda.vaultState,
        inputTokenAccount: pda.vaultTokens, outputTokenAccount: vaultQuote,
        ammConfig: new PublicKey(poolKeys.config.id), poolState: poolId,
        inputVault: new PublicKey(poolKeys.vault.A), outputVault: new PublicKey(poolKeys.vault.B),
        observationState: computePoolInfo.observationId, inputVaultMint: tokenA, outputVaultMint: tokenB,
        clmmProgram: CLMM, tokenProgram: TOKEN_PROGRAM_ID, tokenProgram2022: TOKEN_2022_PROGRAM_ID, memoProgram: MEMO,
      })
      .remainingAccounts([exBitmap, ...q.remainingAccounts].map((k) => meta(k, true)))
      .preInstructions([CU])
      .rpc();
    console.log(`swap_via_raydium (ребалансировка A→B): ${link(tx)}`);
    return tx;
  }

  const out: any = state.position ?? {};

  // 1. Открыть позицию ±5% вокруг текущей цены (80 токенов A + соответствующее B).
  if (!out.position1) {
    const p0 = new Decimal(String((await pool()).poolInfo.price));
    out.position1 = await openPosition(p0.mul(0.95), p0.mul(1.05), new BN(80_000_000));
    saveState({ position: out });
  }
  const pos1 = out.position1;

  // 2–3. Сделки с другого кошелька (туда-обратно, цена в диапазоне) и сбор комиссий.
  if (!out.fees1) {
    console.log("Сделки трейдера для комиссий…");
    for (let i = 0; i < 6; i++) {
      await traderSwap(true, new BN(4_000_000_000));
      await traderSwap(false, new BN(4_000_000_000));
    }
    console.log("Цена пула после сделок:", String((await pool()).poolInfo.price));
    out.fees1 = await collectFees(pos1);
    saveState({ position: out });
  }

  // 4. Вывести цену за нижнюю границу диапазона.
  if (!out.close1) {
    let price = new Decimal(String((await pool()).poolInfo.price));
    const floor = new Decimal(pos1.priceLower).mul(0.99);
    if (price.gt(floor)) console.log("Сдвигаем цену ниже диапазона…");
    for (let i = 0; i < 40 && price.gt(floor); i++) {
      await traderSwap(true, new BN(5_000_000_000));
      price = new Decimal(String((await pool()).poolInfo.price));
    }
    out.priceOutOfRange = price.toString();
    console.log(`Цена вышла из диапазона: ${price} < ${pos1.priceLower}`);

    // 5. Ребалансировка: закрыть старую позицию.
    out.close1 = await closePosition(pos1);
    saveState({ position: out });
  }

  // ...и открыть новую вокруг новой цены.
  if (!out.position2) {
    const p1 = new Decimal(String((await pool()).poolInfo.price));
    // Старая позиция вернулась почти целиком в токене A: часть меняем на B, чтобы собрать обе ноги новой позиции.
    out.rebalanceSwapTx = await vaultSwapAtoB(new BN(75_000_000));
    out.position2 = await openPosition(p1.mul(0.95), p1.mul(1.05), new BN(80_000_000));
    out.rebalanced = true;
    saveState({ position: out });
  }

  // 6. Для README: проверка прав — не админ не может открыть/закрыть позицию.
  console.log(JSON.stringify(out, null, 2));
}

main().catch((e) => {
  console.error(e);
  if (e?.logs) console.error(e.logs.join("\n"));
  process.exit(1);
});
