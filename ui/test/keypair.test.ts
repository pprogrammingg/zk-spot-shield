import { describe, expect, it } from "vitest";
import { Keypair } from "@solana/web3.js";
import { keypairFromJsonText } from "../src/keypair.ts";

describe("keypairFromJsonText", () => {
  it("round-trips a Solana CLI secret array", () => {
    const kp = Keypair.generate();
    const parsed = keypairFromJsonText(JSON.stringify([...kp.secretKey]));
    expect(parsed.publicKey.equals(kp.publicKey)).toBe(true);
  });

  it("rejects non-arrays", () => {
    expect(() => keypairFromJsonText("{}")).toThrow(/Not a Solana CLI keypair/);
  });
});
