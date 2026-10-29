import { Keypair } from "@solana/web3.js";

/** Parse a Solana CLI keypair file (`[n, n, …]` 64-byte secret). */
export function keypairFromJsonText(text: string): Keypair {
  const nums = JSON.parse(text) as unknown;
  if (!Array.isArray(nums) || nums.length < 32 || !nums.every((n) => typeof n === "number")) {
    throw new Error("Not a Solana CLI keypair. File should be a JSON array of numbers.");
  }
  return Keypair.fromSecretKey(Uint8Array.from(nums));
}
