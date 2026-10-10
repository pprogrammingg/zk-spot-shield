import { describe, expect, it } from "vitest";
import {
  getSettleProof,
  loadHappyProofFixture,
  parseJournalPublic,
} from "../src/prove.js";

describe("Day 34 prove bridge", () => {
  it("loads committed happy groth16 + journal", () => {
    const bundle = loadHappyProofFixture();
    expect(bundle.source).toBe("fixture");
    expect(bundle.proof.length).toBe(356);
    expect(bundle.journal.length).toBe(104);
  });

  it("getSettleProof defaults to fixtures", async () => {
    const bundle = await getSettleProof();
    expect(bundle.source).toBe("fixture");
    const pub = parseJournalPublic(bundle.journal);
    expect(pub.requestedSwapAmount).toBe(100n);
    expect(pub.assetIdMint.equals(Buffer.alloc(32, 3))).toBe(true);
  });
});
