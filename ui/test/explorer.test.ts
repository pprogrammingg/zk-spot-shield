import { describe, expect, it } from "vitest";
import { accountUrl, isLocalRpc, localConfirmCmd, txUrl } from "../src/explorer.ts";

describe("explorer links", () => {
  it("does not send localnet to explorer.solana.com", () => {
    expect(isLocalRpc("http://127.0.0.1:8899")).toBe(true);
    expect(txUrl("sig", "http://127.0.0.1:8899")).toBeNull();
    expect(accountUrl("pk", "http://localhost:8899")).toBeNull();
    expect(localConfirmCmd("sig", "http://127.0.0.1:8899")).toContain("--url http://127.0.0.1:8899");
  });

  it("uses the public explorer on Devnet", () => {
    const href = txUrl("sig", "https://api.devnet.solana.com");
    expect(href).toContain("explorer.solana.com/tx/sig");
    expect(href).toContain("cluster=devnet");
  });
});
