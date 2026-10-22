#!/usr/bin/env npx tsx
/**
 * Days 60–61: create GlobalConfig + happy-path CleanFundsRoot, then VaultState.
 *
 *   cd sdk && npm install
 *   export ANCHOR_PROVIDER_URL=https://api.devnet.solana.com
 *   export ANCHOR_WALLET=$HOME/.config/solana/id.json
 *   npx tsx scripts/devnet-init.ts config
 *   npx tsx scripts/devnet-init.ts vault
 *   npx tsx scripts/devnet-init.ts mints
 *   NEW_VKEY_HASH=0x… npx tsx scripts/devnet-init.ts vkey
 *   npx tsx scripts/devnet-init.ts deposit
 *   npx tsx scripts/devnet-init.ts settle
 *
 * Wallet must be protocol admin (pays rent). Do not run until source
 * `declare_id!` matches the on-chain program (notes/devnet.md Day 59).
 */

import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import BN from "bn.js";
import { PublicKey, SystemProgram, Transaction } from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  createAssociatedTokenAccountInstruction,
  createMint,
  getAccount,
  getAssociatedTokenAddressSync,
  mintTo,
} from "@solana/spl-token";
import {
  computeLeaf,
  createLocalhostProvider,
  createProgram,
  decodeVaultState,
  findCleanFundsRootPda,
  findDepositLeafPda,
  findGlobalConfigPda,
  findVaultPda,
  loadHappyProofFixture,
  loadKeypairWallet,
  parseJournalPublic,
  requestShieldedSwap,
} from "../src/index.ts";

const REPO_ROOT = join(dirname(fileURLToPath(import.meta.url)), "../..");

function requireDevnetUrl(url: string): void {
  if (process.env.DEVNET_INIT_ALLOW_LOCAL === "1") return;
  if (!url.includes("devnet")) {
    throw new Error(
      `ANCHOR_PROVIDER_URL must be a Devnet RPC (got ${url}). Set DEVNET_INIT_ALLOW_LOCAL=1 to override.`,
    );
  }
}

function parseVkeyHash(raw: string): number[] {
  const hex = raw.trim().replace(/^0x/i, "");
  if (hex.length !== 64) {
    throw new Error(
      `NEW_VKEY_HASH must be 32-byte hex (got ${hex.length / 2} bytes)`,
    );
  }
  const out: number[] = [];
  for (let i = 0; i < 32; i++) {
    out.push(Number.parseInt(hex.slice(i * 2, i * 2 + 2), 16));
  }
  if (out.some((b) => Number.isNaN(b))) {
    throw new Error("NEW_VKEY_HASH contains non-hex");
  }
  if (out.every((b) => b === 0)) {
    throw new Error("NEW_VKEY_HASH must be non-zero");
  }
  return out;
}

async function setVkey(): Promise<void> {
  const raw = process.env.NEW_VKEY_HASH;
  if (!raw) {
    throw new Error("set NEW_VKEY_HASH=0x… (live guest vk.bytes32())");
  }
  const vkeyHash = parseVkeyHash(raw);
  const url = process.env.ANCHOR_PROVIDER_URL ?? "https://api.devnet.solana.com";
  requireDevnetUrl(url);
  const wallet = loadKeypairWallet();
  const provider = createLocalhostProvider({ url, wallet, setGlobal: false });
  const program = createProgram(provider);
  const [globalConfig] = await findGlobalConfigPda(program.programId);
  const sig = await program.methods
    .setVkeyHash(vkeyHash)
    .accountsPartial({
      authority: wallet.publicKey,
      globalConfig,
    })
    .rpc();
  console.log(
    JSON.stringify(
      {
        globalConfig: globalConfig.toBase58(),
        vkeyHash: Buffer.from(vkeyHash).toString("hex"),
        sig,
        explorerTx: `https://explorer.solana.com/tx/${sig}?cluster=devnet`,
      },
      null,
      2,
    ),
  );
}

