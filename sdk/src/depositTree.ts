/**
 * Append-only depth-20 Poseidon Merkle tree (mirrors `zk-circuit-io::DepositTree`).
 */

import {
  MERKLE_DEPTH,
  computeLeaf,
  hashNodes,
  verifyMerklePath,
  type MerklePathEntry,
} from "./merkle.js";

export type DepositNote = {
  secret: Uint8Array;
  userAddress: Uint8Array;
  balance: number | bigint;
  /** SPL mint spent in the note / settle journal. */
  assetIdMint: Uint8Array;
};

export class DepositTree {
  private leaves: Uint8Array[] = [];
  private empty: Uint8Array[] = [];

  private constructor(empty: Uint8Array[]) {
    this.empty = empty;
  }

  static async create(): Promise<DepositTree> {
    const empty: Uint8Array[] = new Array(MERKLE_DEPTH);
    empty[0] = new Uint8Array(32);
    for (let h = 1; h < MERKLE_DEPTH; h++) {
      empty[h] = await hashNodes(empty[h - 1]!, empty[h - 1]!);
    }
    return new DepositTree(empty);
  }

  get length(): number {
    return this.leaves.length;
  }

  /** Append leaf; returns index. */
  insert(leaf: Uint8Array): number {
    if (leaf.length !== 32) throw new Error("leaf must be 32 bytes");
    if (this.leaves.length >= 1 << MERKLE_DEPTH) throw new Error("tree full");
    const index = this.leaves.length;
    this.leaves.push(Uint8Array.from(leaf));
    return index;
  }

  leafAt(index: number): Uint8Array | undefined {
    const l = this.leaves[index];
    return l ? Uint8Array.from(l) : undefined;
  }

  async path(index: number): Promise<MerklePathEntry[]> {
    if (index < 0 || index >= this.leaves.length) {
      throw new Error("index out of range");
    }
    const path: MerklePathEntry[] = [];
    let idx = index;
    for (let level = 0; level < MERKLE_DEPTH; level++) {
      const siblingIdx = idx ^ 1;
      const sibling = await this.nodeAt(level, siblingIdx);
      path.push({ sibling, isRight: idx % 2 === 0 });
      idx = Math.floor(idx / 2);
    }
    return path;
  }

  async root(): Promise<Uint8Array> {
    if (this.leaves.length === 0) {
      return hashNodes(
        this.empty[MERKLE_DEPTH - 1]!,
        this.empty[MERKLE_DEPTH - 1]!,
      );
    }
    const leaf0 = this.leaves[0]!;
    const p = await this.path(0);
    return verifyMerklePath(leaf0, p);
  }

  private async nodeAt(level: number, index: number): Promise<Uint8Array> {
    if (level === 0) {
      return this.leaves[index]
        ? Uint8Array.from(this.leaves[index]!)
        : new Uint8Array(32);
    }
    const subtreeWidth = 1 << level;
    const start = index * subtreeWidth;
    if (start >= this.leaves.length) {
      return Uint8Array.from(this.empty[level]!);
    }
    const left = await this.nodeAt(level - 1, index * 2);
    const right = await this.nodeAt(level - 1, index * 2 + 1);
    return hashNodes(left, right);
  }
}

/** Insert note leaf and return path + root for prove/execute. */
export async function privateInputsFromDeposit(
  note: DepositNote,
  tree: DepositTree,
  requestedSwapAmount: number | bigint,
): Promise<{
  leaf: Uint8Array;
  leafIndex: number;
  merklePath: MerklePathEntry[];
  expectedRoot: Uint8Array;
}> {
  const leaf = await computeLeaf(note.secret, note.userAddress, note.balance);
  const leafIndex = tree.insert(leaf);
  const merklePath = await tree.path(leafIndex);
  const expectedRoot = await tree.root();
  const swap =
    typeof requestedSwapAmount === "number"
      ? BigInt(requestedSwapAmount)
      : requestedSwapAmount;
  const bal =
    typeof note.balance === "number" ? BigInt(note.balance) : note.balance;
  if (bal < swap) throw new Error("swap exceeds balance");
  return { leaf, leafIndex, merklePath, expectedRoot };
}
