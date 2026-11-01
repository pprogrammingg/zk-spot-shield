import { describe, expect, it } from "vitest";
import {
  DepositTree,
  computeLeaf,
  hexToBytes32,
  proofInputsForExistingNote,
} from "../src/index.js";
import { buildWitnessFile } from "../src/agentWitness.js";

describe("agent witness JSON", () => {
  it("emits leaves_hex + leaf_index for a two-leaf tree", async () => {
    const note0 = {
      secret: hexToBytes32("0x" + "01".repeat(32)),
      userAddress: hexToBytes32("0x" + "02".repeat(32)),
      balance: 1000n,
      assetIdMint: hexToBytes32("0x" + "03".repeat(32)),
    };
    const note1 = {
      secret: hexToBytes32("0x" + "05".repeat(32)),
      userAddress: hexToBytes32("0x" + "06".repeat(32)),
      balance: 200n,
      assetIdMint: hexToBytes32("0x" + "03".repeat(32)),
    };
    const tree = await DepositTree.create();
    const leaf0 = await computeLeaf(note0.secret, note0.userAddress, note0.balance);
    const leaf1 = await computeLeaf(note1.secret, note1.userAddress, note1.balance);
    tree.insert(leaf0);
    tree.insert(leaf1);
    const packed = await proofInputsForExistingNote(note1, tree, 50n, 1);
    expect(packed.leafIndex).toBe(1);
    expect(tree.length).toBe(2);

    const w = buildWitnessFile({
      note: note1,
      tree,
      leafIndex: packed.leafIndex,
      requestedSwapAmount: 50n,
      leaf: packed.leaf,
    });
    expect(w.leaves_hex).toHaveLength(2);
    expect(w.leaf_index).toBe(1);
    expect(w.requested_swap_amount).toBe(50);
    expect(w.balance).toBe(200);
    expect(JSON.stringify(w)).not.toMatch(/wallet|payer/i);
  });
});