async function initConfig(): Promise<void> {
  const url = process.env.ANCHOR_PROVIDER_URL ?? "https://api.devnet.solana.com";
  requireDevnetUrl(url);
  const provider = createLocalhostProvider({ url, setGlobal: false });
  const program = createProgram(provider);
  const [globalConfig] = await findGlobalConfigPda(program.programId);
  const { journal } = loadHappyProofFixture();
  const { merkleRoot } = parseJournalPublic(journal);
  const [cleanFundsRoot] = await findCleanFundsRootPda(
    program.programId,
    merkleRoot,
  );

  const sigCfg = await program.methods
    .initializeGlobalConfig()
    .accountsPartial({
      payer: provider.wallet.publicKey,
      globalConfig,
      systemProgram: SystemProgram.programId,
    })
    .rpc();
  const sigRoot = await program.methods
    .registerCleanFundsRoot([...merkleRoot])
    .accountsPartial({
      authority: provider.wallet.publicKey,
      globalConfig,
      cleanFundsRoot,
      systemProgram: SystemProgram.programId,
    })
    .rpc();

  console.log(
    JSON.stringify(
      {
        programId: program.programId.toBase58(),
        globalConfig: globalConfig.toBase58(),
        cleanFundsRoot: cleanFundsRoot.toBase58(),
        sigCfg,
        sigRoot,
        explorerConfig: `https://explorer.solana.com/address/${globalConfig.toBase58()}?cluster=devnet`,
      },
      null,
      2,
    ),
  );
}

async function initVault(): Promise<void> {
  const url = process.env.ANCHOR_PROVIDER_URL ?? "https://api.devnet.solana.com";
  requireDevnetUrl(url);
  const provider = createLocalhostProvider({ url, setGlobal: false });
  const program = createProgram(provider);
  const [vault] = await findVaultPda(program.programId);
  const sig = await program.methods
    .initializeVault()
    .accountsPartial({
      payer: provider.wallet.publicKey,
      vault,
      systemProgram: SystemProgram.programId,
    })
    .rpc();

  console.log(
    JSON.stringify(
      {
        programId: program.programId.toBase58(),
        vault: vault.toBase58(),
        sig,
        explorerVault: `https://explorer.solana.com/address/${vault.toBase58()}?cluster=devnet`,
      },
      null,
      2,
    ),
  );
}

async function initMints(): Promise<void> {
  const url = process.env.ANCHOR_PROVIDER_URL ?? "https://api.devnet.solana.com";
  requireDevnetUrl(url);
  const wallet = loadKeypairWallet();
  const provider = createLocalhostProvider({ url, wallet, setGlobal: false });
  const program = createProgram(provider);
  const payer = wallet.publicKey;
  const [globalConfig] = await findGlobalConfigPda(program.programId);
  const [vault] = await findVaultPda(program.programId);

  const mintA = process.env.MINT_A
    ? new PublicKey(process.env.MINT_A)
    : await createMint(provider.connection, wallet.payer, payer, null, 6);
  const mintB = process.env.MINT_B
    ? new PublicKey(process.env.MINT_B)
    : await createMint(provider.connection, wallet.payer, payer, null, 6);

  const sigSet = await program.methods
    .setVaultMints(mintA, mintB)
    .accountsPartial({
      authority: payer,
      globalConfig,
      vault,
    })
    .rpc();

  const vaultAtaA = getAssociatedTokenAddressSync(mintA, vault, true);
  const vaultAtaB = getAssociatedTokenAddressSync(mintB, vault, true);
  const ixs = [];
  if (!(await provider.connection.getAccountInfo(vaultAtaA))) {
    ixs.push(
      createAssociatedTokenAccountInstruction(
        payer,
        vaultAtaA,
        vault,
        mintA,
      ),
    );
  }
  if (!(await provider.connection.getAccountInfo(vaultAtaB))) {
    ixs.push(
      createAssociatedTokenAccountInstruction(
        payer,
        vaultAtaB,
        vault,
        mintB,
      ),
    );
  }
  let sigAtas: string | null = null;
  if (ixs.length > 0) {
    const tx = new Transaction().add(...ixs);
    sigAtas = await provider.sendAndConfirm(tx);
  }

  const vaultAcc = await provider.connection.getAccountInfo(vault);
  if (!vaultAcc) throw new Error("vault missing after set_vault_mints");
  const decoded = decodeVaultState(vaultAcc.data);
  const ataA = await getAccount(provider.connection, vaultAtaA);
  const ataB = await getAccount(provider.connection, vaultAtaB);

  console.log(
    JSON.stringify(
      {
        programId: program.programId.toBase58(),
        vault: vault.toBase58(),
        mintA: mintA.toBase58(),
        mintB: mintB.toBase58(),
        vaultAtaA: vaultAtaA.toBase58(),
        vaultAtaB: vaultAtaB.toBase58(),
        vaultMintA: decoded.mintA.toBase58(),
        vaultMintB: decoded.mintB.toBase58(),
        ataAOwner: ataA.owner.toBase58(),
        ataBOwner: ataB.owner.toBase58(),
        sigSet,
        sigAtas,
        explorerVault: `https://explorer.solana.com/address/${vault.toBase58()}?cluster=devnet`,
      },
      null,
      2,
    ),
  );
}

