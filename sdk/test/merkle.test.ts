import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import {
  bytes32ToHex,
  computeLeaf,
  computeNullifier,
  hashNodes,
  hexToBytes32,
  index0EmptyPath,
  verifyMerklePath,
} from "../src/merkle.js";

const root = join(dirname(fileURLToPath(import.meta.url)), "../..");
const happy = JSON.parse(
  readFileSync(join(root, "zk-circuit/fixtures/happy.json"), "utf8"),
) as {
  secret: string;
  user_address: string;
  balance: number;
  asset_id_mint: string;
  expected_root: string;
  merkle_path: { sibling: string; is_right: boolean }[];
  derived: { leaf: string; nullifier: string };
};

describe("Poseidon Merkle (matches zk-circuit-io)", () => {
  it("hash_nodes(0,0) matches happy path level-1 empty sibling", async () => {
    const z = new Uint8Array(32);
    const h = await hashNodes(z, z);
    expect(bytes32ToHex(h)).toBe(happy.merkle_path[1]!.sibling);
  });

  it("compute_leaf matches happy.json derived.leaf", async () => {
    const leaf = await computeLeaf(
      hexToBytes32(happy.secret),
      hexToBytes32(happy.user_address),
      happy.balance,
    );
    expect(bytes32ToHex(leaf)).toBe(happy.derived.leaf);
  });

  it("compute_nullifier matches happy.json derived.nullifier", async () => {
    const leaf = hexToBytes32(happy.derived.leaf);
    const n = await computeNullifier(
      hexToBytes32(happy.secret),
      leaf,
      hexToBytes32(happy.asset_id_mint),
    );
    expect(bytes32ToHex(n)).toBe(happy.derived.nullifier);
  });

  it("index0EmptyPath + verifyMerklePath matches expected_root", async () => {
    const leaf = await computeLeaf(
      hexToBytes32(happy.secret),
      hexToBytes32(happy.user_address),
      happy.balance,
    );
    const path = await index0EmptyPath();
    for (let i = 0; i < 20; i++) {
      expect(bytes32ToHex(path[i]!.sibling)).toBe(happy.merkle_path[i]!.sibling);
      expect(path[i]!.isRight).toBe(true);
    }
    const root = await verifyMerklePath(leaf, path);
    expect(bytes32ToHex(root)).toBe(happy.expected_root);
  });

  it("path from happy.json verifies to expected_root", async () => {
    const leaf = hexToBytes32(happy.derived.leaf);
    const path = happy.merkle_path.map((e) => ({
      sibling: hexToBytes32(e.sibling),
      isRight: e.is_right,
    }));
    const root = await verifyMerklePath(leaf, path);
    expect(bytes32ToHex(root)).toBe(happy.expected_root);
  });
});
