/**
 * Localnet agent actions: deposit A, buy B (402 → prove → settle → root), balances.
 * Token accounts come from demo:localnet (`ui/public/demo/accounts.json`).
 */

import { existsSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import {
  createAssociatedTokenAccountInstruction,
  getAssociatedTokenAddressSync,
  mintTo,
} from "@solana/spl-token";
import { PublicKey, SystemProgram, Transaction } from "@solana/web3.js";
import { randomBytes } from "node:crypto";
import {
  DepositTree,
  proofInputsForExistingNote,
} from "./depositTree.js";
import { bytes32ToHex, computeLeaf, computeNullifier, hexToBytes32 } from "./merkle.js";
import {
  buildDepositIx,
  confirmSignature,
  createProgram,
  findCleanFundsRootPda,
  findDepositLeafPda,
  findGlobalConfigPda,
  findNullifierPda,
  findVaultPda,
} from "./instructions.js";
import {
  getSettleProof,
  parseJournalPublic,
  REPO_ROOT,
} from "./prove.js";
import { requestShieldedSwap, getVaultReserves } from "./client.js";
import {
  DEVNET_URL,
  LOCALNET_URL,
  createLocalhostProvider,
  loadKeypairWallet,
} from "./provider.js";
import { journalToPayBody, payUrl, quoteUrl, type PaymentQuote } from "./pay402.js";
import {
  appendLeafHex,
  formatNotesTable,
  loadAgentState,
  markSpent,
  noteFromParts,
  pickSpendableNote,
  saveAgentState,
  storedToDepositNote,
  type AgentState,
  type StoredNote,
} from "./agentStore.js";
import { BuyTimer, formatClock, formatDuration } from "./agentTiming.js";
import { buildWitnessFile, writeWitnessJson } from "./agentWitness.js";
import { decimalsForCluster, formatTokenAmount } from "./tokenAmount.js";

const HERE = dirname(fileURLToPath(import.meta.url));
const SDK_ROOT = join(HERE, "..");

export const DEFAULT_RELAYER_URL = "http://127.0.0.1:8787";

/** Live Devnet vault from `notes/devnet.md`. User ATAs are derived from the CLI wallet. */
export const DEVNET_FIXED = {
  programId: "74UW993wrDasAMyBQ53PjHpYLK5vMkWbRRWFZ7kCmM3G",
  authority: "9sLGgVEotwD8gJgL5HhJdP7w61zeCrJ25NNkJPM3BAaX",
  globalConfig: "BQbrv7CVnh571XjvGv4SMfo5beUcrgF3wtsHrzWAU3CY",
  vault: "A3h8dhmMipthngLSS5AmhsxYvt9sbgLmuUf1wfpJDqmd",
  mintA: "DNNkZidPWCHxoyuaQ3h5N8FUCWaJZddEYvF7JyHHKbVE",
  mintB: "HfgizYEd9aVxk6hAXjEtoac1jcVHA7KyrqyJYdm5wqzg",
  vaultTokenA: "EVmRfTNFecvTBtW8Zcv5BrQJwZTA5HbSJMVBUJTEdTGQ",
  vaultTokenB: "8wBhwWC9c52D2qgSzYxNsCEN2YMJgsM916c8kBFrMTso",
};

export function agentCluster(): "localnet" | "devnet" {
  const c = (process.env.AGENT_CLUSTER ?? "localnet").toLowerCase();
  return c === "devnet" ? "devnet" : "localnet";
}

export function agentRpcUrl(): string {
  if (process.env.ANCHOR_PROVIDER_URL) return process.env.ANCHOR_PROVIDER_URL;
  return agentCluster() === "devnet" ? DEVNET_URL : LOCALNET_URL;
}

export function loadDevnetBundle(): LocalnetBundle {
  const wallet = loadKeypairWallet();
  const mintA = new PublicKey(DEVNET_FIXED.mintA);
  const mintB = new PublicKey(DEVNET_FIXED.mintB);
  return {
    ...DEVNET_FIXED,
    userTokenA: getAssociatedTokenAddressSync(mintA, wallet.publicKey).toBase58(),
    userTokenB: getAssociatedTokenAddressSync(mintB, wallet.publicKey).toBase58(),
  };
}

export function loadAgentBundle(): LocalnetBundle {
  return agentCluster() === "devnet" ? loadDevnetBundle() : loadLocalnetBundle();
}

export type LocalnetBundle = {
  programId: string;
  authority: string;
  globalConfig: string;
  vault: string;
  mintA: string;
  mintB: string;
  vaultTokenA: string;
  vaultTokenB: string;
  userTokenA: string;
  userTokenB: string;
};

export function loadLocalnetBundle(): LocalnetBundle {
  const candidates = [
    process.env.AGENT_ACCOUNTS,
    join(SDK_ROOT, ".e2e-accounts/bundle.json"),
    join(REPO_ROOT, "ui/public/demo/accounts.json"),
  ].filter((p): p is string => !!p);
  for (const p of candidates) {
    if (existsSync(p)) {
      return JSON.parse(readFileSync(p, "utf8")) as LocalnetBundle;
    }
  }
  throw new Error(
    "no localnet accounts.json — start the demo validator: cd sdk && npm run demo:localnet",
  );
}

export async function treeFromState(state: AgentState): Promise<DepositTree> {
  const leaves = state.leavesHex.map((h) => {
    const hex = h.replace(/^0x/, "");
    return Uint8Array.from(Buffer.from(hex, "hex"));
  });
  return DepositTree.fromLeaves(leaves);
}

export async function registerRoot(
  program: ReturnType<typeof createProgram>,
  authority: PublicKey,
  root: Uint8Array,
): Promise<string | "exists"> {
  const [globalConfig] = await findGlobalConfigPda(program.programId);
  const [cleanFundsRoot] = await findCleanFundsRootPda(program.programId, root);
  const info = await program.provider.connection.getAccountInfo(cleanFundsRoot);
  if (info) return "exists";
  return program.methods
    .registerCleanFundsRoot([...Buffer.from(root)])
    .accountsPartial({
      authority,
      globalConfig,
      cleanFundsRoot,
      systemProgram: SystemProgram.programId,
    })
    .rpc();
}

export async function fetch402Quote(
  relayerUrl: string,
  amount: bigint,
  mintHex: string,
): Promise<PaymentQuote> {
  const url = quoteUrl(relayerUrl, { amount: amount.toString(), mintHex });
  let res: Response;
  try {
    res = await fetch(url);
  } catch {
    throw new Error(
      `relayer unreachable at ${relayerUrl}. Start: cargo run -p zk-spot-shield-relayer`,
    );
  }
  const body = (await res.json()) as PaymentQuote;
  if (res.status !== 402) {
    throw new Error(
      `expected HTTP 402 from ${url}, got ${res.status}. Start: cargo run -p zk-spot-shield-relayer`,
    );
  }
  return body;
}

export async function post402Pay(
  relayerUrl: string,
  journal: Buffer,
): Promise<{ paid: boolean; nullifier_hex: string; amount: number }> {
  const res = await fetch(payUrl(relayerUrl), {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify(journalToPayBody(journal)),
  });
  const text = await res.text();
  if (!res.ok) {
    throw new Error(`POST /402/pay ${res.status}: ${text}`);
  }
  return JSON.parse(text) as {
    paid: boolean;
    nullifier_hex: string;
    amount: number;
  };
}

export type AgentCtx = {
  bundle: LocalnetBundle;
  relayerUrl: string;
  statePath: string;
  rpcUrl: string;
};

export async function readBalances(ctx: AgentCtx): Promise<{
  notes: string;
  tokenA: string;
  tokenB: string;
  vaultA: string;
  vaultB: string;
  wallet: string;
}> {
  const provider = createLocalhostProvider({
    setGlobal: false,
    url: ctx.rpcUrl,
  });
  const wallet = provider.wallet.publicKey.toBase58();
  const state = loadAgentState(ctx.statePath);
  const a = await provider.connection.getTokenAccountBalance(
    new PublicKey(ctx.bundle.userTokenA),
  );
  const b = await provider.connection.getTokenAccountBalance(
    new PublicKey(ctx.bundle.userTokenB),
  );
  const [vault] = await findVaultPda(
    new PublicKey(ctx.bundle.programId),
  );
  const reserves = await getVaultReserves(provider.connection, vault);
  return {
    notes: formatNotesTable(state, decimalsForCluster(agentCluster())),
    tokenA: a.value.amount,
    tokenB: b.value.amount,
    vaultA: reserves.reserveA.toString(),
    vaultB: reserves.reserveB.toString(),
    wallet,
  };
}

export async function depositMintA(
  ctx: AgentCtx,
  amount: bigint,
): Promise<{ sig: string; note: StoredNote; rootSig: string | "exists" }> {
  const provider = createLocalhostProvider({
    setGlobal: false,
    url: ctx.rpcUrl,
  });
  const program = createProgram(provider);
  const wallet = loadKeypairWallet();
  const mintA = new PublicKey(ctx.bundle.mintA);
  const secret = randomBytes(32);
  const userAddress = Uint8Array.from(wallet.publicKey.toBytes());
  const assetIdMint = Uint8Array.from(mintA.toBytes());
  const leaf = await computeLeaf(secret, userAddress, amount);
  const [vault] = await findVaultPda(program.programId);
  const [depositLeaf] = await findDepositLeafPda(program.programId, leaf);
  const ix = await buildDepositIx(
    program,
    {
      depositor: wallet.publicKey,
      vault,
      depositLeaf,
      vaultToken: new PublicKey(ctx.bundle.vaultTokenA),
      userToken: new PublicKey(ctx.bundle.userTokenA),
    },
    leaf,
    amount,
  );
  const tx = new Transaction().add(ix);
  const sig = await provider.sendAndConfirm(tx);
  await confirmSignature(provider.connection, sig);

  const state = loadAgentState(ctx.statePath);
  const tree = await treeFromState(state);
  const leafIndex = tree.insert(leaf);
  const note = noteFromParts({
    secret,
    userAddress,
    balance: amount,
    assetIdMint,
    leaf,
    leafIndex,
  });
  state.notes.push(note);
  appendLeafHex(state, bytes32ToHex(leaf));
  saveAgentState(state, ctx.statePath);

  const root = await tree.root();
  const rootSig = await registerRoot(program, wallet.publicKey, root);
  return { sig, note, rootSig };
}

async function ensureUserAta(
  connection: ReturnType<typeof createLocalhostProvider>["connection"],
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

/** Admin: mint Token A into the wallet ATA (no vault deposit). Wallet must be mint authority. */
export async function mintAToUser(
  ctx: AgentCtx,
  amount: bigint,
): Promise<{ sig: string; amount: string; userTokenA: string }> {
  if (amount <= 0n) throw new Error("amount must be positive");
  const provider = createLocalhostProvider({
    setGlobal: false,
    url: ctx.rpcUrl,
  });
  const wallet = loadKeypairWallet();
  const mintA = new PublicKey(ctx.bundle.mintA);
  const send = (tx: Transaction) => provider.sendAndConfirm(tx);
  const userAta = await ensureUserAta(
    provider.connection,
    wallet.publicKey,
    mintA,
    wallet.publicKey,
    send,
  );
  const payerKp = (
    wallet as typeof wallet & { payer: import("@solana/web3.js").Keypair }
  ).payer;
  const sig = await mintTo(
    provider.connection,
    payerKp,
    mintA,
    userAta,
    wallet.publicKey,
    amount,
  );
  await confirmSignature(provider.connection, sig);
  return {
    sig,
    amount: amount.toString(),
    userTokenA: userAta.toBase58(),
  };
}

/**
 * Admin: mint Token B + `deposit_shielded` into the vault (liquidity for settle).
 * Does not add a spendable agent note (buy only spends A).
 */
export async function depositMintB(
  ctx: AgentCtx,
  amount: bigint,
): Promise<{ sig: string; amount: string; reserveB: string }> {
  if (amount <= 0n) throw new Error("amount must be positive");
  const provider = createLocalhostProvider({
    setGlobal: false,
    url: ctx.rpcUrl,
  });
  const program = createProgram(provider);
  const wallet = loadKeypairWallet();
  const mintB = new PublicKey(ctx.bundle.mintB);
  const send = (tx: Transaction) => provider.sendAndConfirm(tx);
  const userAta = await ensureUserAta(
    provider.connection,
    wallet.publicKey,
    mintB,
    wallet.publicKey,
    send,
  );
  const payerKp = (
    wallet as typeof wallet & { payer: import("@solana/web3.js").Keypair }
  ).payer;
  await mintTo(
    provider.connection,
    payerKp,
    mintB,
    userAta,
    wallet.publicKey,
    amount,
  );

  const secret = randomBytes(32);
  const userAddress = Uint8Array.from(wallet.publicKey.toBytes());
  const leaf = await computeLeaf(secret, userAddress, amount);
  const [vault] = await findVaultPda(program.programId);
  const [depositLeaf] = await findDepositLeafPda(program.programId, leaf);
  const ix = await buildDepositIx(
    program,
    {
      depositor: wallet.publicKey,
      vault,
      depositLeaf,
      vaultToken: new PublicKey(ctx.bundle.vaultTokenB),
      userToken: userAta,
    },
    leaf,
    amount,
  );
  const sig = await provider.sendAndConfirm(new Transaction().add(ix));
  await confirmSignature(provider.connection, sig);
  const reserves = await getVaultReserves(provider.connection, vault);
  return {
    sig,
    amount: amount.toString(),
    reserveB: reserves.reserveB.toString(),
  };
}

export type BuyResult =
  | { kind: "underfunded"; quote: bigint }
  | {
      kind: "ok";
      quote: PaymentQuote;
      note: StoredNote;
      settleSig: string;
      pay: { paid: boolean; nullifier_hex: string; amount: number };
      change?: StoredNote;
      rootSig: string | "exists" | "spend-all";
    };

/** Batched nullifier PDA check. Not on UI poll. Boot + buy + this TTL. */
export const NULLIFIER_SYNC_TTL_MS = 60_000;
let lastNullifierSyncMs = 0;

/** Local `spent: false` but the nullifier PDA already exists — do not prove again. */
export async function syncSpentFromChain(
  ctx: AgentCtx,
  state: AgentState,
  log: (line: string) => void,
  opts?: { force?: boolean },
): Promise<AgentState> {
  const now = Date.now();
  if (!opts?.force && now - lastNullifierSyncMs < NULLIFIER_SYNC_TTL_MS) {
    return state;
  }
  const open = state.notes.filter((n) => !n.spent);
  if (open.length === 0) {
    lastNullifierSyncMs = now;
    return state;
  }
  const provider = createLocalhostProvider({
    setGlobal: false,
    url: ctx.rpcUrl,
  });
  const programId = new PublicKey(ctx.bundle.programId);
  const pdas: PublicKey[] = [];
  for (const n of open) {
    const nf = await computeNullifier(
      hexToBytes32(n.secretHex),
      hexToBytes32(n.leafHex),
      hexToBytes32(n.assetIdMintHex),
    );
    const [pda] = await findNullifierPda(programId, nf);
    pdas.push(pda);
  }
  const infos = await provider.connection.getMultipleAccountsInfo(pdas);
  let dirty = false;
  for (let i = 0; i < open.length; i++) {
    if (!infos[i]) continue;
    const n = open[i]!;
    n.spent = true;
    dirty = true;
    log(
      `note ${n.id} idx ${n.leafIndex} already spent on-chain (nullifier) — skipping`,
    );
  }
  lastNullifierSyncMs = Date.now();
  if (dirty) saveAgentState(state, ctx.statePath);
  return state;
}

export async function buyMintB(
  ctx: AgentCtx,
  amount: bigint,
  log: (line: string) => void,
): Promise<BuyResult> {
  const timer = new BuyTimer();
  let state = await syncSpentFromChain(
    ctx,
    loadAgentState(ctx.statePath),
    log,
    { force: true },
  );
  if (!pickSpendableNote(state, amount)) {
    return { kind: "underfunded", quote: amount };
  }
  const dec = decimalsForCluster(agentCluster());
  log(`buy ${formatTokenAmount(amount, dec)} B  started ${formatClock(timer.startedAt)}`);

  const mintHex = bytes32ToHex(Uint8Array.from(new PublicKey(ctx.bundle.mintA).toBytes()));
  const quote = await fetch402Quote(ctx.relayerUrl, amount, mintHex);
  const quoteMint = quote.mint_hex.replace(/^0x/i, "").toLowerCase();
  const wantMint = mintHex.replace(/^0x/i, "").toLowerCase();
  if (quoteMint !== wantMint) {
    throw new Error(
      `402 mint ${quote.mint_hex} ≠ vault mint A ${mintHex}`,
    );
  }
  const quoteAmt = BigInt(quote.amount);
  state = await syncSpentFromChain(
    ctx,
    loadAgentState(ctx.statePath),
    log,
    { force: true },
  );
  const picked = pickSpendableNote(state, quoteAmt);
  timer.mark("402 quote");
  if (!picked) {
    return { kind: "underfunded", quote: quoteAmt };
  }

  log(
    `402 PAYMENT REQUIRED  ${formatTokenAmount(quote.amount, dec)} mint ${quote.mint_hex.slice(0, 12)}…  unlock=${quote.unlock ?? "nullifier"}`,
  );
  log(
    `402 note ${picked.id} balance ${formatTokenAmount(picked.balance, dec)}  (not a wallet transfer)`,
  );

  const tree = await treeFromState(state);
  const depositNote = storedToDepositNote(picked);
  const packed = await proofInputsForExistingNote(
    depositNote,
    tree,
    quoteAmt,
    picked.leafIndex,
  );
  const artifactDir = join(REPO_ROOT, "sp1-artifacts", "agent-cli");
  const witnessPath = join(artifactDir, "witness.json");
  writeWitnessJson(
    witnessPath,
    buildWitnessFile({
      note: depositNote,
      tree,
      leafIndex: packed.leafIndex,
      requestedSwapAmount: quoteAmt,
      leaf: packed.leaf,
    }),
  );
  timer.mark("witness");

  log("==> Groth16 (~25s, $PROVE). This wait is for buy, not deposit.");
  const bundle = await getSettleProof({
    invokeHost: true,
    witnessJson: witnessPath,
    artifactDir,
    repoRoot: REPO_ROOT,
  });
  const grothMs = timer.mark("Groth16");
  log(`==> Groth16 done in ${formatDuration(grothMs)}`);

  const pub = parseJournalPublic(bundle.journal);
  if (pub.requestedSwapAmount !== quoteAmt) {
    throw new Error(
      `journal amount ${pub.requestedSwapAmount} ≠ 402 quote ${quoteAmt}`,
    );
  }

  const provider = createLocalhostProvider({
    setGlobal: false,
    url: ctx.rpcUrl,
  });
  const program = createProgram(provider);
  const wallet = loadKeypairWallet();
  log("==> settle");
  let settleSig: string;
  try {
    const plan = await requestShieldedSwap({
      program,
      feePayer: wallet.publicKey,
      vaultTokenA: new PublicKey(ctx.bundle.vaultTokenA),
      vaultTokenB: new PublicKey(ctx.bundle.vaultTokenB),
      userTokenA: new PublicKey(ctx.bundle.userTokenA),
      userTokenB: new PublicKey(ctx.bundle.userTokenB),
      proof: bundle,
    });
    settleSig = await provider.sendAndConfirm(plan.tx);
  } catch (e) {
    log(`settle error: ${e instanceof Error ? e.message : e}`);
    throw e;
  }
  await confirmSignature(provider.connection, settleSig);
  timer.mark("settle");
  log(timer.walletLines());

  markSpent(state, picked.leafIndex);
  let change: StoredNote | undefined;
  let rootSig: string | "exists" | "spend-all" = "spend-all";
  if (pub.changeAmount > 0n) {
    const changeLeaf = await computeLeaf(
      depositNote.secret,
      depositNote.userAddress,
      pub.changeAmount,
    );
    if (!Buffer.from(changeLeaf).equals(Buffer.from(pub.changeLeaf))) {
      throw new Error("journal change_leaf ≠ Poseidon(secret, user, remainder)");
    }
    const changeIndex = tree.insert(changeLeaf);
    change = noteFromParts({
      secret: depositNote.secret,
      userAddress: depositNote.userAddress,
      balance: pub.changeAmount,
      assetIdMint: depositNote.assetIdMint,
      leaf: changeLeaf,
      leafIndex: changeIndex,
    });
    state.notes.push(change);
    appendLeafHex(state, bytes32ToHex(changeLeaf));
    saveAgentState(state, ctx.statePath);
    const root = await tree.root();
    rootSig = await registerRoot(program, wallet.publicKey, root);
  } else {
    saveAgentState(state, ctx.statePath);
  }

  const pay = await post402Pay(ctx.relayerUrl, bundle.journal);

  return {
    kind: "ok",
    quote,
    note: picked,
    settleSig,
    pay,
    change,
    rootSig,
  };
}
