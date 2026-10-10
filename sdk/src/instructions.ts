/**
 * Day 35: compute-budget + settle instruction packaging (+ ALT key plan).
 */

import { Program, type Idl, type AnchorProvider } from "@coral-xyz/anchor";
import {
  ComputeBudgetProgram,
  PublicKey,
  SystemProgram,
  Transaction,
  TransactionInstruction,
  type Connection,
} from "@solana/web3.js";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";

const HERE = dirname(fileURLToPath(import.meta.url));

/** Match Rust `client::SETTLE_CU_LIMIT` / LiteSVM settle tests. */
export const SETTLE_CU_LIMIT = 1_400_000;

/** Optional micro-lamports per CU; 0 omits the price ix. */
export const SETTLE_CU_PRICE_MICRO_LAMPORTS = 0;

/** Static keys to put in a settle Address Lookup Table (Day 32 plan). */
export const SETTLE_ALT_STATIC_LABELS = [
  "token_program",
  "system_program",
  "global_config",
  "vault",
  "vault_token_a",
  "vault_token_b",
] as const;

export const SEEDS = {
  globalConfig: Buffer.from("global-config"),
  spotVault: Buffer.from("spot_vault"),
  nullifier: Buffer.from("nullifier"),
  cleanFundsRoot: Buffer.from("clean_funds_root"),
} as const;

export function loadIdl(
  idlPath: string = join(HERE, "../idl/zk_spot_shield.json"),
): Idl {
  return JSON.parse(readFileSync(idlPath, "utf8")) as Idl;
}

export function programIdFromIdl(idl: Idl = loadIdl()): PublicKey {
  const addr = (idl as Idl & { address?: string }).address;
  if (!addr) throw new Error("IDL missing address");
  return new PublicKey(addr);
}

export function createProgram(
  provider: AnchorProvider,
  idl: Idl = loadIdl(),
): Program {
  return new Program(idl, provider);
}

export function settleComputeBudgetIxs(
  cuLimit: number = SETTLE_CU_LIMIT,
  cuPrice: number = SETTLE_CU_PRICE_MICRO_LAMPORTS,
): TransactionInstruction[] {
  const ixs = [ComputeBudgetProgram.setComputeUnitLimit({ units: cuLimit })];
  if (cuPrice > 0) {
    ixs.push(
      ComputeBudgetProgram.setComputeUnitPrice({ microLamports: cuPrice }),
    );
  }
  return ixs;
}

export type SettleAccounts = {
  payer: PublicKey;
  globalConfig: PublicKey;
  vault: PublicKey;
  cleanFundsRoot: PublicKey;
  nullifierAccount: PublicKey;
  vaultTokenA: PublicKey;
  vaultTokenB: PublicKey;
  userTokenA: PublicKey;
  userTokenB: PublicKey;
};

export async function buildSettleIx(
  program: Program,
  accounts: SettleAccounts,
  proof: Buffer | Uint8Array,
  journal: Buffer | Uint8Array,
): Promise<TransactionInstruction> {
  return program.methods
    .settleShieldedSpot(Buffer.from(proof), Buffer.from(journal))
    .accountsPartial({
      payer: accounts.payer,
      globalConfig: accounts.globalConfig,
      vault: accounts.vault,
      cleanFundsRoot: accounts.cleanFundsRoot,
      nullifierAccount: accounts.nullifierAccount,
      vaultTokenA: accounts.vaultTokenA,
      vaultTokenB: accounts.vaultTokenB,
      userTokenA: accounts.userTokenA,
      userTokenB: accounts.userTokenB,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    })
    .instruction();
}

/** CU budget ixs + settle ix (legacy tx; ALT wiring is address list helpers). */
export async function buildSettleTransaction(
  program: Program,
  accounts: SettleAccounts,
  proof: Buffer | Uint8Array,
  journal: Buffer | Uint8Array,
): Promise<Transaction> {
  const tx = new Transaction();
  for (const ix of settleComputeBudgetIxs()) {
    tx.add(ix);
  }
  tx.add(await buildSettleIx(program, accounts, proof, journal));
  return tx;
}

/** Pubkeys that belong in the static settle ALT (when known). */
export function settleAltStaticAddresses(accounts: {
  globalConfig: PublicKey;
  vault: PublicKey;
  vaultTokenA: PublicKey;
  vaultTokenB: PublicKey;
}): PublicKey[] {
  return [
    TOKEN_PROGRAM_ID,
    SystemProgram.programId,
    accounts.globalConfig,
    accounts.vault,
    accounts.vaultTokenA,
    accounts.vaultTokenB,
  ];
}

export async function findGlobalConfigPda(
  programId: PublicKey,
): Promise<[PublicKey, number]> {
  return PublicKey.findProgramAddressSync([SEEDS.globalConfig], programId);
}

export async function findVaultPda(
  programId: PublicKey,
): Promise<[PublicKey, number]> {
  return PublicKey.findProgramAddressSync([SEEDS.spotVault], programId);
}

export async function findNullifierPda(
  programId: PublicKey,
  nullifier: Buffer | Uint8Array,
): Promise<[PublicKey, number]> {
  return PublicKey.findProgramAddressSync(
    [SEEDS.nullifier, Buffer.from(nullifier)],
    programId,
  );
}

export async function findCleanFundsRootPda(
  programId: PublicKey,
  root: Buffer | Uint8Array,
): Promise<[PublicKey, number]> {
  return PublicKey.findProgramAddressSync(
    [SEEDS.cleanFundsRoot, Buffer.from(root)],
    programId,
  );
}

/** Optional: wait for signature confirmation (Day 38 listener hook). */
export async function confirmSignature(
  connection: Connection,
  signature: string,
  commitment: "processed" | "confirmed" | "finalized" = "confirmed",
): Promise<void> {
  const latest = await connection.getLatestBlockhash(commitment);
  await connection.confirmTransaction(
    { signature, ...latest },
    commitment,
  );
}
