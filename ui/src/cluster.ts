/** Localnet vs Devnet. RPC + URL `?cluster=` — not a second UI. */

export type Cluster = "localnet" | "devnet";

export const LOCALNET_RPC = "http://127.0.0.1:8899";
export const DEVNET_RPC = "https://api.devnet.solana.com";

export function parseClusterParam(search: string): Cluster | null {
  const q = search.startsWith("?") ? search.slice(1) : search;
  const c = new URLSearchParams(q).get("cluster");
  if (c === "devnet") return "devnet";
  if (c === "localnet") return "localnet";
  return null;
}

export function rpcForCluster(cluster: Cluster): string {
  return cluster === "devnet" ? DEVNET_RPC : LOCALNET_RPC;
}

/** Loopback agent `GET /status` (no Solana RPC). */
export const AGENT_STATUS_POLL_MS = 4_000;

/**
 * On-chain vault / ATA poll. Public Devnet RPC 429s if this is too hot.
 * Localnet can stay faster; Devnet is 15s.
 */
export function chainPollMs(cluster: Cluster): number {
  return cluster === "devnet" ? 15_000 : 8_000;
}

export function clusterFromRpc(rpcList: string): Cluster {
  return rpcList.includes("devnet") ? "devnet" : "localnet";
}

/** Live Devnet vault from `notes/devnet.md`. User ATAs come from the running agent. */
export const DEVNET_VAULT = {
  programId: "74UW993wrDasAMyBQ53PjHpYLK5vMkWbRRWFZ7kCmM3G",
  vaultTokenA: "EVmRfTNFecvTBtW8Zcv5BrQJwZTA5HbSJMVBUJTEdTGQ",
  vaultTokenB: "8wBhwWC9c52D2qgSzYxNsCEN2YMJgsM916c8kBFrMTso",
  mintA: "DNNkZidPWCHxoyuaQ3h5N8FUCWaJZddEYvF7JyHHKbVE",
  mintB: "HfgizYEd9aVxk6hAXjEtoac1jcVHA7KyrqyJYdm5wqzg",
} as const;
