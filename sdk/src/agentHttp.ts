/**
 * Loopback HTTP for the swap UI. Not the relayer (8787). Secrets stay in the agent process.
 */

import {
  createServer,
  type IncomingMessage,
  type Server,
  type ServerResponse,
} from "node:http";
import { formatTokenAmount, parseTokenAmount } from "./tokenAmount.js";

export const DEFAULT_AGENT_HTTP_PORT = 8790;
export const AGENT_HTTP_PORTS = [8790, 8791, 8792, 8793, 8794, 8795];
export const DEFAULT_AGENT_HTTP_HOST = "127.0.0.1";

/** Subset of `BuyResult` — never include note secrets in the HTTP body. */
export type AgentBuyFn = (amount: bigint) => Promise<
  | { kind: "underfunded"; quote: bigint }
  | {
      kind: "ok";
      settleSig: string;
      note: { leafIndex: number; balance: string };
      change?: { leafIndex: number; balance: string };
      rootSig: string | "exists" | "spend-all";
      pay: { paid: boolean };
    }
>;

export type AgentBuyOkJson = {
  ok: true;
  settleSig: string;
  spentIndex: number;
  spentBalance: string;
  changeBalance: string | null;
  changeIndex: number | null;
  rootSig: string;
  paid: boolean;
};

export type AgentBuyErrJson = {
  ok: false;
  error: string;
  quote?: string;
};

export type AgentDepositFn = (amount: bigint) => Promise<{
  sig: string;
  note: { leafIndex: number; balance: string };
  rootSig: string | "exists";
}>;

export type AgentDepositOkJson = {
  ok: true;
  sig: string;
  leafIndex: number;
  balance: string;
  rootSig: string;
};

/** Secret-free snapshot for the swap UI. */
export type AgentStatusNote = {
  leafIndex: number;
  balance: string;
  spent: boolean;
  leafPrefix: string;
};

export type AgentStatusJson = {
  ok: true;
  cluster: "localnet" | "devnet";
  programId: string;
  vaultTokenA: string;
  vaultTokenB: string;
  userTokenA: string;
  userTokenB: string;
  mintA: string;
  mintB: string;
  decimals: number;
  notes: AgentStatusNote[];
};

export type AgentStatusFn = () => AgentStatusJson | Promise<AgentStatusJson>;

const gate = { busy: false };

export class AgentBusyError extends Error {
  readonly code = "BUSY" as const;
  constructor() {
    super("agent already in progress");
    this.name = "AgentBusyError";
  }
}

export async function withAgentBuyGate<T>(fn: () => Promise<T>): Promise<T> {
  if (gate.busy) throw new AgentBusyError();
  gate.busy = true;
  try {
    return await fn();
  } finally {
    gate.busy = false;
  }
}

export function parseBuyBody(raw: unknown, decimals: number = 0): bigint {
  if (!raw || typeof raw !== "object") {
    throw new Error("JSON body required");
  }
  const amount = (raw as { amount?: unknown }).amount;
  if (typeof amount === "number" && Number.isFinite(amount)) {
    return parseTokenAmount(String(amount), decimals);
  }
  if (typeof amount === "string") return parseTokenAmount(amount, decimals);
  throw new Error("amount must be a positive number");
}

function publicBuyJson(
  result: Awaited<ReturnType<AgentBuyFn>>,
  decimals: number,
): AgentBuyOkJson | AgentBuyErrJson {
  if (result.kind === "underfunded") {
    return {
      ok: false,
      error: `no note covers ${formatTokenAmount(result.quote, decimals)}. Deposit more A.`,
      quote: result.quote.toString(),
    };
  }
  return {
    ok: true,
    settleSig: result.settleSig,
    spentIndex: result.note.leafIndex,
    spentBalance: result.note.balance,
    changeBalance: result.change?.balance ?? null,
    changeIndex: result.change?.leafIndex ?? null,
    rootSig: String(result.rootSig),
    paid: result.pay.paid,
  };
}

function setCors(res: ServerResponse): void {
  res.setHeader("Access-Control-Allow-Origin", "*");
  res.setHeader("Access-Control-Allow-Methods", "GET, POST, OPTIONS");
  res.setHeader("Access-Control-Allow-Headers", "content-type");
}

function sendJson(res: ServerResponse, status: number, body: unknown): void {
  setCors(res);
  const payload = JSON.stringify(body);
  res.writeHead(status, { "content-type": "application/json" });
  res.end(payload);
}

function readBody(req: IncomingMessage, limit = 4096): Promise<string> {
  return new Promise((resolve, reject) => {
    const chunks: Buffer[] = [];
    let n = 0;
    req.on("data", (c: Buffer) => {
      n += c.length;
      if (n > limit) {
        reject(new Error("body too large"));
        req.destroy();
        return;
      }
      chunks.push(c);
    });
    req.on("end", () => resolve(Buffer.concat(chunks).toString("utf8")));
    req.on("error", reject);
  });
}

