/**
 * Shared Devnet E2E helpers: pins, faucet, mint A/B, done-markers.
 *
 * Markers live in gitignored `sdk/.devnet-e2e-done.json`. After a case passes it
 * is skipped on later runs unless `DEVNET_FORCE=1` or listed in `DEVNET_CASES`.
 */

import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import {
  Connection,
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram,
  Transaction,
} from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  createAssociatedTokenAccountInstruction,
  getAccount,
  getAssociatedTokenAddressSync,
  mintTo,
} from "@solana/spl-token";
import type { AnchorProvider, Program, Wallet } from "@coral-xyz/anchor";
import BN from "bn.js";
import {
  computeLeaf,
  createLocalhostProvider,
  createProgram,
  decodeGlobalConfig,
  decodeVaultState,
  findDepositLeafPda,
  findGlobalConfigPda,
  findVaultPda,
  loadKeypairWallet,
  programIdFromIdl,
  type VaultReserves,
} from "../../src/index.js";

const HERE = dirname(fileURLToPath(import.meta.url));
export const SDK_ROOT = join(HERE, "../..");

export const PIN = {
  program: "74UW993wrDasAMyBQ53PjHpYLK5vMkWbRRWFZ7kCmM3G",
  mintA: "DNNkZidPWCHxoyuaQ3h5N8FUCWaJZddEYvF7JyHHKbVE",
  mintB: "HfgizYEd9aVxk6hAXjEtoac1jcVHA7KyrqyJYdm5wqzg",
  vkeyHex: "00435696d666d305172306e3895a44017e188663de9b3876970ae270345fc240",
  globalConfig: "BQbrv7CVnh571XjvGv4SMfo5beUcrgF3wtsHrzWAU3CY",
  vault: "A3h8dhmMipthngLSS5AmhsxYvt9sbgLmuUf1wfpJDqmd",
} as const;

/** Case ids — keep in sync with `notes/devnet.md` Day 68 table. */
export const DEVNET_CASES = [
  "reads",
  "pause",
  "deposit_a",
  "deposit_b",
  "public_swap",
  "neg_zero_swap",
  "neg_underflow",
  "neg_paused_swap",
] as const;

export type DevnetCaseId = (typeof DEVNET_CASES)[number];

export type DoneMap = Record<string, { at: string; note?: string }>;

export function donePath(): string {
  return (
    process.env.DEVNET_E2E_DONE ?? join(SDK_ROOT, ".devnet-e2e-done.json")
  );
}

export function loadDone(): DoneMap {
  const path = donePath();
  if (!existsSync(path)) return {};
  return JSON.parse(readFileSync(path, "utf8")) as DoneMap;
}

export function markDone(id: DevnetCaseId, note?: string): void {
  const path = donePath();
  mkdirSync(dirname(path), { recursive: true });
  const cur = loadDone();
  cur[id] = { at: new Date().toISOString(), ...(note ? { note } : {}) };
  writeFileSync(path, JSON.stringify(cur, null, 2) + "\n");
}

export function resetDone(ids?: DevnetCaseId[]): void {
  const path = donePath();
  if (!ids || ids.length === 0) {
    writeFileSync(path, "{}\n");
    return;
  }
  const cur = loadDone();
  for (const id of ids) delete cur[id];
  writeFileSync(path, JSON.stringify(cur, null, 2) + "\n");
}

/**
 * Whether this case should execute now.
 * - Needs `DEVNET_ONCHAIN=1`
 * - `DEVNET_CASES=a,b` limits to that subset (still respects done unless FORCE)
 * - `DEVNET_FORCE=1` ignores done markers
 * - `DEVNET_RESET_DONE=1` clears all markers once at process start
 */
let resetApplied = false;
export function shouldRunCase(id: DevnetCaseId): boolean {
  if (process.env.DEVNET_ONCHAIN !== "1") return false;
  if (process.env.DEVNET_RESET_DONE === "1" && !resetApplied) {
    resetDone();
    resetApplied = true;
  }
  const filter = (process.env.DEVNET_CASES ?? "")
    .split(",")
    .map((s) => s.trim())
    .filter(Boolean);
  if (filter.length > 0 && !filter.includes(id)) return false;
  if (process.env.DEVNET_FORCE === "1") return true;
  return loadDone()[id] === undefined;
}

export function rpcUrl(): string {
  return process.env.ANCHOR_PROVIDER_URL ?? "https://api.devnet.solana.com";
}

export function requireDevnetRpc(url: string = rpcUrl()): void {
  if (!url.includes("devnet")) {
    throw new Error(`ANCHOR_PROVIDER_URL must be Devnet, got ${url}`);
  }
}

export function bytesToHex(u8: Uint8Array): string {
  return Buffer.from(u8).toString("hex");
}

export type DevnetCtx = {
  url: string;
  connection: Connection;
  wallet: Wallet;
  provider: AnchorProvider;
  program: Program;
  programId: PublicKey;
  payer: PublicKey;
  globalConfig: PublicKey;
  vault: PublicKey;
};

export async function openDevnetCtx(): Promise<DevnetCtx> {
  const url = rpcUrl();
  requireDevnetRpc(url);
  const wallet = loadKeypairWallet();
  const provider = createLocalhostProvider({ url, wallet, setGlobal: false });
  const program = createProgram(provider);
  const programId = process.env.DEVNET_PROGRAM_ID
    ? new PublicKey(process.env.DEVNET_PROGRAM_ID)
    : programIdFromIdl();
  if (programId.toBase58() !== PIN.program) {
    throw new Error(
      `program id ${programId.toBase58()} ≠ pin ${PIN.program}`,
    );
  }
  const [globalConfig] = await findGlobalConfigPda(programId);
  const [vault] = await findVaultPda(programId);
  return {
    url,
    connection: provider.connection,
    wallet,
    provider,
    program,
    programId,
    payer: wallet.publicKey,
    globalConfig,
    vault,
  };
}