async function ensureUserAta(
  connection: ReturnType<typeof createLocalhostProvider>["connection"],
  payer: PublicKey,
  mint: PublicKey,
  send: (tx: Transaction) => Promise<string>,
): Promise<PublicKey> {
  const ata = getAssociatedTokenAddressSync(mint, payer, false);
  if (!(await connection.getAccountInfo(ata))) {
    const tx = new Transaction().add(
      createAssociatedTokenAccountInstruction(payer, ata, payer, mint),
    );
    await send(tx);
  }
  return ata;
}

/** Day 63 custody: mint 1.0 of each vault mint to the admin wallet and deposit. */
async function depositBoth(): Promise<void> {
  const url = process.env.ANCHOR_PROVIDER_URL ?? "https://api.devnet.solana.com";
  requireDevnetUrl(url);
  const wallet = loadKeypairWallet();
  const provider = createLocalhostProvider({ url, wallet, setGlobal: false });
  const program = createProgram(provider);
  const payer = wallet.publicKey;
  const [vault] = await findVaultPda(program.programId);
  const vaultAcc = await provider.connection.getAccountInfo(vault);
  if (!vaultAcc) throw new Error("vault missing — run mints first");
  const decoded = decodeVaultState(vaultAcc.data);
  if (decoded.mintA.equals(PublicKey.default) || decoded.mintB.equals(PublicKey.default)) {
    throw new Error("vault mints unset — run mints first");
  }

  const amount = 1_000_000n; // 1.0 with 6 decimals
  const send = (tx: Transaction) => provider.sendAndConfirm(tx);
  const userAtaA = await ensureUserAta(provider.connection, payer, decoded.mintA, send);
  const userAtaB = await ensureUserAta(provider.connection, payer, decoded.mintB, send);
  await mintTo(provider.connection, wallet.payer, decoded.mintA, userAtaA, payer, amount);
  await mintTo(provider.connection, wallet.payer, decoded.mintB, userAtaB, payer, amount);

  const vaultAtaA = getAssociatedTokenAddressSync(decoded.mintA, vault, true);
  const vaultAtaB = getAssociatedTokenAddressSync(decoded.mintB, vault, true);
  const userAddr = payer.toBytes();
  const secretA = crypto.getRandomValues(new Uint8Array(32));
  const secretB = crypto.getRandomValues(new Uint8Array(32));
  const leafA = await computeLeaf(secretA, userAddr, amount);
  const leafB = await computeLeaf(secretB, userAddr, amount);
  const [depositLeafA] = await findDepositLeafPda(program.programId, leafA);
  const [depositLeafB] = await findDepositLeafPda(program.programId, leafB);

  const sigA = await program.methods
    .depositShielded([...leafA], new BN(amount.toString()))
    .accountsPartial({
      depositor: payer,
      vault,
      depositLeaf: depositLeafA,
      vaultToken: vaultAtaA,
      userToken: userAtaA,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    })
    .rpc();
  const sigB = await program.methods
    .depositShielded([...leafB], new BN(amount.toString()))
    .accountsPartial({
      depositor: payer,
      vault,
      depositLeaf: depositLeafB,
      vaultToken: vaultAtaB,
      userToken: userAtaB,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    })
    .rpc();

  const after = decodeVaultState(
    (await provider.connection.getAccountInfo(vault))!.data,
  );
  console.log(
    JSON.stringify(
      {
        amount: amount.toString(),
        mintA: decoded.mintA.toBase58(),
        mintB: decoded.mintB.toBase58(),
        vaultAtaA: vaultAtaA.toBase58(),
        vaultAtaB: vaultAtaB.toBase58(),
        depositLeafA: depositLeafA.toBase58(),
        depositLeafB: depositLeafB.toBase58(),
        leafA: Buffer.from(leafA).toString("hex"),
        leafB: Buffer.from(leafB).toString("hex"),
        secretA: Buffer.from(secretA).toString("hex"),
        secretB: Buffer.from(secretB).toString("hex"),
        sigA,
        sigB,
        reserveA: after.reserveA.toString(),
        reserveB: after.reserveB.toString(),
      },
      null,
      2,
    ),
  );
}

