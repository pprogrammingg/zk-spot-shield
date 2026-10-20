/**
 * Day 49: high-level SDK — proof inputs, vault-only swap request, vault reads,
 * CleanFundsRoot subscribe helper, relayer settle-intent encoding.
 *
 * Relayer may be `feePayer`; recipient ATAs may differ from the depositor
 * (hidden-destination groundwork — Architecture phases C–E).
 */

import { createHash } from "node:crypto";
import type { Program } from "@coral-xyz/anchor";
import {
  PublicKey,
  type Connection,
  type Transaction,
} from "@solana/web3.js";
import {
  DepositTree,
  privateInputsFromDeposit,
  type DepositNote,
} from "./depositTree.js";
import {
  buildSettleTransaction,
  findCleanFundsRootPda,
  findNullifierPda,
  findVaultPda,
  type SettleAccounts,
} from "./instructions.js";
import { bytes32ToHex, type MerklePathEntry } from "./merkle.js";
import { getSettleProof, parseJournalPublic, type ProofBundle } from "./prove.js";

/** Guest-shaped private inputs (matches `zk-circuit-io::PrivateInputs`). */
export type ProofInputs = {
  secret: Uint8Array;
  userAddress: Uint8Array;
  merklePath: MerklePathEntry[];
  balance: bigint;
  requestedSwapAmount: bigint;
  assetIdMint: Uint8Array;
  expectedRoot: Uint8Array;
  leaf: Uint8Array;
  leafIndex: number;
};

/** Insert note, return prove-ready witness (no `$PROVE`). */
export async function generateProofInputs(
  note: DepositNote,
  tree: DepositTree,
  requestedSwapAmount: number | bigint,
): Promise<ProofInputs> {
  const swap =
    typeof requestedSwapAmount === "number"
      ? BigInt(requestedSwapAmount)
      : requestedSwapAmount;
  const bal =
    typeof note.balance === "number" ? BigInt(note.balance) : note.balance;
  const packed = await privateInputsFromDeposit(note, tree, swap);
  return {
    secret: Uint8Array.from(note.secret),
    userAddress: Uint8Array.from(note.userAddress),
    merklePath: packed.merklePath,
    balance: bal,
    requestedSwapAmount: swap,
    assetIdMint: Uint8Array.from(note.assetIdMint),
    expectedRoot: packed.expectedRoot,
    leaf: packed.leaf,
    leafIndex: packed.leafIndex,
  };
}

export type VaultReserves = {
  authority: PublicKey;
  mintA: PublicKey;
  mintB: PublicKey;
  reserveA: bigint;
  reserveB: bigint;
  bump: number;
};

const VAULT_BODY = 120;

/** Decode zero-copy `VaultState` (8-byte disc + 120-byte body). */
export function decodeVaultState(data: Buffer | Uint8Array): VaultReserves {
  const buf = Buffer.from(data);
  if (buf.length < 8 + VAULT_BODY) {
    throw new Error(`vault account too small: ${buf.length}`);
  }
  const body = buf.subarray(8);
  return {
    authority: new PublicKey(body.subarray(0, 32)),
    mintA: new PublicKey(body.subarray(32, 64)),
    mintB: new PublicKey(body.subarray(64, 96)),
    reserveA: body.readBigUInt64LE(96),
    reserveB: body.readBigUInt64LE(104),
    bump: body[112]!,
  };
}

export async function getVaultReserves(
  connection: Connection,
  vault: PublicKey,
): Promise<VaultReserves> {
  const acc = await connection.getAccountInfo(vault);
  if (!acc) throw new Error(`vault not found: ${vault.toBase58()}`);
  return decodeVaultState(acc.data);
}

export type SubscribeCleanFundsRootOpts = {
  connection: Connection;
  programId: PublicKey;
  merkleRoot: Buffer | Uint8Array;
  onAccount: (info: { data: Buffer; lamports: number } | null) => void;
};

/** Watch the `CleanFundsRoot` PDA (websocket `onAccountChange`). */
export async function subscribeCleanFundsRoot(
  opts: SubscribeCleanFundsRootOpts,
): Promise<{ pda: PublicKey; subscriptionId: number }> {
  const [pda] = await findCleanFundsRootPda(opts.programId, opts.merkleRoot);
  const subscriptionId = opts.connection.onAccountChange(pda, (acc) => {
    if (!acc) {
      opts.onAccount(null);
      return;
    }
    opts.onAccount({ data: Buffer.from(acc.data), lamports: acc.lamports });
  });
  return { pda, subscriptionId };
}

