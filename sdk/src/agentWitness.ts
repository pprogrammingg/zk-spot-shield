/**
 * Host WITNESS_JSON writer. `leaves_hex` is the full append-only tree so a
 * second deposit or a change note proves against the registered tip.
 */

import { mkdirSync, writeFileSync } from "node:fs";
import { dirname } from "node:path";
import { bytes32ToHex } from "./merkle.js";
import type { DepositNote } from "./depositTree.js";
import type { DepositTree } from "./depositTree.js";

export type AgentWitnessFile = {
  secret_hex: string;
  user_address_hex: string;
  asset_id_mint_hex: string;
  balance: number;
  requested_swap_amount: number;
  expected_leaf_hex: string;
  leaves_hex: string[];
  leaf_index: number;
};

export function buildWitnessFile(args: {
  note: DepositNote;
  tree: DepositTree;
  leafIndex: number;
  requestedSwapAmount: bigint;
  leaf: Uint8Array;
}): AgentWitnessFile {
  const bal =
    typeof args.note.balance === "number"
      ? BigInt(args.note.balance)
      : args.note.balance;
  const maxSafe = BigInt(Number.MAX_SAFE_INTEGER);
  if (args.requestedSwapAmount > maxSafe || bal > maxSafe) {
    throw new Error("witness JSON amounts must fit JSON number (u64 via host)");
  }
  return {
    secret_hex: bytes32ToHex(args.note.secret),
    user_address_hex: bytes32ToHex(args.note.userAddress),
    asset_id_mint_hex: bytes32ToHex(args.note.assetIdMint),
    balance: Number(bal),
    requested_swap_amount: Number(args.requestedSwapAmount),
    expected_leaf_hex: bytes32ToHex(args.leaf),
    leaves_hex: args.tree.toLeaves().map((l) => bytes32ToHex(l)),
    leaf_index: args.leafIndex,
  };
}

export function writeWitnessJson(path: string, file: AgentWitnessFile): void {
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, JSON.stringify(file, null, 2) + "\n");
}
