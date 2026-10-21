// Неделя 11: свой второй токен, свой CLMM-пул Raydium в devnet и ручная позиция вокруг цены.
import BN from "bn.js";
import Decimal from "decimal.js";
import { PublicKey } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, createMint, createAssociatedTokenAccountIdempotent, mintTo } from "@solana/spl-token";
import { Raydium, TxVersion, DEVNET_PROGRAM_ID, DEV_API_URLS, TickUtils, PoolUtils } from "@raydium-io/raydium-sdk-v2";
import { addr, link, loadState, saveState, setup } from "./common.js";

const CLMM_PROGRAM = DEVNET_PROGRAM_ID.CLMM_PROGRAM_ID;
const INITIAL_PRICE = new Decimal(1); // 1 токен A = 1 токен B
const RANGE = 0.1; //                  диапазон ±10% вокруг цены

const apiToken = (address: string, symbol: string) => ({
  chainId: 103,
  address,
  programId: TOKEN_PROGRAM_ID.toBase58(),
  logoURI: "",
  symbol,
  name: symbol,
  decimals: 6,
  tags: [],
  extensions: {},
});

async function main() {
  const { connection, admin } = setup();
  let state = loadState();
  if (!state.tokenMint) throw new Error("Сначала запустите 01-vault.ts");

  // --- Второй токен пула ---
  if (!state.quoteMint) {
    const quote = await createMint(connection, admin, admin.publicKey, null, 6);
    const ata = await createAssociatedTokenAccountIdempotent(connection, admin, quote, admin.publicKey);
    await mintTo(connection, admin, quote, ata, admin, 1_000_000_000_000n);
    state = saveState({ quoteMint: quote.toBase58() });
  }
  console.log("Token A (vault):", state.tokenMint, "\nToken B (quote):", state.quoteMint);

  const raydium = await Raydium.load({
    owner: admin,
    connection,
    cluster: "devnet",
    disableFeatureCheck: true,
    disableLoadToken: true,
    blockhashCommitment: "finalized",
    urlConfigs: DEV_API_URLS,
  });

  // --- Создание пула ---
  if (!state.poolId) {
    const configs = await raydium.api.getClmmConfigs();
    const ammConfig = configs.find((c: any) => c.tickSpacing === 10)!;
    const mintA = apiToken(state.tokenMint, "VLT");
    const mintB = apiToken(state.quoteMint, "QTE");
    const { execute, extInfo } = await raydium.clmm.createPool({
      programId: CLMM_PROGRAM,
      mint1: mintA as any,
      mint2: mintB as any,
      ammConfig: { ...ammConfig, id: new PublicKey(ammConfig.id), fundOwner: "", description: "" } as any,
      initialPrice: INITIAL_PRICE,
      txVersion: TxVersion.V0,
    });
    const { txId } = await execute({ sendAndConfirm: true });
    const poolId = String(extInfo.address.id);
    console.log("create_pool:", link(txId), "\nPool:", addr(poolId));
    state = saveState({ poolId, createPoolTx: txId, ammConfigId: ammConfig.id });
  }

  // --- Ручная позиция: чтобы в пуле появилась ликвидность ---
  const { poolInfo, poolKeys } = await raydium.clmm.getPoolInfoFromRpc(state.poolId);
  console.log(`Текущая цена пула: ${poolInfo.price}; tickSpacing=${poolInfo.config.tickSpacing}`);

  const lowerPrice = new Decimal(poolInfo.price).mul(1 - RANGE);
  const upperPrice = new Decimal(poolInfo.price).mul(1 + RANGE);
  const { tick: tickLower } = TickUtils.getPriceAndTick({ poolInfo, price: lowerPrice, baseIn: true });
  const { tick: tickUpper } = TickUtils.getPriceAndTick({ poolInfo, price: upperPrice, baseIn: true });

  const baseAmount = new BN(50_000_000_000); // 50 000 токенов A
  const epochInfo = await raydium.fetchEpochInfo();
  const res = await PoolUtils.getLiquidityAmountOutFromAmountIn({
    poolInfo,
    slippage: 0,
    inputA: true,
    tickUpper,
    tickLower,
    amount: baseAmount,
    add: true,
    epochInfo,
    amountHasFee: false,
  });

  const { execute, extInfo } = await raydium.clmm.openPositionFromBase({
    poolInfo,
    poolKeys,
    tickUpper,
    tickLower,
    base: "MintA",
    ownerInfo: { useSOLBalance: false },
    baseAmount,
    otherAmountMax: res.amountSlippageB.amount,
    txVersion: TxVersion.V0,
    nft2022: true,
  });
  const { txId } = await execute({ sendAndConfirm: true });
  console.log("open_position (вручную):", link(txId), "\nNFT:", extInfo.nftMint.toBase58());

  saveState({
    pool: {
      poolId: state.poolId,
      tokenA: state.tokenMint,
      tokenB: state.quoteMint,
      priceAtCreation: String(poolInfo.price),
      manualPosition: {
        tickLower,
        tickUpper,
        priceLower: lowerPrice.toString(),
        priceUpper: upperPrice.toString(),
        nftMint: extInfo.nftMint.toBase58(),
        tx: txId,
        amountA: baseAmount.toString(),
        amountB: res.amountSlippageB.amount.toString(),
      },
    },
  });
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
