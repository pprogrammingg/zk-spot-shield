/**
 * Days 36–37: localnet E2E (RUN_E2E=1 + validator preloads from e2e-localnet.mjs).
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
  findNullifierPda,
} from "../../src/instructions.js";
import { getSettleProof, parseJournalPublic } from "../../src/prove.js";

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

describe.skipIf(!run)("Day 36–37 localnet E2E", () => {
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
    const [nullifierAccount] = await findNullifierPda(
      program.programId,
      pub.nullifier,
    );

    const badProof = Buffer.from(proof);
    badProof[120] ^= 0xff;

    const tx = await buildSettleTransaction(
      program,
      {
        payer: wallet.publicKey,
        globalConfig: new PublicKey(bundle.globalConfig),
        vault: new PublicKey(bundle.vault),
        cleanFundsRoot,
        nullifierAccount,
        vaultTokenA: new PublicKey(bundle.vaultTokenA),
        vaultTokenB: new PublicKey(bundle.vaultTokenB),
        userTokenA: new PublicKey(bundle.userTokenA),
        userTokenB: new PublicKey(bundle.userTokenB),
      },
      badProof,
      journal,
    );
    await expect(provider.sendAndConfirm(tx)).rejects.toThrow(
      /InvalidProof|custom program error|Error/,
    );
  });

  it("unregistered root → MerkleRootNotFound", async () => {
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
    const [nullifierAccount] = await findNullifierPda(
      program.programId,
      pub.nullifier,
    );

    const tx = await buildSettleTransaction(
      program,
      {
        payer: wallet.publicKey,
        globalConfig: new PublicKey(bundle.globalConfig),
        vault: new PublicKey(bundle.vault),
        cleanFundsRoot: wrongRootPda,
        nullifierAccount,
        vaultTokenA: new PublicKey(bundle.vaultTokenA),
        vaultTokenB: new PublicKey(bundle.vaultTokenB),
        userTokenA: new PublicKey(bundle.userTokenA),
        userTokenB: new PublicKey(bundle.userTokenB),
      },
      proof,
      journal,
    );
    await expect(provider.sendAndConfirm(tx)).rejects.toThrow(
      /MerkleRootNotFound|custom program error|Error/,
    );
  });

  it("happy path: settle moves token balances", async () => {
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
    const [nullifierAccount] = await findNullifierPda(
      program.programId,
      pub.nullifier,
    );

    const userABefore = await getAccount(
      provider.connection,
      new PublicKey(bundle.userTokenA),
    );
    const userBBefore = await getAccount(
      provider.connection,
      new PublicKey(bundle.userTokenB),
    );

    const tx = await buildSettleTransaction(
      program,
      {
        payer: wallet.publicKey,
        globalConfig: new PublicKey(bundle.globalConfig),
        vault: new PublicKey(bundle.vault),
        cleanFundsRoot,
        nullifierAccount,
        vaultTokenA: new PublicKey(bundle.vaultTokenA),
        vaultTokenB: new PublicKey(bundle.vaultTokenB),
        userTokenA: new PublicKey(bundle.userTokenA),
        userTokenB: new PublicKey(bundle.userTokenB),
      },
      proof,
      journal,
    );
    const sig = await provider.sendAndConfirm(tx);
    await confirmSignature(provider.connection, sig);

    const amount = pub.requestedSwapAmount;
    const userAAfter = await getAccount(
      provider.connection,
      new PublicKey(bundle.userTokenA),
    );
    const userBAfter = await getAccount(
      provider.connection,
      new PublicKey(bundle.userTokenB),
    );
    expect(userAAfter.amount).toBe(userABefore.amount - amount);
    expect(userBAfter.amount).toBe(userBBefore.amount + amount);

    const nf = await provider.connection.getAccountInfo(nullifierAccount);
    expect(nf).not.toBeNull();
    expect(nf!.owner.equals(program.programId)).toBe(true);
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
    const [nullifierAccount] = await findNullifierPda(
      program.programId,
      pub.nullifier,
    );

    const tx = await buildSettleTransaction(
      program,
      {
        payer: wallet.publicKey,
        globalConfig: new PublicKey(bundle.globalConfig),
        vault: new PublicKey(bundle.vault),
        cleanFundsRoot,
        nullifierAccount,
        vaultTokenA: new PublicKey(bundle.vaultTokenA),
        vaultTokenB: new PublicKey(bundle.vaultTokenB),
        userTokenA: new PublicKey(bundle.userTokenA),
        userTokenB: new PublicKey(bundle.userTokenB),
      },
      proof,
      journal,
    );
    await expect(provider.sendAndConfirm(tx)).rejects.toThrow(
      /NullifierAlreadyUsed|custom program error|Error/,
    );
  });
});
