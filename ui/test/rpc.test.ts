import { describe, expect, it } from "vitest";
import { parseRpcList, firstHealthyRpc } from "../src/rpc.ts";

describe("UI multi-RPC", () => {
  it("uses the second URL when the first is dead", async () => {
    const urls = parseRpcList("https://dead.invalid, https://ok.example");
    const winner = await firstHealthyRpc(urls, async (url) => {
      if (url.includes("dead")) throw new Error("boom");
      return { ok: true, json: async () => ({ result: "ok" }) };
    });
    expect(winner).toBe("https://ok.example");
  });
});
