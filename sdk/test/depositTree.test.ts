import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import {
  DepositTree,
  computeLeaf,
  hexToBytes32,
  privateInputsFromDeposit,
  verifyMerklePath,
} from "../src/index.js";

const root = join(dirname(fileURLToPath(import.meta.url)), "../..");
const happy = JSON.parse(
  readFileSync(join(root, "zk-circuit/fixtures/happy.json"), "utf8"),
) as {
  secret: string;
  user_address: string;
  balance: number;
  asset_id_mint: string;
  expected_root: string;
  derived: { leaf: string };
};

describe("DepositTree (Day 42)", () => {
  it("first insert matches happy fixture root + leaf", async () => {
    const tree = await DepositTree.create();
    const leaf = await computeLeaf(
      hexToBytes32(happy.secret),
      hexToBytes32(happy.user_address),
      happy.balance,
    );
    const idx = tree.insert(leaf);
    expect(idx).toBe(0);
    const path = await tree.path(0);
    const tip = await tree.root();
    expect(Buffer.from(tip).toString("hex")).toBe(
      happy.expected_root.slice(2),
    );
    expect(Buffer.from(leaf).toString("hex")).toBe(
      happy.derived.leaf.slice(2),
    );
    expect(Buffer.from(await verifyMerklePath(leaf, path)).toString("hex")).toBe(
      happy.expected_root.slice(2),
    );
  });

  it("privateInputsFromDeposit builds prove-ready path", async () => {
    const tree = await DepositTree.create();
    const out = await privateInputsFromDeposit(
      {
        secret: hexToBytes32(happy.secret),
        userAddress: hexToBytes32(happy.user_address),
        balance: happy.balance,
        assetIdMint: hexToBytes32(happy.asset_id_mint),
      },
      tree,
      100,
    );
    expect(out.leafIndex).toBe(0);
    expect(Buffer.from(out.expectedRoot).toString("hex")).toBe(
      happy.expected_root.slice(2),
    );
  });
});