async function settleDevnet(): Promise<void> {
  const url = process.env.ANCHOR_PROVIDER_URL ?? "https://api.devnet.solana.com";
  requireDevnetUrl(url);
  const proofDir =
    process.env.PROOF_DIR ?? join(REPO_ROOT, "sp1-artifacts/devnet-settle");
  const bundle = loadHappyProofFixture(proofDir);
  const pub = parseJournalPublic(bundle.journal);
  const wallet = loadKeypairWallet();
  const provider = createLocalhostProvider({ url, wallet, setGlobal: false });
  const program = createProgram(provider);
  const [globalConfig] = await findGlobalConfigPda(program.programId);
  const [vault] = await findVaultPda(program.programId);
  const [cleanFundsRoot] = await findCleanFundsRootPda(
    program.programId,
    pub.merkleRoot,
  );
  const vaultAcc = await provider.connection.getAccountInfo(vault);
  if (!vaultAcc) throw new Error("vault missing");
  const decoded = decodeVaultState(vaultAcc.data);

  let sigRoot: string | null = null;
  const rootInfo = await provider.connection.getAccountInfo(cleanFundsRoot);
  if (!rootInfo) {
    sigRoot = await program.methods
      .registerCleanFundsRoot([...pub.merkleRoot])
      .accountsPartial({
        authority: wallet.publicKey,
        globalConfig,
        cleanFundsRoot,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
  }

  const userTokenA = getAssociatedTokenAddressSync(
    decoded.mintA,
    wallet.publicKey,
    false,
  );
  const userTokenB = getAssociatedTokenAddressSync(
    decoded.mintB,
    wallet.publicKey,
    false,
  );
  const vaultTokenA = getAssociatedTokenAddressSync(decoded.mintA, vault, true);
  const vaultTokenB = getAssociatedTokenAddressSync(decoded.mintB, vault, true);

  const plan = await requestShieldedSwap({
    program,
    feePayer: wallet.publicKey,
    vaultTokenA,
    vaultTokenB,
    userTokenA,
    userTokenB,
    proof: bundle,
  });
  const sigSettle = await provider.sendAndConfirm(plan.tx);
  const after = decodeVaultState(
    (await provider.connection.getAccountInfo(vault))!.data,
  );
  console.log(
    JSON.stringify(
      {
        proofDir,
        mint: new PublicKey(pub.assetIdMint).toBase58(),
        amount: pub.requestedSwapAmount.toString(),
        merkleRoot: Buffer.from(pub.merkleRoot).toString("hex"),
        cleanFundsRoot: cleanFundsRoot.toBase58(),
        sigRoot,
        sigSettle,
        reserveA: after.reserveA.toString(),
        reserveB: after.reserveB.toString(),
        explorerTx: `https://explorer.solana.com/tx/${sigSettle}?cluster=devnet`,
      },
      null,
      2,
    ),
  );
}

async function main(): Promise<void> {
  const step = process.argv[2];
  if (step === "config") {
    await initConfig();
    return;
  }
  if (step === "vault") {
    await initVault();
    return;
  }
  if (step === "mints") {
    await initMints();
    return;
  }
  if (step === "vkey") {
    await setVkey();
    return;
  }
  if (step === "deposit") {
    await depositBoth();
    return;
  }
  if (step === "settle") {
    await settleDevnet();
    return;
  }
  console.error(
    "usage: npx tsx scripts/devnet-init.ts config|vault|mints|vkey|deposit|settle",
  );
  process.exit(1);
}

main().catch((e: unknown) => {
  console.error(e);
  process.exit(1);
});
