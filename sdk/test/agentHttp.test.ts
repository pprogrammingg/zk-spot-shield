import { describe, expect, it } from "vitest";
import {
  parseBuyBody,
  startAgentHttp,
  withAgentBuyGate,
  AgentBusyError,
} from "../src/agentHttp.js";
import type { AgentBuyFn } from "../src/agentHttp.js";

const ok: Awaited<ReturnType<AgentBuyFn>> = {
  kind: "ok",
  settleSig: "Sig111",
  note: { leafIndex: 0, balance: "333" },
  pay: { paid: true },
  change: { leafIndex: 2, balance: "283" },
  rootSig: "Root111",
};

describe("agent HTTP loopback", () => {
  it("parseBuyBody accepts string or int", () => {
    expect(parseBuyBody({ amount: "50" })).toBe(50n);
    expect(parseBuyBody({ amount: 7 })).toBe(7n);
    expect(() => parseBuyBody({ amount: "1.5" })).toThrow();
    expect(parseBuyBody({ amount: "33" }, 6)).toBe(33_000_000n);
    expect(parseBuyBody({ amount: "0.9" }, 6)).toBe(900_000n);
    expect(() => parseBuyBody({})).toThrow(/amount/);
  });

  it("health + buy omit secrets and set CORS", async () => {
    const { server, origin } = await startAgentHttp({
      port: 0,
      buy: async () => ok,
    });
    try {
      const health = await fetch(`${origin}/health`);
      expect(health.headers.get("access-control-allow-origin")).toBe("*");
      expect(await health.json()).toEqual({ ok: true });

      const pre = await fetch(`${origin}/buy`, { method: "OPTIONS" });
      expect(pre.status).toBe(204);

      const res = await fetch(`${origin}/buy`, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ amount: "50" }),
      });
      const body = (await res.json()) as Record<string, unknown>;
      expect(res.status).toBe(200);
      expect(body.ok).toBe(true);
      expect(body.settleSig).toBe("Sig111");
      expect(body.changeBalance).toBe("283");
      expect(JSON.stringify(body)).not.toMatch(/secret/i);

      const dep = await fetch(`${origin}/deposit`, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ amount: "111" }),
      });
      expect(dep.status).toBe(501);

      const missing = await fetch(`${origin}/status`);
      expect(missing.status).toBe(501);
    } finally {
      await new Promise<void>((r) => server.close(() => r()));
    }
  });

  it("GET /status is secret-free ATAs + notes", async () => {
    const { server, origin } = await startAgentHttp({
      port: 0,
      buy: async () => ok,
      status: () => ({
        ok: true as const,
        cluster: "devnet",
        programId: "Prog111",
        vaultTokenA: "VaultA111",
        vaultTokenB: "VaultB111",
        userTokenA: "UserA111",
        userTokenB: "UserB111",
        mintA: "MintA111",
        mintB: "MintB111",
        decimals: 6,
        notes: [
          {
            leafIndex: 0,
            balance: "33",
            spent: false,
            leafPrefix: "aabbccddeeff",
          },
        ],
      }),
    });
    try {
      const res = await fetch(`${origin}/status`);
      const body = (await res.json()) as Record<string, unknown>;
      expect(res.status).toBe(200);
      expect(body.ok).toBe(true);
      expect(body.cluster).toBe("devnet");
      expect(body.userTokenA).toBe("UserA111");
      expect(body.notes).toEqual([
        {
          leafIndex: 0,
          balance: "33",
          spent: false,
          leafPrefix: "aabbccddeeff",
        },
      ]);
      expect(JSON.stringify(body)).not.toMatch(/secret/i);
    } finally {
      await new Promise<void>((r) => server.close(() => r()));
    }
  });

  it("POST /deposit returns public leaf fields", async () => {
    const { server, origin } = await startAgentHttp({
      port: 0,
      buy: async () => ok,
      deposit: async (amount) => ({
        sig: "DepSig",
        note: { leafIndex: 2, balance: amount.toString() },
        rootSig: "RootDep",
      }),
    });
    try {
      const res = await fetch(`${origin}/deposit`, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ amount: "111" }),
      });
      const body = (await res.json()) as Record<string, unknown>;
      expect(res.status).toBe(200);
      expect(body).toEqual({
        ok: true,
        sig: "DepSig",
        leafIndex: 2,
        balance: "111",
        rootSig: "RootDep",
      });
      expect(JSON.stringify(body)).not.toMatch(/secret/i);
    } finally {
      await new Promise<void>((r) => server.close(() => r()));
    }
  });

  it("underfunded is 422; overlapping buy is 409", async () => {
    let release!: () => void;
    const held = new Promise<Awaited<ReturnType<AgentBuyFn>>>((resolve) => {
      release = () =>
        resolve({ kind: "underfunded", quote: 50n });
    });
    let started!: () => void;
    const startedP = new Promise<void>((r) => {
      started = r;
    });
    let calls = 0;
    const { server, origin } = await startAgentHttp({
      port: 0,
      buy: async () => {
        calls += 1;
        if (calls === 1) {
          started();
          return held;
        }
        return { kind: "underfunded", quote: 1n };
      },
    });
    try {
      const first = fetch(`${origin}/buy`, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ amount: "50" }),
      });
      await startedP;
      const second = await fetch(`${origin}/buy`, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ amount: "50" }),
      });
      expect(second.status).toBe(409);
      release();
      const done = await first;
      expect(done.status).toBe(422);
      const body = (await done.json()) as { quote?: string };
      expect(body.quote).toBe("50");
    } finally {
      await new Promise<void>((r) => server.close(() => r()));
    }
  });

  it("withAgentBuyGate rejects a second caller", async () => {
    let unlock!: () => void;
    const wait = new Promise<void>((r) => {
      unlock = r;
    });
    const first = withAgentBuyGate(() => wait.then(() => 1));
    await expect(withAgentBuyGate(async () => 2)).rejects.toBeInstanceOf(
      AgentBusyError,
    );
    unlock();
    await expect(first).resolves.toBe(1);
  });
});
