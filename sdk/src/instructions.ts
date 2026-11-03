/**
 * Compute-budget + settle instruction packaging (+ ALT key plan).
 */

import { Program, type Idl, type AnchorProvider } from "@coral-xyz/anchor";
import BN from "bn.js";
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

/** Anchor disc for `settle_shielded_spot`. */
export const SETTLE_DISCRIMINATOR = Buffer.from([
  233, 96, 13, 60, 148, 116, 38, 171,
]);

function borshBytes(bytes: Buffer): Buffer {
  const out = Buffer.alloc(4 + bytes.length);
  out.writeUInt32LE(bytes.length, 0);
  bytes.copy(out, 4);
  return out;
}

/** Optional micro-lamports per CU; 0 omits the price ix. */
export const SETTLE_CU_PRICE_MICRO_LAMPORTS = 0;

/** Static keys to put in a settle Address Lookup Table. */
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
  depositLeaf: Buffer.from("deposit_leaf"),
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

/** Settle metas. `payer` = SOL fee payer (relayer OK); ≠ note owner. */
export type SettleAccounts = {
  /** Fee payer: SOL + nullifier rent. May be a relayer key. */
  payer: PublicKey;
  globalConfig: PublicKey;
  vault: PublicKey;
  cleanFundsRoot: PublicKey;
  nullifierAccount: PublicKey;
  vaultTokenA: PublicKey;
  vaultTokenB: PublicKey;
  /** Recipient ATA mint A (vault-only settle does not debit journal mint). */
  userTokenA: PublicKey;
  /** Recipient ATA mint B. */
  userTokenB: PublicKey;
  /**
   * Remainder `DepositLeafAccount` PDA when the journal binds a change note.
   * Spend-all still occupies this slot with the program id (Anchor `Option` None).
   */
  changeLeaf?: PublicKey;
};

/**
 * Explicit metas. `changeLeaf` sits before Token program. Spend-all passes
 * `program.programId` in that slot — omitting it makes System land on `token_program` (0xbc0).
 */
export async function buildSettleIx(
  program: Program,
  accounts: SettleAccounts,
  proof: Buffer | Uint8Array,
  journal: Buffer | Uint8Array,
): Promise<TransactionInstruction> {
  const keys = [
    { pubkey: accounts.payer, isSigner: true, isWritable: true },
    { pubkey: accounts.globalConfig, isSigner: false, isWritable: false },
    { pubkey: accounts.vault, isSigner: false, isWritable: true },
    { pubkey: accounts.cleanFundsRoot, isSigner: false, isWritable: false },
    { pubkey: accounts.nullifierAccount, isSigner: false, isWritable: true },
    { pubkey: accounts.vaultTokenA, isSigner: false, isWritable: true },
    { pubkey: accounts.vaultTokenB, isSigner: false, isWritable: true },
    { pubkey: accounts.userTokenA, isSigner: false, isWritable: true },
    { pubkey: accounts.userTokenB, isSigner: false, isWritable: true },
    {
      pubkey: accounts.changeLeaf ?? program.programId,
      isSigner: false,
      isWritable: accounts.changeLeaf != null,
    },
    { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
    { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
  ];
  const data = Buffer.concat([
    SETTLE_DISCRIMINATOR,
    borshBytes(Buffer.from(proof)),
    borshBytes(Buffer.from(journal)),
  ]);
  return new TransactionInstruction({
    programId: program.programId,
    keys,
    data,
  });
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

export async function findDepositLeafPda(
  programId: PublicKey,
  leaf: Buffer | Uint8Array,
): Promise<[PublicKey, number]> {
  return PublicKey.findProgramAddressSync(
    [SEEDS.depositLeaf, Buffer.from(leaf)],
    programId,
  );
}

export async function buildSetVkeyHashIx(
  program: Program,
  accounts: { authority: PublicKey; globalConfig: PublicKey },
  vkeyHash: Buffer | Uint8Array,
): Promise<TransactionInstruction> {
  return program.methods
    .setVkeyHash([...Buffer.from(vkeyHash)])
    .accountsPartial({
      authority: accounts.authority,
      globalConfig: accounts.globalConfig,
    })
    .instruction();
}

export async function buildSetVaultMintsIx(
  program: Program,
  accounts: { authority: PublicKey; globalConfig: PublicKey; vault: PublicKey },
  mintA: PublicKey,
  mintB: PublicKey,
): Promise<TransactionInstruction> {
  return program.methods
    .setVaultMints(mintA, mintB)
    .accountsPartial({
      authority: accounts.authority,
      globalConfig: accounts.globalConfig,
      vault: accounts.vault,
    })
    .instruction();
}

export type DepositAccounts = {
  depositor: PublicKey;
  vault: PublicKey;
  depositLeaf: PublicKey;
  vaultToken: PublicKey;
  userToken: PublicKey;
};

/** SPL → vault + record public leaf PDA. */
export async function buildDepositIx(
  program: Program,
  accounts: DepositAccounts,
  leaf: Buffer | Uint8Array,
  amount: number | bigint,
): Promise<TransactionInstruction> {
  const amt = typeof amount === "bigint" ? amount : BigInt(amount);
  return program.methods
    .depositShielded([...Buffer.from(leaf)], new BN(amt.toString()))
    .accountsPartial({
      depositor: accounts.depositor,
      vault: accounts.vault,
      depositLeaf: accounts.depositLeaf,
      vaultToken: accounts.vaultToken,
      userToken: accounts.userToken,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    })
    .instruction();
}

export type PublicSpotSwapAccounts = {
  payer: PublicKey;
  globalConfig: PublicKey;
  vault: PublicKey;
  vaultTokenA: PublicKey;
  vaultTokenB: PublicKey;
  userTokenA: PublicKey;
  userTokenB: PublicKey;
};

/** Public 1:1 A→B. No proof / nullifier. Payer pubkey is on the receipt. */
export async function buildPublicSpotSwapIx(
  program: Program,
  accounts: PublicSpotSwapAccounts,
  amount: number | bigint,
): Promise<TransactionInstruction> {
  const amt = typeof amount === "bigint" ? amount : BigInt(amount);
  return program.methods
    .publicSpotSwap(new BN(amt.toString()))
    .accountsPartial({
      payer: accounts.payer,
      globalConfig: accounts.globalConfig,
      vault: accounts.vault,
      vaultTokenA: accounts.vaultTokenA,
      vaultTokenB: accounts.vaultTokenB,
      userTokenA: accounts.userTokenA,
      userTokenB: accounts.userTokenB,
      tokenProgram: TOKEN_PROGRAM_ID,
    })
    .instruction();
}

/** Optional: wait for signature confirmation. */
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