async function handle(
  req: IncomingMessage,
  res: ServerResponse,
  buy: AgentBuyFn,
  deposit: AgentDepositFn | undefined,
  status: AgentStatusFn | undefined,
  decimals: number,
): Promise<void> {
  const url = new URL(req.url ?? "/", "http://127.0.0.1");
  if (req.method === "OPTIONS") {
    setCors(res);
    res.writeHead(204);
    res.end();
    return;
  }
  if (req.method === "GET" && url.pathname === "/health") {
    sendJson(res, 200, { ok: true });
    return;
  }
  if (req.method === "GET" && url.pathname === "/status") {
    if (!status) {
      sendJson(res, 501, {
        ok: false,
        error: "status not enabled",
      } satisfies AgentBuyErrJson);
      return;
    }
    try {
      const body = await status();
      sendJson(res, 200, body);
    } catch (e) {
      sendJson(res, 500, {
        ok: false,
        error: e instanceof Error ? e.message : String(e),
      } satisfies AgentBuyErrJson);
    }
    return;
  }
  if (req.method === "POST" && url.pathname === "/buy") {
    let amount: bigint;
    try {
      const raw = await readBody(req);
      amount = parseBuyBody(raw ? JSON.parse(raw) : null, decimals);
    } catch (e) {
      sendJson(res, 400, {
        ok: false,
        error: e instanceof Error ? e.message : String(e),
      } satisfies AgentBuyErrJson);
      return;
    }
    try {
      const result = await withAgentBuyGate(() => buy(amount));
      const body = publicBuyJson(result, decimals);
      sendJson(res, body.ok ? 200 : 422, body);
    } catch (e) {
      if (e instanceof AgentBusyError) {
        sendJson(res, 409, { ok: false, error: e.message } satisfies AgentBuyErrJson);
        return;
      }
      sendJson(res, 500, {
        ok: false,
        error: e instanceof Error ? e.message : String(e),
      } satisfies AgentBuyErrJson);
    }
    return;
  }
  if (req.method === "POST" && url.pathname === "/deposit") {
    if (!deposit) {
      sendJson(res, 501, {
        ok: false,
        error: "deposit not enabled",
      } satisfies AgentBuyErrJson);
      return;
    }
    let amount: bigint;
    try {
      const raw = await readBody(req);
      amount = parseBuyBody(raw ? JSON.parse(raw) : null, decimals);
    } catch (e) {
      sendJson(res, 400, {
        ok: false,
        error: e instanceof Error ? e.message : String(e),
      } satisfies AgentBuyErrJson);
      return;
    }
    try {
      const result = await withAgentBuyGate(() => deposit(amount));
      sendJson(res, 200, {
        ok: true,
        sig: result.sig,
        leafIndex: result.note.leafIndex,
        balance: result.note.balance,
        rootSig: String(result.rootSig),
      } satisfies AgentDepositOkJson);
    } catch (e) {
      if (e instanceof AgentBusyError) {
        sendJson(res, 409, { ok: false, error: e.message } satisfies AgentBuyErrJson);
        return;
      }
      sendJson(res, 500, {
        ok: false,
        error: e instanceof Error ? e.message : String(e),
      } satisfies AgentBuyErrJson);
    }
    return;
  }
  sendJson(res, 404, { ok: false, error: "not found" } satisfies AgentBuyErrJson);
}

function listen(
  server: Server,
  port: number,
  host: string,
): Promise<number> {
  return new Promise((resolve, reject) => {
    const onErr = (e: Error) => {
      server.off("listening", onListen);
      reject(e);
    };
    const onListen = () => {
      server.off("error", onErr);
      const addr = server.address();
      resolve(typeof addr === "object" && addr ? addr.port : port);
    };
    server.once("error", onErr);
    server.once("listening", onListen);
    server.listen(port, host);
  });
}

function createBuyServer(
  buy: AgentBuyFn,
  deposit: AgentDepositFn | undefined,
  status: AgentStatusFn | undefined,
  decimals: number,
): Server {
  return createServer((req, res) => {
    void handle(req, res, buy, deposit, status, decimals).catch((e: unknown) => {
      if (!res.headersSent) {
        sendJson(res, 500, {
          ok: false,
          error: e instanceof Error ? e.message : String(e),
        });
      }
    });
  });
}

export async function startAgentHttp(opts: {
  buy: AgentBuyFn;
  deposit?: AgentDepositFn;
  status?: AgentStatusFn;
  decimals?: number;
  port?: number;
  host?: string;
}): Promise<{ server: Server; origin: string; port: number }> {
  const host = opts.host ?? DEFAULT_AGENT_HTTP_HOST;
  const decimals = opts.decimals ?? 0;
  const wanted = opts.port;
  const ports =
    wanted === 0
      ? [0]
      : wanted !== undefined
        ? [wanted]
        : AGENT_HTTP_PORTS;
  let last: unknown;
  for (const p of ports) {
    const server = createBuyServer(opts.buy, opts.deposit, opts.status, decimals);
    try {
      const port = await listen(server, p, host);
      return { server, port, origin: `http://${host}:${port}` };
    } catch (e) {
      last = e;
      await new Promise<void>((r) => server.close(() => r()));
    }
  }
  throw last instanceof Error ? last : new Error("agent http listen failed");
}
