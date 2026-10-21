// Недели 8–10: каркас vault, депозит, вывод. Запускается на devnet.
import BN from "bn.js";
import { Keypair, SystemProgram, Transaction, sendAndConfirmTransaction } from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  createMint,
  createAssociatedTokenAccountIdempotent,
  mintTo,
  getAccount,
  getMint,
} from "@solana/spl-token";
import { addr, appendLog, expectFail, link, loadState, saveState, setup, vaultPdas } from "./common.js";

async function main() {
  const { connection, admin, program } = setup();
  const log: Record<string, string> = {};
  let state = loadState();

  // --- Токен, который принимает vault (6 decimals) ---
  if (!state.tokenMint) {
    const mint = await createMint(connection, admin, admin.publicKey, null, 6);
    state = saveState({ tokenMint: mint.toBase58() });
  }
  const tokenMint = new (await import("@solana/web3.js")).PublicKey(state.tokenMint);
  const pda = vaultPdas(program.programId, tokenMint);
  console.log("Vault token mint:", tokenMint.toBase58());

  // --- Неделя 8: initialize_vault ---
  if (!(await connection.getAccountInfo(pda.vaultState))) {
    log.initializeVault = await program.methods
      .initializeVault()
      .accountsPartial({
        vaultState: pda.vaultState,
        vaultTokenAccount: pda.vaultTokens,
        shareMint: pda.shareMint,
        tokenMint,
        admin: admin.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
    console.log("initialize_vault:", link(log.initializeVault));
  }

  // --- Второй пользователь (для проверки долей) ---
  const user = Keypair.generate();
  await sendAndConfirmTransaction(
    connection,
    new Transaction().add(
      SystemProgram.transfer({ fromPubkey: admin.publicKey, toPubkey: user.publicKey, lamports: 30_000_000 }),
    ),
    [admin],
  );

  const adminAta = await createAssociatedTokenAccountIdempotent(connection, admin, tokenMint, admin.publicKey);
  const adminShares = await createAssociatedTokenAccountIdempotent(connection, admin, pda.shareMint, admin.publicKey);
  const userAta = await createAssociatedTokenAccountIdempotent(connection, admin, tokenMint, user.publicKey);
  const userShares = await createAssociatedTokenAccountIdempotent(connection, admin, pda.shareMint, user.publicKey);
  await mintTo(connection, admin, tokenMint, adminAta, admin, 1_000_000_000_000n); // 1 000 000 токенов админу
  await mintTo(connection, admin, tokenMint, userAta, admin, 200_000_000n); //       200 токенов пользователю

  // --- Неделя 8: create_user_card (admin и user) ---
  const adminCard = pda.userCard(admin.publicKey);
  if (!(await connection.getAccountInfo(adminCard))) {
    log.createCardAdmin = await program.methods
      .createUserCard()
      .accountsPartial({ vaultState: pda.vaultState, userCard: adminCard, user: admin.publicKey })
      .rpc();
    console.log("create_user_card (admin):", link(log.createCardAdmin));
  }
  const userCard = pda.userCard(user.publicKey);
  log.createCardUser = await program.methods
    .createUserCard()
    .accountsPartial({ vaultState: pda.vaultState, userCard, user: user.publicKey })
    .signers([user])
    .rpc();
  console.log("create_user_card (user):", link(log.createCardUser));
  await expectFail("вторая карточка тем же пользователем", () =>
    program.methods
      .createUserCard()
      .accountsPartial({ vaultState: pda.vaultState, userCard: adminCard, user: admin.publicKey })
      .rpc(), "already in use");

  // --- Неделя 9: deposit ---
  const depositAccounts = (u: Keypair["publicKey"], card: typeof adminCard, tok: typeof adminAta, sh: typeof adminShares) => ({
    vaultState: pda.vaultState,
    userCard: card,
    vaultTokenAccount: pda.vaultTokens,
    shareMint: pda.shareMint,
    userTokenAccount: tok,
    userShareAccount: sh,
    user: u,
    tokenProgram: TOKEN_PROGRAM_ID,
  });
  await expectFail("депозит нуля", () =>
    program.methods.deposit(new BN(0)).accountsPartial(depositAccounts(admin.publicKey, adminCard, adminAta, adminShares)).rpc(),
    "ZeroAmount");

  log.depositAdmin = await program.methods
    .deposit(new BN(1_000_000_000)) // 1000 токенов
    .accountsPartial(depositAccounts(admin.publicKey, adminCard, adminAta, adminShares))
    .rpc();
  console.log("deposit (admin 1000):", link(log.depositAdmin));

  // Симулируем доход: кто-то перевёл в vault ещё 100 токенов (цена share выросла на 10%).
  await mintTo(connection, admin, tokenMint, pda.vaultTokens, admin, 100_000_000n);
  log.syncBalance = await program.methods
    .syncBalance()
    .accountsPartial({ vaultState: pda.vaultState, vaultTokenAccount: pda.vaultTokens, admin: admin.publicKey })
    .rpc();
  console.log("sync_balance (доход +100):", link(log.syncBalance));
  await expectFail("sync_balance не админом", () =>
    program.methods
      .syncBalance()
      .accountsPartial({ vaultState: pda.vaultState, vaultTokenAccount: pda.vaultTokens, admin: user.publicKey })
      .signers([user])
      .rpc(), "Unauthorized");

  // В vault 1100 токенов / 1000 shares. Пользователь вносит 110 → получает 100 shares (пример из методички).
  log.depositUser = await program.methods
    .deposit(new BN(110_000_000))
    .accountsPartial(depositAccounts(user.publicKey, userCard, userAta, userShares))
    .signers([user])
    .rpc();
  console.log("deposit (user 110):", link(log.depositUser));
  const userSharesBal = (await getAccount(connection, userShares)).amount;
  console.log("  shares пользователя:", userSharesBal.toString(), "(ожидаем 100000000)");
  if (userSharesBal !== 100_000_000n) throw new Error("неверное число shares");

  // --- Неделя 10: withdraw ---
  const before = (await getAccount(connection, userAta)).amount;
  log.withdrawUser = await program.methods
    .withdraw(new BN(50_000_000)) // половина shares
    .accountsPartial(depositAccounts(user.publicKey, userCard, userAta, userShares))
    .signers([user])
    .rpc();
  const after = (await getAccount(connection, userAta)).amount;
  console.log("withdraw (user 50 shares):", link(log.withdrawUser), "получено", (after - before).toString());
  if (after - before !== 55_000_000n) throw new Error("неверная сумма вывода");

  // Закрытие карточки с вкладом запрещено
  await expectFail("close_user_card с вкладом", () =>
    program.methods
      .closeUserCard()
      .accountsPartial({ vaultState: pda.vaultState, userCard, user: user.publicKey })
      .signers([user])
      .rpc(), "CardHasDeposit");

  // Пауза запрещает новые карточки
  await program.methods.setPaused(true).accountsPartial({ vaultState: pda.vaultState, admin: admin.publicKey }).rpc();
  const late = Keypair.generate();
  await expectFail("карточка при паузе", async () => {
    await sendAndConfirmTransaction(connection, new Transaction().add(
      SystemProgram.transfer({ fromPubkey: admin.publicKey, toPubkey: late.publicKey, lamports: 5_000_000 })), [admin]);
    await program.methods.createUserCard()
      .accountsPartial({ vaultState: pda.vaultState, userCard: pda.userCard(late.publicKey), user: late.publicKey })
      .signers([late]).rpc();
  }, "VaultPaused");
  await program.methods.setPaused(false).accountsPartial({ vaultState: pda.vaultState, admin: admin.publicKey }).rpc();

  const vs: any = await (program.account as any).vaultState.fetch(pda.vaultState);
  const summary = {
    programId: program.programId.toBase58(),
    tokenMint: tokenMint.toBase58(),
    vaultState: pda.vaultState.toBase58(),
    vaultTokenAccount: pda.vaultTokens.toBase58(),
    shareMint: pda.shareMint.toBase58(),
    adminCard: adminCard.toBase58(),
    userCard: userCard.toBase58(),
    totalTokens: vs.totalTokens.toString(),
    totalShares: vs.totalShares.toString(),
    shareDecimals: (await getMint(connection, pda.shareMint)).decimals,
    transactions: log,
  };
  saveState({ vault: summary });
  console.log(JSON.stringify(summary, null, 2));
  console.log("Explorer:", addr(pda.vaultState));
  appendLog("runs", { script: "01-vault", at: new Date().toISOString() });
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
