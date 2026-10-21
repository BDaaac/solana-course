import { useCallback, useEffect, useState } from "react";
import { useAnchorWallet, useConnection } from "@solana/wallet-adapter-react";
import { WalletMultiButton } from "@solana/wallet-adapter-react-ui";
import { EXPLORER } from "./config";
import * as vault from "./vault";

const DECIMALS = 6;
const fmt = (v: bigint) => (Number(v) / 10 ** DECIMALS).toLocaleString("ru-RU", { maximumFractionDigits: DECIMALS });
const toRaw = (s: string) => BigInt(Math.round(parseFloat(s.replace(",", ".")) * 10 ** DECIMALS));

type Status = { kind: "idle" | "pending" | "ok" | "error"; text?: string; sig?: string };

export function App() {
  const { connection } = useConnection();
  const wallet = useAnchorWallet();
  const [view, setView] = useState<vault.VaultView | null>(null);
  const [amount, setAmount] = useState("");
  const [shares, setShares] = useState("");
  const [status, setStatus] = useState<Status>({ kind: "idle" });

  const refresh = useCallback(async () => {
    if (!wallet) return setView(null);
    try {
      setView(await vault.loadView(connection, wallet));
    } catch (e) {
      setStatus({ kind: "error", text: `Не удалось загрузить vault: ${(e as Error).message}` });
    }
  }, [connection, wallet]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  async function run(label: string, fn: () => Promise<string>) {
    setStatus({ kind: "pending", text: `${label}: подтвердите в кошельке…` });
    try {
      const sig = await fn();
      setStatus({ kind: "ok", text: `${label}: готово`, sig });
      await refresh();
    } catch (e) {
      setStatus({ kind: "error", text: `${label}: ${(e as Error).message}` });
    }
  }

  const price =
    view && view.totalShares > 0n ? Number(view.totalTokens) / Number(view.totalShares) : 1;
  const isAdmin = !!(wallet && view && view.admin === wallet.publicKey.toBase58());

  return (
    <main>
      <header>
        <h1>Liquidity Vault</h1>
        <WalletMultiButton />
      </header>
      <p className="muted">
        Devnet. Вносите токены — получаете share-токены (долю в общем котле). Программа:{" "}
        <a href={EXPLORER("address", vault.programId.toBase58())} target="_blank" rel="noreferrer">
          {vault.programId.toBase58().slice(0, 8)}…
        </a>
      </p>

      {!wallet && <p>Подключите кошелёк (Phantom или Solflare в режиме devnet).</p>}

      {wallet && view && (
        <>
          <section className="grid">
            <Card title="Всего в vault" value={`${fmt(view.totalTokens)} токенов`} />
            <Card title="Выпущено shares" value={fmt(view.totalShares)} />
            <Card title="Цена 1 share" value={price.toFixed(6)} />
            {view.paused && <Card title="Статус" value="пауза" />}
          </section>

          <section className="grid">
            <Card title="Ваши токены" value={fmt(view.userTokens)} />
            <Card title="Ваши share-токены" value={fmt(view.userShares)} />
            <Card title="Внесено (по карточке)" value={fmt(view.deposited)} />
          </section>

          <section className="actions">
            <div>
              <h2>Внести</h2>
              <input value={amount} onChange={(e) => setAmount(e.target.value)} placeholder="Сумма токенов" />
              <button
                disabled={!amount || status.kind === "pending"}
                onClick={() => run("Внесение", async () => (await vault.deposit(connection, wallet, toRaw(amount), view.hasCard)))}
              >
                Внести
              </button>
              {!view.hasCard && <small>Карточка пользователя создастся автоматически.</small>}
            </div>
            <div>
              <h2>Вывести</h2>
              <input value={shares} onChange={(e) => setShares(e.target.value)} placeholder="Сколько share-токенов вернуть" />
              <button
                disabled={!shares || status.kind === "pending"}
                onClick={() => run("Вывод", async () => await vault.withdraw(connection, wallet, toRaw(shares)))}
              >
                Вывести
              </button>
              {shares && <small>≈ {(Number(toRaw(shares)) * price / 10 ** DECIMALS).toFixed(4)} токенов</small>}
            </div>
          </section>

          {isAdmin && (
            <section className="admin">
              <h2>Администратор</h2>
              <p className="muted">
                Swap, открытие/закрытие позиции и сбор комиссий в Raydium выполняются скриптами{" "}
                <code>scripts/03-swap.ts</code> и <code>scripts/04-position.ts</code> (им нужны данные пула Raydium SDK).
                Здесь — управление самим vault.
              </p>
              <button onClick={() => run(view.paused ? "Снятие паузы" : "Пауза", () => vault.setPaused(connection, wallet, !view.paused))}>
                {view.paused ? "Снять паузу" : "Поставить на паузу"}
              </button>
              <button onClick={() => run("Синхронизация баланса", () => vault.syncBalance(connection, wallet))}>
                Синхронизировать баланс (после закрытия позиции)
              </button>
            </section>
          )}
        </>
      )}

      {status.kind !== "idle" && (
        <div className={`status ${status.kind}`}>
          {status.text}{" "}
          {status.sig && (
            <a href={EXPLORER("tx", status.sig)} target="_blank" rel="noreferrer">
              Открыть в Explorer
            </a>
          )}
        </div>
      )}
    </main>
  );
}

function Card({ title, value }: { title: string; value: string }) {
  return (
    <div className="card">
      <div className="muted">{title}</div>
      <div className="value">{value}</div>
    </div>
  );
}
