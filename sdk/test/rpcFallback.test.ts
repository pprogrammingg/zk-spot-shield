import { describe, expect, it } from "vitest";
import { firstHealthyRpc, parseRpcList } from "../src/rpcFallback.js";

describe("RPC fallback", () => {
  it("parseRpcList splits comma and newline, drops dupes", () => {
    expect(
      parseRpcList("http://127.0.0.1:8899, https://api.devnet.solana.com\nhttp://127.0.0.1:8899"),
    ).toEqual(["http://127.0.0.1:8899", "https://api.devnet.solana.com"]);
  });

  it("firstHealthyRpc skips a dead endpoint", async () => {
    const calls: string[] = [];
    const url = await firstHealthyRpc(
      ["https://dead.example", "https://ok.example"],
      async (u) => {
        calls.push(u);
        if (u.includes("dead")) {
          throw new Error("network");
        }
        return {
          ok: true,
          json: async () => ({ result: "ok" }),
        };
      },
    );
    expect(url).toBe("https://ok.example");
    expect(calls).toEqual(["https://dead.example", "https://ok.example"]);
  });

  it("firstHealthyRpc fails closed when every URL dies", async () => {
    await expect(
      firstHealthyRpc(["https://a.example"], async () => {
        throw new Error("down");
      }),
    ).rejects.toThrow(/all RPCs failed/);
  });
});
