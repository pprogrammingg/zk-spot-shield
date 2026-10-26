import { describe, expect, it } from "vitest";
import {
  getSettleProof,
  loadChangeProofFixture,
  loadHappyProofFixture,
  parseJournalPublic,
} from "../src/prove.js";

describe("prove bridge", () => {
  it("loads committed happy groth16 + journal", () => {
    const bundle = loadHappyProofFixture();
    expect(bundle.source).toBe("fixture");
    expect(bundle.proof.length).toBe(356);
    expect(bundle.journal.length).toBe(104);
  });

  it("loads committed change groth16 + 144-byte journal", () => {
    const bundle = loadChangeProofFixture();
    expect(bundle.source).toBe("fixture");
    expect(bundle.proof.length).toBe(356);
    expect(bundle.journal.length).toBe(144);
    const pub = parseJournalPublic(bundle.journal);
    expect(pub.requestedSwapAmount).toBe(100n);
    expect(pub.changeAmount).toBe(900n);
    expect(pub.changeLeaf.every((b) => b === 0)).toBe(false);
  });

  it("getSettleProof defaults to fixtures", async () => {
    const bundle = await getSettleProof();
    expect(bundle.source).toBe("fixture");
    const pub = parseJournalPublic(bundle.journal);
    expect(pub.requestedSwapAmount).toBe(100n);
    expect(pub.assetIdMint.equals(Buffer.alloc(32, 3))).toBe(true);
    expect(pub.changeAmount).toBe(0n);
  });
});
