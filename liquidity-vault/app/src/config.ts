// Адреса в devnet (см. ../../state.json после запуска scripts/01-vault.ts).
export const RPC_URL = "https://api.devnet.solana.com";
export const TOKEN_MINT = "DDuiXekcXiGckNegNd8vWmbirpPYq41EDTAy9fBHD3sf"; // токен, который принимает vault
export const EXPLORER = (kind: "tx" | "address", v: string) =>
  `https://explorer.solana.com/${kind}/${v}?cluster=devnet`;
