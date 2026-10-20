import { PublicKey } from "@solana/web3.js";

export type VaultView = {
  authority: PublicKey;
  mintA: PublicKey;
  mintB: PublicKey;
  reserveA: bigint;
  reserveB: bigint;
  bump: number;
};

export type ConfigView = {
  authority: PublicKey;
  vkeyHash: Uint8Array;
  pauseFlag: boolean;
};

export type JournalView = {
  requestedSwapAmount: bigint;
  assetIdMint: Uint8Array;
  nullifier: Uint8Array;
  merkleRoot: Uint8Array;
};

function u64le(bytes: Uint8Array, offset: number): bigint {
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  return view.getBigUint64(offset, true);
}

export function decodeVaultState(data: Uint8Array): VaultView {
  if (data.length < 8 + 120) {
    throw new Error(`vault account too small: ${data.length}`);
  }
  const body = data.subarray(8);
  return {
    authority: new PublicKey(body.subarray(0, 32)),
    mintA: new PublicKey(body.subarray(32, 64)),
    mintB: new PublicKey(body.subarray(64, 96)),
    reserveA: u64le(body, 96),
    reserveB: u64le(body, 104),
    bump: body[112]!,
  };
}

export function decodeGlobalConfig(data: Uint8Array): ConfigView {
  if (data.length < 8 + 65) {
    throw new Error(`global config too small: ${data.length}`);
  }
  const body = data.subarray(8);
  return {
    authority: new PublicKey(body.subarray(0, 32)),
    vkeyHash: body.subarray(32, 64),
    pauseFlag: body[64] !== 0,
  };
}

export function parseJournal(bytes: Uint8Array): JournalView {
  if (bytes.length !== 104) {
    throw new Error(`journal must be 104 bytes, got ${bytes.length}`);
  }
  return {
    requestedSwapAmount: u64le(bytes, 0),
    assetIdMint: bytes.subarray(8, 40),
    nullifier: bytes.subarray(40, 72),
    merkleRoot: bytes.subarray(72, 104),
  };
}

export function bytesToHex(bytes: Uint8Array): string {
  return [...bytes].map((b) => b.toString(16).padStart(2, "0")).join("");
}

export function decodeBase64(b64: string): Uint8Array {
  const bin = atob(b64.trim());
  const out = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) out[i] = bin.charCodeAt(i);
  return out;
}

export function encodeBase64(bytes: Uint8Array): string {
  let s = "";
  for (const b of bytes) s += String.fromCharCode(b);
  return btoa(s);
}