/** Top up SOL from the Devnet faucet if below `minSol`. No-op when already funded. */
export async function ensureSol(
  connection: Connection,
  pubkey: PublicKey,
  minSol = 0.5,
): Promise<{ balance: number; airdropped: boolean; sig?: string }> {
  const min = Math.floor(minSol * LAMPORTS_PER_SOL);
  let balance = await connection.getBalance(pubkey, "confirmed");
  if (balance >= min) {
    return { balance, airdropped: false };
  }
  const need = Math.max(min - balance, LAMPORTS_PER_SOL);
  const sig = await connection.requestAirdrop(pubkey, need);
  const latest = await connection.getLatestBlockhash("confirmed");
  await connection.confirmTransaction({ signature: sig, ...latest }, "confirmed");
  balance = await connection.getBalance(pubkey, "confirmed");
  if (balance < min) {
    throw new Error(
      `faucet left ${balance} lamports (< ${min}); fund ${pubkey.toBase58()} manually`,
    );
  }
  return { balance, airdropped: true, sig };
}

export async function ensureAta(
  connection: Connection,
  owner: PublicKey,
  mint: PublicKey,
  payer: PublicKey,
  send: (tx: Transaction) => Promise<string>,
): Promise<PublicKey> {
  const ata = getAssociatedTokenAddressSync(mint, owner, false);
  if (!(await connection.getAccountInfo(ata))) {
    await send(
      new Transaction().add(
        createAssociatedTokenAccountInstruction(payer, ata, owner, mint),
      ),
    );
  }
  return ata;
}

/** Mint toy vault mint (authority = admin wallet) into `ata`. Amount is raw base units. */
export async function mintVaultToken(
  ctx: DevnetCtx,
  mint: PublicKey,
  ata: PublicKey,
  amount: bigint,
): Promise<string> {
  const sig = await mintTo(
    ctx.connection,
    // Wallet exposes payer as Keypair via Anchor Wallet
    (ctx.wallet as Wallet & { payer: import("@solana/web3.js").Keypair }).payer,
    mint,
    ata,
    ctx.payer,
    amount,
  );
  return sig;
}

export async function readVault(ctx: DevnetCtx): Promise<VaultReserves> {
  const acc = await ctx.connection.getAccountInfo(ctx.vault);
  if (!acc) throw new Error("vault missing");
  return decodeVaultState(acc.data);
}

export async function readConfig(ctx: DevnetCtx) {
  const acc = await ctx.connection.getAccountInfo(ctx.globalConfig);
  if (!acc) throw new Error("global config missing");
  return decodeGlobalConfig(acc.data);
}

export async function vaultAtas(vault: VaultReserves, vaultPda: PublicKey) {
  return {
    a: getAssociatedTokenAddressSync(vault.mintA, vaultPda, true),
    b: getAssociatedTokenAddressSync(vault.mintB, vaultPda, true),
  };
}

/** Mint + deposit `amount` of mint A or B into the vault; returns deposit sig + leaf. */
export async function mintAndDeposit(
  ctx: DevnetCtx,
  side: "a" | "b",
  amount: bigint,
): Promise<{ sig: string; leafHex: string; reserveA: string; reserveB: string }> {
  await ensureSol(ctx.connection, ctx.payer);
  const vault = await readVault(ctx);
  const mint = side === "a" ? vault.mintA : vault.mintB;
  const send = (tx: Transaction) => ctx.provider.sendAndConfirm(tx);
  const userAta = await ensureAta(ctx.connection, ctx.payer, mint, ctx.payer, send);
  await mintVaultToken(ctx, mint, userAta, amount);
  const atas = await vaultAtas(vault, ctx.vault);
  const vaultToken = side === "a" ? atas.a : atas.b;
  const secret = crypto.getRandomValues(new Uint8Array(32));
  const leaf = await computeLeaf(secret, ctx.payer.toBytes(), amount);
  const [depositLeaf] = await findDepositLeafPda(ctx.programId, leaf);
  const sig = await ctx.program.methods
    .depositShielded([...leaf], new BN(amount.toString()))
    .accountsPartial({
      depositor: ctx.payer,
      vault: ctx.vault,
      depositLeaf,
      vaultToken,
      userToken: userAta,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    })
    .rpc();
  const after = await readVault(ctx);
  return {
    sig,
    leafHex: bytesToHex(leaf),
    reserveA: after.reserveA.toString(),
    reserveB: after.reserveB.toString(),
  };
}

/** Match Anchor / Solana custom errors in rpc() rejects. */
export async function expectFail(
  fn: () => Promise<unknown>,
  pattern: RegExp,
): Promise<void> {
  try {
    await fn();
  } catch (e) {
    const msg = e instanceof Error ? e.message + String(e) : String(e);
    if (pattern.test(msg)) return;
    throw new Error(`expected error matching ${pattern}, got: ${msg}`);
  }
  throw new Error(`expected failure matching ${pattern}`);
}

export async function tokenAmount(
  connection: Connection,
  ata: PublicKey,
): Promise<bigint> {
  return (await getAccount(connection, ata)).amount;
}
