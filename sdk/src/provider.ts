/**
 * Day 33: Anchor provider pointed at localnet (solana-test-validator).
 *
 * Default RPC matches `solana config set --url localhost` → 127.0.0.1:8899.
 * Wallet: `ANCHOR_WALLET` or `~/.config/solana/id.json` (same as Anchor.toml).
 */

import {
  AnchorProvider,
  Wallet,
  setProvider,
} from "@coral-xyz/anchor";
import { Connection, Keypair } from "@solana/web3.js";
import { readFileSync } from "node:fs";
import { homedir } from "node:os";
import { join } from "node:path";

/** Local validator RPC (Agave / solana-test-validator default). */
export const LOCALNET_URL = "http://127.0.0.1:8899";

export type LocalhostProviderOptions = {
  /** Override RPC URL (still typically localhost). */
  url?: string;
  /** Injected wallet; otherwise load from disk. */
  wallet?: Wallet;
  /** Commitment for confirms. */
  commitment?: "processed" | "confirmed" | "finalized";
  /** Also call `setProvider` for Anchor globals. Default true. */
  setGlobal?: boolean;
};

/** Resolve keypair path: `ANCHOR_WALLET` or `~/.config/solana/id.json`. */
export function defaultWalletPath(): string {
  return process.env.ANCHOR_WALLET ?? join(homedir(), ".config", "solana", "id.json");
}

/** Load a Node wallet from a Solana CLI keypair JSON file. */
export function loadKeypairWallet(path: string = defaultWalletPath()): Wallet {
  const raw = JSON.parse(readFileSync(path, "utf8")) as number[];
  const keypair = Keypair.fromSecretKey(Uint8Array.from(raw));
  return new Wallet(keypair);
}

/**
 * Build an `AnchorProvider` for localnet.
 * Does not require the validator to be running until you send a tx / `getLatestBlockhash`.
 */
export function createLocalhostProvider(
  opts: LocalhostProviderOptions = {},
): AnchorProvider {
  const url = opts.url ?? process.env.ANCHOR_PROVIDER_URL ?? LOCALNET_URL;
  const commitment = opts.commitment ?? "confirmed";
  const connection = new Connection(url, commitment);
  const wallet = opts.wallet ?? loadKeypairWallet();
  const provider = new AnchorProvider(connection, wallet, {
    commitment,
    preflightCommitment: commitment,
  });
  if (opts.setGlobal !== false) {
    setProvider(provider);
  }
  return provider;
}

/** True when the provider URL targets loopback localnet. */
export function isLocalhostUrl(url: string): boolean {
  try {
    const u = new URL(url);
    return (
      u.hostname === "127.0.0.1" ||
      u.hostname === "localhost" ||
      u.hostname === "[::1]"
    );
  } catch {
    return false;
  }
}
