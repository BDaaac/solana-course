// Клиент недели 6: создать, обновить и прочитать on-chain профиль (PDA).
import { Keypair, PublicKey } from "@solana/web3.js";
import { anchorProgram } from "../../common/anchor.js";
import { explorer, saveJson } from "../../common/solana.js";
import type { Week6AnchorProfile } from "../target/types/week6_anchor_profile.js";

const DIR = "week6-anchor-profile";

async function main() {
  const { program, provider } = anchorProgram<Week6AnchorProfile>(`${DIR}/target/idl/week6_anchor_profile.json`);
  const authority = provider.wallet.publicKey;
  const pid = program.programId;

  // Адреса PDA детерминированы: их можно вычислить на клиенте без запросов в сеть
  const [profilePda, profileBump] = PublicKey.findProgramAddressSync([Buffer.from("profile"), authority.toBuffer()], pid);
  const username = `dev_${authority.toBase58().slice(0, 6).toLowerCase()}`;
  const [usernamePda] = PublicKey.findProgramAddressSync([Buffer.from("username"), Buffer.from(username)], pid);
  console.log(`Program:      ${pid.toBase58()}`);
  console.log(`Profile PDA:  ${profilePda.toBase58()} (bump ${profileBump})`);
  console.log(`Username PDA: ${usernamePda.toBase58()} (@${username})`);

  const txs: Record<string, string> = {};
  if (await provider.connection.getAccountInfo(profilePda)) {
    txs.deleteOld = await program.methods.deleteProfile().accounts({ authority }).rpc();
    console.log("Старый профиль удалён (повторный запуск)");
  }

  txs.create = await program.methods
    .createProfile(username, "Learning Solana: week 6", "https://avatars.githubusercontent.com/u/35608259")
    .accounts({ authority })
    .rpc();
  txs.update = await program.methods
    .updateProfile("PDA profile works! Built with Anchor", "https://avatars.githubusercontent.com/u/35608259")
    .accounts({ authority })
    .rpc();

  const profile = await program.account.profile.fetch(profilePda);
  const record = await program.account.username.fetch(usernamePda);
  console.log("\nПрофиль on-chain:", {
    username: profile.username,
    bio: profile.bio,
    avatarUrl: profile.avatarUrl,
    createdAt: new Date(profile.createdAt.toNumber() * 1000).toISOString(),
    updatedAt: new Date(profile.updatedAt.toNumber() * 1000).toISOString(),
  });

  // Проверка уникальности: другой кошелёк не может занять то же имя
  let duplicateError = "";
  const other = Keypair.generate();
  try {
    await program.methods
      .createProfile(username, "", "")
      .accounts({ authority: other.publicKey })
      .signers([other])
      .rpc();
  } catch (err) {
    duplicateError = (err as Error).message.split("\n")[0];
    console.log(`Повторное имя от другого кошелька отклонено: ${duplicateError}`);
  }

  // Все профили программы: getProgramAccounts с фильтром по discriminator делает Anchor
  const all = await program.account.profile.all();
  console.log(`Всего профилей в программе: ${all.length}`);

  Object.entries(txs).forEach(([k, sig]) => console.log(`${k.padEnd(10)} ${explorer("tx", sig)}`));
  saveJson(`${DIR}/result.json`, {
    programId: pid.toBase58(),
    programExplorer: explorer("address", pid.toBase58()),
    profilePda: profilePda.toBase58(),
    usernamePda: usernamePda.toBase58(),
    profile: {
      authority: profile.authority.toBase58(),
      username: profile.username,
      bio: profile.bio,
      avatarUrl: profile.avatarUrl,
      createdAt: profile.createdAt.toString(),
      updatedAt: profile.updatedAt.toString(),
    },
    usernameRecordOwner: record.authority.toBase58(),
    duplicateUsernameError: duplicateError,
    totalProfiles: all.length,
    transactions: Object.fromEntries(Object.entries(txs).map(([k, s]) => [k, explorer("tx", s)])),
  });
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
