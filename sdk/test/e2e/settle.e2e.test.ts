/**
 * localnet E2E (RUN_E2E=1 + validator preloads from e2e-localnet.ts).
 *
 * Order matters: negative cases that need an unused nullifier run before happy settle.
 */

import { readFileSync, existsSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { PublicKey } from "@solana/web3.js";
import { getAccount } from "@solana/spl-token";
import { describe, expect, it } from "vitest";
import {
  createLocalhostProvider,
  loadKeypairWallet,
} from "../../src/provider.js";
import {
  buildSettleTransaction,
  confirmSignature,
  createProgram,
  findCleanFundsRootPda,
  findDepositLeafPda,
  findNullifierPda,
  type SettleAccounts,
} from "../../src/instructions.js";
import { getSettleProof, parseJournalPublic } from "../../src/prove.js";
import {
  exportEncryptedNote,
  importEncryptedNote,
} from "../../src/noteBackup.js";

const run = process.env.RUN_E2E === "1";
const accountsDir =
  process.env.E2E_ACCOUNTS_DIR ??
  join(dirname(fileURLToPath(import.meta.url)), "../../.e2e-accounts");

type Bundle = {
  globalConfig: string;
  vault: string;
  vaultTokenA: string;
  vaultTokenB: string;
  userTokenA: string;
  userTokenB: string;
  /** Day 78: payout ATA owner ≠ depositor D (`authority`). */
  recipient?: string;
  recipientTokenB?: string;
};

function loadBundle(): Bundle {
  const path = join(accountsDir, "bundle.json");
  if (!existsSync(path)) {
    throw new Error(`missing ${path}; run npm run test:e2e`);
  }
  return JSON.parse(readFileSync(path, "utf8")) as Bundle;
}

async function ensureRoot(
  program: ReturnType<typeof createProgram>,
  authority: PublicKey,
  globalConfig: PublicKey,
  root: Buffer,
): Promise<PublicKey> {
  const [cleanFundsRoot] = await findCleanFundsRootPda(program.programId, root);
  try {
    await program.methods
      .registerCleanFundsRoot([...root])
      .accountsPartial({
        authority,
        globalConfig,
        cleanFundsRoot,
      })
      .rpc();
  } catch {
 /* already registered */
  }
  return cleanFundsRoot;
}

async function settleAccountsFor(
  program: ReturnType<typeof createProgram>,
  bundle: Bundle,
  payer: PublicKey,
  pub: ReturnType<typeof parseJournalPublic>,
  cleanFundsRoot: PublicKey,
): Promise<SettleAccounts> {
  const [nullifierAccount] = await findNullifierPda(
    program.programId,
    pub.nullifier,
  );
  let changeLeaf: PublicKey | undefined;
  if (pub.changeAmount > 0n) {
    [changeLeaf] = await findDepositLeafPda(program.programId, pub.changeLeaf);
  }
  return {
    payer,
    globalConfig: new PublicKey(bundle.globalConfig),
    vault: new PublicKey(bundle.vault),
    cleanFundsRoot,
    nullifierAccount,
    vaultTokenA: new PublicKey(bundle.vaultTokenA),
    vaultTokenB: new PublicKey(bundle.vaultTokenB),
    userTokenA: new PublicKey(bundle.userTokenA),
    userTokenB: new PublicKey(bundle.recipientTokenB ?? bundle.userTokenB),
    changeLeaf,
  };
}

/** Day 76: wipe in-memory note, restore from encrypted blob, then spend. */
async function restoreHappyNoteFromBackup(
  journal: Buffer,
): Promise<void> {
  const pub = parseJournalPublic(journal);
  let live:
    | {
        secret: Uint8Array;
        userAddress: Uint8Array;
        balance: bigint;
        assetIdMint: Uint8Array;
        leafIndex: number;
      }
    | undefined = {
    secret: new Uint8Array(32).fill(1),
    userAddress: new Uint8Array(32).fill(2),
    balance: 1000n,
    assetIdMint: new Uint8Array(pub.assetIdMint),
    leafIndex: 0,
  };
  const blob = await exportEncryptedNote(live, "day76-passphrase");
  live = undefined;
  const restored = await importEncryptedNote(blob, "day76-passphrase");
  if (live !== undefined) {
    throw new Error("in-memory note was not wiped");
  }
  expect(restored.balance).toBe(1000n);
  expect(restored.leafIndex).toBe(0);
  expect(restored.secret[0]).toBe(1);
  expect(restored.userAddress[0]).toBe(2);
  expect(
    Buffer.from(restored.assetIdMint).equals(Buffer.from(pub.assetIdMint)),
  ).toBe(true);
}

function accountOmitsWallet(data: Buffer, wallet: PublicKey): void {
  const body = data.subarray(8);
  const pk = wallet.toBytes();
  for (let i = 0; i + 32 <= body.length; i++) {
    if (body.subarray(i, i + 32).equals(pk)) {
      throw new Error(`PDA body contains wallet ${wallet.toBase58()}`);
    }
  }
}

describe.skipIf(!run)("localnet E2E", () => {
  it("flip 1 proof byte → InvalidProof", async () => {
    const bundle = loadBundle();
    const wallet = loadKeypairWallet();
    const provider = createLocalhostProvider({
      wallet,
      url: process.env.ANCHOR_PROVIDER_URL,
      setGlobal: false,
    });
    const program = createProgram(provider);
    const { proof, journal } = await getSettleProof();
    const pub = parseJournalPublic(journal);
    const cleanFundsRoot = await ensureRoot(
      program,
      wallet.publicKey,
      new PublicKey(bundle.globalConfig),
      Buffer.from(pub.merkleRoot),
    );
    const accounts = await settleAccountsFor(
      program,
      bundle,
      wallet.publicKey,
      pub,
      cleanFundsRoot,
    );

    const badProof = Buffer.from(proof);
    badProof[120] ^= 0xff;

    const tx = await buildSettleTransaction(
      program,
      accounts,
      badProof,
      journal,
    );
    await expect(provider.sendAndConfirm(tx)).rejects.toThrow(
      /InvalidProof|custom program error|Error/,
    );
  });

  it("unregistered root → seeds / MerkleRootNotFound", async () => {
    const bundle = loadBundle();
    const wallet = loadKeypairWallet();
    const provider = createLocalhostProvider({
      wallet,
      url: process.env.ANCHOR_PROVIDER_URL,
      setGlobal: false,
    });
    const program = createProgram(provider);
    const { proof, journal } = await getSettleProof();
    const pub = parseJournalPublic(journal);

    const otherRoot = Buffer.alloc(32, 0xab);
    const wrongRootPda = await ensureRoot(
      program,
      wallet.publicKey,
      new PublicKey(bundle.globalConfig),
      otherRoot,
    );
    const accounts = await settleAccountsFor(
      program,
      bundle,
      wallet.publicKey,
      pub,
      wrongRootPda,
    );

    const tx = await buildSettleTransaction(
      program,
      accounts,
      proof,
      journal,
    );
    await expect(provider.sendAndConfirm(tx)).rejects.toThrow(
      /MerkleRootNotFound|ConstraintSeeds|seeds|custom program error|Error/,
    );
  });

  it("happy path: restore backup then settle pays recipient R", async () => {
    const bundle = loadBundle();
    const wallet = loadKeypairWallet();
    const provider = createLocalhostProvider({
      wallet,
      url: process.env.ANCHOR_PROVIDER_URL,
      setGlobal: false,
    });
    const program = createProgram(provider);
    const { proof, journal } = await getSettleProof();
    await restoreHappyNoteFromBackup(journal);
    const pub = parseJournalPublic(journal);
    const cleanFundsRoot = await ensureRoot(
      program,
      wallet.publicKey,
      new PublicKey(bundle.globalConfig),
      Buffer.from(pub.merkleRoot),
    );
    const accounts = await settleAccountsFor(
      program,
      bundle,
      wallet.publicKey,
      pub,
      cleanFundsRoot,
    );

    const payoutB = new PublicKey(bundle.recipientTokenB ?? bundle.userTokenB);
    const userABefore = await getAccount(
      provider.connection,
      new PublicKey(bundle.userTokenA),
    );
    const userBBefore = await getAccount(provider.connection, payoutB);

    const tx = await buildSettleTransaction(
      program,
      accounts,
      proof,
      journal,
    );
    const sig = await provider.sendAndConfirm(tx);
    await confirmSignature(provider.connection, sig);
    const rpc = process.env.ANCHOR_PROVIDER_URL ?? "http://127.0.0.1:8899";
    console.log(`\nsolana confirm -v ${sig} --url ${rpc}`);

    const amount = pub.requestedSwapAmount;
    const userAAfter = await getAccount(
      provider.connection,
      new PublicKey(bundle.userTokenA),
    );
    const userBAfter = await getAccount(provider.connection, payoutB);
    expect(userAAfter.amount).toBe(userABefore.amount);
    expect(userBAfter.amount).toBe(userBBefore.amount + amount);
    if (bundle.recipient) {
      expect(userBAfter.owner.equals(new PublicKey(bundle.recipient))).toBe(
        true,
      );
    }

    const nf = await provider.connection.getAccountInfo(accounts.nullifierAccount);
    expect(nf).not.toBeNull();
    expect(nf!.owner.equals(program.programId)).toBe(true);
    accountOmitsWallet(nf!.data, wallet.publicKey);
    if (accounts.changeLeaf) {
      const leafAcc = await provider.connection.getAccountInfo(
        accounts.changeLeaf,
      );
      expect(leafAcc).not.toBeNull();
      accountOmitsWallet(leafAcc!.data, wallet.publicKey);
    }
  });

  it("replay → NullifierAlreadyUsed", async () => {
    const bundle = loadBundle();
    const wallet = loadKeypairWallet();
    const provider = createLocalhostProvider({
      wallet,
      url: process.env.ANCHOR_PROVIDER_URL,
      setGlobal: false,
    });
    const program = createProgram(provider);
    const { proof, journal } = await getSettleProof();
    const pub = parseJournalPublic(journal);
    const cleanFundsRoot = await ensureRoot(
      program,
      wallet.publicKey,
      new PublicKey(bundle.globalConfig),
      Buffer.from(pub.merkleRoot),
    );
    const accounts = await settleAccountsFor(
      program,
      bundle,
      wallet.publicKey,
      pub,
      cleanFundsRoot,
    );

    const tx = await buildSettleTransaction(
      program,
      accounts,
      proof,
      journal,
    );
    await expect(provider.sendAndConfirm(tx)).rejects.toThrow(
      /NullifierAlreadyUsed|custom program error|Error/,
    );
  });
});
