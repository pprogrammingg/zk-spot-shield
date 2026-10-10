/**
 * Day 33: Poseidon Merkle helpers matching `zk-circuit-io` / guest
 * (`light_poseidon` circom BN254, big-endian 32-byte field encoding).
 *
 * Depth-20 tree; path entries are `(sibling, is_right)` as in Rust.
 */

import { buildPoseidon } from "circomlibjs";

export const MERKLE_DEPTH = 20;

export type MerklePathEntry = {
  sibling: Uint8Array;
  /** When true, sibling is on the right: parent = hash(current, sibling). */
  isRight: boolean;
};

type PoseidonFn = ((inputs: unknown[]) => unknown) & {
  F: {
    e: (x: bigint | string | number) => unknown;
    toObject: (x: unknown) => bigint;
  };
};

let poseidonPromise: Promise<PoseidonFn> | null = null;

async function getPoseidon(): Promise<PoseidonFn> {
  if (!poseidonPromise) {
    poseidonPromise = buildPoseidon() as Promise<PoseidonFn>;
  }
  return poseidonPromise;
}

function assertBytes32(label: string, bytes: Uint8Array): void {
  if (bytes.length !== 32) {
    throw new Error(`${label} must be 32 bytes, got ${bytes.length}`);
  }
}

/** Big-endian 32-byte → BN254 field element (circom / light_poseidon). */
function bytes32ToField(poseidon: PoseidonFn, bytes: Uint8Array): unknown {
  assertBytes32("field input", bytes);
  let x = 0n;
  for (const b of bytes) {
    x = (x << 8n) + BigInt(b);
  }
  return poseidon.F.e(x);
}

/** Field element → big-endian 32 bytes. */
function fieldToBytes32(poseidon: PoseidonFn, fe: unknown): Uint8Array {
  const hex = poseidon.F.toObject(fe).toString(16).padStart(64, "0");
  const out = new Uint8Array(32);
  for (let i = 0; i < 32; i++) {
    out[i] = Number.parseInt(hex.slice(i * 2, i * 2 + 2), 16);
  }
  return out;
}

/** `u64` balance as 32-byte BE field (high 24 bytes zero) — matches Rust `compute_leaf`. */
export function balanceToFieldBytes(balance: bigint | number): Uint8Array {
  const n = typeof balance === "number" ? BigInt(balance) : balance;
  if (n < 0n || n > 0xffff_ffff_ffff_ffffn) {
    throw new Error("balance must fit in u64");
  }
  const out = new Uint8Array(32);
  for (let i = 0; i < 8; i++) {
    out[24 + i] = Number((n >> BigInt(56 - 8 * i)) & 0xffn);
  }
  return out;
}

/** 2-to-1 Poseidon node hash (`hash_nodes`). */
export async function hashNodes(
  left: Uint8Array,
  right: Uint8Array,
): Promise<Uint8Array> {
  const poseidon = await getPoseidon();
  const h = poseidon([
    bytes32ToField(poseidon, left),
    bytes32ToField(poseidon, right),
  ]);
  return fieldToBytes32(poseidon, h);
}

/** Note leaf: Poseidon(secret, user_address, balance_be32). */
export async function computeLeaf(
  secret: Uint8Array,
  userAddress: Uint8Array,
  balance: bigint | number,
): Promise<Uint8Array> {
  assertBytes32("secret", secret);
  assertBytes32("userAddress", userAddress);
  const poseidon = await getPoseidon();
  const h = poseidon([
    bytes32ToField(poseidon, secret),
    bytes32ToField(poseidon, userAddress),
    bytes32ToField(poseidon, balanceToFieldBytes(balance)),
  ]);
  return fieldToBytes32(poseidon, h);
}

/** Nullifier: Poseidon(secret, leaf, asset_id_mint). */
export async function computeNullifier(
  secret: Uint8Array,
  leaf: Uint8Array,
  assetIdMint: Uint8Array,
): Promise<Uint8Array> {
  assertBytes32("secret", secret);
  assertBytes32("leaf", leaf);
  assertBytes32("assetIdMint", assetIdMint);
  const poseidon = await getPoseidon();
  const h = poseidon([
    bytes32ToField(poseidon, secret),
    bytes32ToField(poseidon, leaf),
    bytes32ToField(poseidon, assetIdMint),
  ]);
  return fieldToBytes32(poseidon, h);
}

/** Climb a depth-20 path to a Merkle root (`verify_merkle_path`). */
export async function verifyMerklePath(
  leaf: Uint8Array,
  path: MerklePathEntry[],
): Promise<Uint8Array> {
  assertBytes32("leaf", leaf);
  if (path.length !== MERKLE_DEPTH) {
    throw new Error(`path must have length ${MERKLE_DEPTH}, got ${path.length}`);
  }
  let current = leaf;
  for (const { sibling, isRight } of path) {
    assertBytes32("sibling", sibling);
    current = isRight
      ? await hashNodes(current, sibling)
      : await hashNodes(sibling, current);
  }
  return current;
}

/**
 * Depth-20 path for leaf index 0 when every other leaf is zero
 * (`index0_empty_path` in Rust).
 */
export async function index0EmptyPath(): Promise<MerklePathEntry[]> {
  const empty: Uint8Array[] = new Array(MERKLE_DEPTH);
  empty[0] = new Uint8Array(32);
  for (let h = 1; h < MERKLE_DEPTH; h++) {
    empty[h] = await hashNodes(empty[h - 1]!, empty[h - 1]!);
  }
  return empty.map((sibling) => ({ sibling, isRight: true }));
}

/** Hex helpers for fixtures (`0x` + 64 hex chars). */
export function hexToBytes32(hex: string): Uint8Array {
  const h = hex.startsWith("0x") || hex.startsWith("0X") ? hex.slice(2) : hex;
  if (h.length !== 64) {
    throw new Error(`expected 32-byte hex, got length ${h.length / 2}`);
  }
  const out = new Uint8Array(32);
  for (let i = 0; i < 32; i++) {
    out[i] = Number.parseInt(h.slice(i * 2, i * 2 + 2), 16);
  }
  return out;
}

export function bytes32ToHex(bytes: Uint8Array): string {
  assertBytes32("bytes", bytes);
  return (
    "0x" +
    Array.from(bytes)
      .map((b) => b.toString(16).padStart(2, "0"))
      .join("")
  );
}
