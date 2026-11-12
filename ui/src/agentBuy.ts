/** Loopback to `npm run agent` (not the relayer on 8787). */

import { parseTokenAmount } from "./amount.ts";

export const AGENT_HTTP_PORTS = [8790, 8791, 8792, 8793, 8794, 8795];
export const AGENT_HTTP_HOSTS = ["127.0.0.1", "localhost"];

export type AgentBuyOk = {
  ok: true;
  settleSig: string;
  spentIndex: number;
  spentBalance: string;
  changeBalance: string | null;
  changeIndex: number | null;
  rootSig: string;
  paid: boolean;
};

export type AgentBuyErr = {
  ok: false;
  error: string;
  quote?: string;
};

export type AgentDepositOk = {
  ok: true;
  sig: string;
  leafIndex: number;
  balance: string;
  rootSig: string;
};

export type AgentStatusNote = {
  leafIndex: number;
  balance: string;
  spent: boolean;
  leafPrefix: string;
};

export type AgentStatus = {
  ok: true;
  cluster: "localnet" | "devnet";
  programId: string;
  vaultTokenA: string;
  vaultTokenB: string;
  userTokenA: string;
  userTokenB: string;
  mintA: string;
  mintB: string;
  decimals?: number;
  notes: AgentStatusNote[];
};

function isStatusNote(n: unknown): n is AgentStatusNote {
  if (!n || typeof n !== "object") return false;
  const o = n as Record<string, unknown>;
  return (
    typeof o.leafIndex === "number" &&
    typeof o.balance === "string" &&
    typeof o.spent === "boolean" &&
    typeof o.leafPrefix === "string"
  );
}

function isAgentStatus(v: unknown): v is AgentStatus {
  if (!v || typeof v !== "object") return false;
  const o = v as Record<string, unknown>;
  return (
    o.ok === true &&
    (o.cluster === "localnet" || o.cluster === "devnet") &&
    typeof o.programId === "string" &&
    typeof o.vaultTokenA === "string" &&
    typeof o.vaultTokenB === "string" &&
    typeof o.userTokenA === "string" &&
    typeof o.userTokenB === "string" &&
    typeof o.mintA === "string" &&
    typeof o.mintB === "string" &&
    Array.isArray(o.notes) &&
    o.notes.every(isStatusNote)
  );
}

/** Human amount string for the agent. `decimals` matches the agent cluster (0 localnet, 6 Devnet). */
export function parseUiBuyAmount(raw: string, decimals = 0): string {
  parseTokenAmount(raw, decimals);
  return raw.trim();
}

/** Same pick as the agent: smallest unspent note that covers the quote. */
export function pickSpendableNoteIndex(
  notes: { leafIndex: number; balance: string; spent: boolean }[],
  quote: bigint,
): number | null {
  if (quote <= 0n) return null;
  const candidates = notes.filter(
    (n) => !n.spent && BigInt(n.balance) >= quote,
  );
  if (candidates.length === 0) return null;
  return candidates.reduce((best, n) =>
    BigInt(n.balance) < BigInt(best.balance) ? n : best,
  ).leafIndex;
}

export async function discoverAgentOrigin(
  fetchFn: typeof fetch = fetch,
): Promise<string | null> {
  const env = (import.meta as { env?: { VITE_AGENT_URL?: string } }).env
    ?.VITE_AGENT_URL;
  if (env) return env.replace(/\/$/, "");
  for (const port of AGENT_HTTP_PORTS) {
    for (const host of AGENT_HTTP_HOSTS) {
      const origin = `http://${host}:${port}`;
      try {
        const res = await fetchFn(`${origin}/health`, {
          signal: AbortSignal.timeout(800),
        });
        const v = (await res.json()) as { ok?: unknown };
        if (res.ok && v.ok === true) return origin;
      } catch {
        /* try next */
      }
    }
  }
  return null;
}

export async function postAgentBuy(
  origin: string,
  amount: string,
  fetchFn: typeof fetch = fetch,
): Promise<AgentBuyOk> {
  const res = await fetchFn(`${origin.replace(/\/$/, "")}/buy`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ amount }),
  });
  const body = (await res.json()) as AgentBuyOk | AgentBuyErr;
  if (!body || body.ok !== true) {
    const err = body && "error" in body ? body.error : `agent HTTP ${res.status}`;
    throw new Error(err);
  }
  return body;
}

export async function fetchAgentStatus(
  origin: string,
  fetchFn: typeof fetch = fetch,
): Promise<AgentStatus | null> {
  const res = await fetchFn(
    `${origin.replace(/\/$/, "")}/status?t=${Date.now()}`,
    { signal: AbortSignal.timeout(1500) },
  );
  if (!res.ok) return null;
  const body: unknown = await res.json();
  return isAgentStatus(body) ? body : null;
}

export async function postAgentDeposit(
  origin: string,
  amount: string,
  fetchFn: typeof fetch = fetch,
): Promise<AgentDepositOk> {
  const res = await fetchFn(`${origin.replace(/\/$/, "")}/deposit`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ amount }),
  });
  const body = (await res.json()) as AgentDepositOk | AgentBuyErr;
  if (!body || body.ok !== true) {
    const err = body && "error" in body ? body.error : `agent HTTP ${res.status}`;
    throw new Error(err);
  }
  return body;
}

export type AgentPublicSwapOk = {
  ok: true;
  swapSig: string;
  paid: boolean;
  payer: string;
};

export async function postAgentPublicSwap(
  origin: string,
  amount: string,
  fetchFn: typeof fetch = fetch,
): Promise<AgentPublicSwapOk> {
  const res = await fetchFn(`${origin.replace(/\/$/, "")}/swap-public`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ amount }),
  });
  const body = (await res.json()) as AgentPublicSwapOk | AgentBuyErr;
  if (!body || body.ok !== true) {
    const err = body && "error" in body ? body.error : `agent HTTP ${res.status}`;
    throw new Error(err);
  }
  return body;
}

export type AgentDepositBOk = {
  ok: true;
  sig: string;
  amount: string;
  reserveB: string;
};

export type AgentMintAOk = {
  ok: true;
  sig: string;
  amount: string;
  userTokenA: string;
};

export async function postAgentDepositB(
  origin: string,
  amount: string,
  fetchFn: typeof fetch = fetch,
): Promise<AgentDepositBOk> {
  const res = await fetchFn(`${origin.replace(/\/$/, "")}/admin/deposit-b`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ amount }),
  });
  const body = (await res.json()) as AgentDepositBOk | AgentBuyErr;
  if (!body || body.ok !== true) {
    const err = body && "error" in body ? body.error : `agent HTTP ${res.status}`;
    throw new Error(err);
  }
  return body;
}

export async function postAgentMintA(
  origin: string,
  amount: string,
  fetchFn: typeof fetch = fetch,
): Promise<AgentMintAOk> {
  const res = await fetchFn(`${origin.replace(/\/$/, "")}/admin/mint-a`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ amount }),
  });
  const body = (await res.json()) as AgentMintAOk | AgentBuyErr;
  if (!body || body.ok !== true) {
    const err = body && "error" in body ? body.error : `agent HTTP ${res.status}`;
    throw new Error(err);
  }
  return body;
}