export type ShieldedSwapRequest = {
  program: Program;
  /** SOL fee payer — relayer key is allowed (Day 46). */
  feePayer: PublicKey;
  vaultTokenA: PublicKey;
  vaultTokenB: PublicKey;
  /** Depositor / default recipient ATA mint A. */
  userTokenA: PublicKey;
  userTokenB: PublicKey;
  /**
   * Optional payout ATAs when recipient ≠ depositor (unlink groundwork).
   * Defaults to `userTokenA` / `userTokenB`.
   */
  recipientTokenA?: PublicKey;
  recipientTokenB?: PublicKey;
  proof?: ProofBundle;
};

export type ShieldedSwapPlan = {
  tx: Transaction;
  accounts: SettleAccounts;
  proof: Buffer;
  journal: Buffer;
  requestedSwapAmount: bigint;
  nullifier: Buffer;
  merkleRoot: Buffer;
  recipientTokenA: PublicKey;
  recipientTokenB: PublicKey;
  feePayer: PublicKey;
};

/**
 * Vault-only settle plan: fixture (or supplied) Groth16 + journal, recipient
 * ATAs may differ from depositor, fee payer may be a relayer.
 */
export async function requestShieldedSwap(
  req: ShieldedSwapRequest,
): Promise<ShieldedSwapPlan> {
  const bundle = req.proof ?? (await getSettleProof());
  const pub = parseJournalPublic(bundle.journal);
  const recipientTokenA = req.recipientTokenA ?? req.userTokenA;
  const recipientTokenB = req.recipientTokenB ?? req.userTokenB;
  const [vault] = await findVaultPda(req.program.programId);
  const [cleanFundsRoot] = await findCleanFundsRootPda(
    req.program.programId,
    pub.merkleRoot,
  );
  const [nullifierAccount] = await findNullifierPda(
    req.program.programId,
    pub.nullifier,
  );
  const [globalConfig] = PublicKey.findProgramAddressSync(
    [Buffer.from("global-config")],
    req.program.programId,
  );
  const accounts: SettleAccounts = {
    payer: req.feePayer,
    globalConfig,
    vault,
    cleanFundsRoot,
    nullifierAccount,
    vaultTokenA: req.vaultTokenA,
    vaultTokenB: req.vaultTokenB,
    userTokenA: recipientTokenA,
    userTokenB: recipientTokenB,
  };
  const tx = await buildSettleTransaction(
    req.program,
    accounts,
    bundle.proof,
    bundle.journal,
  );
  return {
    tx,
    accounts,
    proof: bundle.proof,
    journal: bundle.journal,
    requestedSwapAmount: pub.requestedSwapAmount,
    nullifier: Buffer.from(pub.nullifier),
    merkleRoot: Buffer.from(pub.merkleRoot),
    recipientTokenA,
    recipientTokenB,
    feePayer: req.feePayer,
  };
}

/** Relayer `SettleIntent` JSON (Day 46) for `POST /submit-intent`. */
export type RelayerSettleIntent = {
  proof_base64: string;
  journal_base64: string;
  global_config: string;
  vault: string;
  clean_funds_root: string;
  nullifier_account: string;
  vault_token_a: string;
  vault_token_b: string;
  user_token_a: string;
  user_token_b: string;
  amount: number;
};

export function encodeSettleIntent(plan: ShieldedSwapPlan): RelayerSettleIntent {
  return {
    proof_base64: plan.proof.toString("base64"),
    journal_base64: plan.journal.toString("base64"),
    global_config: plan.accounts.globalConfig.toBase58(),
    vault: plan.accounts.vault.toBase58(),
    clean_funds_root: plan.accounts.cleanFundsRoot.toBase58(),
    nullifier_account: plan.accounts.nullifierAccount.toBase58(),
    vault_token_a: plan.accounts.vaultTokenA.toBase58(),
    vault_token_b: plan.accounts.vaultTokenB.toBase58(),
    user_token_a: plan.accounts.userTokenA.toBase58(),
    user_token_b: plan.accounts.userTokenB.toBase58(),
    amount: Number(plan.requestedSwapAmount),
  };
}

export function proofInputsToHex(inputs: ProofInputs): Record<string, unknown> {
  return {
    secret: bytes32ToHex(inputs.secret),
    userAddress: bytes32ToHex(inputs.userAddress),
    balance: inputs.balance.toString(),
    requestedSwapAmount: inputs.requestedSwapAmount.toString(),
    assetIdMint: bytes32ToHex(inputs.assetIdMint),
    expectedRoot: bytes32ToHex(inputs.expectedRoot),
    leaf: bytes32ToHex(inputs.leaf),
    leafIndex: inputs.leafIndex,
    merklePathLen: inputs.merklePath.length,
  };
}

/** Anchor disc for `VaultState` (`sha256("account:VaultState")[..8]`). */
export function vaultStateDiscriminator(): Buffer {
  return createHash("sha256")
    .update("account:VaultState")
    .digest()
    .subarray(0, 8);
}
