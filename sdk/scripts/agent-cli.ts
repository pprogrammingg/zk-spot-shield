#!/usr/bin/env npx tsx
/**
 * Localnet agent REPL. HELP: 1 Buy Token B · 2 Deposit Token A · 3 Balance · EXIT
 *
 *   cd sdk && npm run agent
 */

import * as readline from "node:readline/promises";
import { stdin as input, stdout as output } from "node:process";
import {
  AGENT_HELP,
  BACK_HINT,
  balanceUiUrl,
  discoverUiOrigin,
  isBackToMenu,
  openBrowser,
  parseAgentCommand,
  parseAmount,
  parseYesNo,
} from "../src/agentCmd.js";
import { decimalsForCluster, formatTokenAmount } from "../src/tokenAmount.js";
import {
  DEFAULT_RELAYER_URL,
  agentCluster,
  agentRpcUrl,
  buyMintB,
  depositMintA,
  loadAgentBundle,
  syncSpentFromChain,
  NULLIFIER_SYNC_TTL_MS,
  type AgentCtx,
} from "../src/agentLocalnet.js";
import { startAgentHttp, withAgentBuyGate } from "../src/agentHttp.js";
import {
  defaultStatePath,
  formatNotesTable,
  loadAgentState,
  pickSpendableNote,
  publicNotesSnapshot,
  publishUiNotes,
} from "../src/agentStore.js";

type ConfirmResult = "yes" | "no" | "back";

function amtDec(): number {
  return decimalsForCluster(agentCluster());
}

function fmtAmt(n: string | bigint | number): string {
  return formatTokenAmount(n, amtDec());
}

async function confirm(rl: readline.Interface, prompt: string): Promise<ConfirmResult> {
  const ans = await rl.question(`${prompt} [y/N] ${BACK_HINT}: `);
  if (isBackToMenu(ans)) return "back";
  return parseYesNo(ans) ? "yes" : "no";
}

async function readAmount(
  rl: readline.Interface,
  prompt: string,
  decimals: number,
): Promise<bigint | "back"> {
  const raw = await rl.question(`${prompt} ${BACK_HINT}: `);
  if (isBackToMenu(raw)) return "back";
  return parseAmount(raw, decimals);
}

async function cmdBalance(rl: readline.Interface, ctx: AgentCtx): Promise<void> {
  const state = await syncSpentFromChain(
    ctx,
    loadAgentState(ctx.statePath),
    (line) => console.log(line),
  );
  publishUiNotes(state);
  console.log(formatNotesTable(state, amtDec()));
  const ans = await rl.question(`open the swap UI? [Y/n] ${BACK_HINT}: `);
  if (isBackToMenu(ans)) return;
  const open = ans.trim() === "" || parseYesNo(ans);
  if (!open) return;
  const origin = await discoverUiOrigin();
  if (!origin) {
    console.log("start the UI first: cd ui && npm run dev");
    return;
  }
  const url = balanceUiUrl(origin, agentCluster());
  openBrowser(url);
  console.log(url);
}

async function cmdDeposit(rl: readline.Interface, ctx: AgentCtx): Promise<void> {
  let amount: bigint;
  try {
    const got = await readAmount(rl, "amount of mint A to deposit", amtDec());
    if (got === "back") return;
    amount = got;
  } catch (e) {
    console.error(e instanceof Error ? e.message : e);
    return;
  }
  const ok = await confirm(rl, `deposit ${fmtAmt(amount)} mint A into the vault?`);
  if (ok !== "yes") {
    if (ok === "no") console.log("cancelled");
    return;
  }
  console.log("==> deposit (no Groth16)");
  const { sig, note, rootSig } = await depositMintA(ctx, amount);
  console.log(`leaf ${note.id}…  idx ${note.leafIndex}  sig ${sig}`);
  console.log(`admin root ${rootSig === "exists" ? "already registered" : rootSig}`);
}

async function cmdBuy(rl: readline.Interface, ctx: AgentCtx): Promise<void> {
  let amount: bigint;
  try {
    const got = await readAmount(
      rl,
      "amount of mint A to spend (buy Token B 1:1)",
      amtDec(),
    );
    if (got === "back") return;
    amount = got;
  } catch (e) {
    console.error(e instanceof Error ? e.message : e);
    return;
  }
  const buyOk = await confirm(
    rl,
    `buy ${fmtAmt(amount)} mint B (1:1, 402 + Groth16)?`,
  );
  if (buyOk !== "yes") {
    if (buyOk === "no") console.log("cancelled");
    return;
  }

  const state = loadAgentState(ctx.statePath);
  const preview = pickSpendableNote(state, amount);
  if (!preview) {
    console.log(`no note covers ${fmtAmt(amount)}. Deposit more A (2).`);
    return;
  }
  console.log(
    `note ${preview.id} balance ${fmtAmt(preview.balance)} idx ${preview.leafIndex}`,
  );
  const spendOk = await confirm(rl, "spend this note?");
  if (spendOk !== "yes") {
    if (spendOk === "no") console.log("cancelled");
    return;
  }

  const result = await withAgentBuyGate(() =>
    buyMintB(ctx, amount, (line) => console.log(line)),
  );
  if (result.kind === "underfunded") {
    console.log(`no note covers quote ${fmtAmt(result.quote)}. Deposit more A (2).`);
    return;
  }
  console.log(`settle ${result.settleSig}`);
  console.log(
    `402 PAID  nullifier ${result.pay.nullifier_hex}  paid=${result.pay.paid}  (no wallet in the receipt)`,
  );
  if (result.change) {
    console.log(
      `change note ${result.change.id} balance ${fmtAmt(result.change.balance)} idx ${result.change.leafIndex}`,
    );
    console.log(
      `admin root ${result.rootSig === "exists" ? "already registered" : result.rootSig}`,
    );
  } else {
    console.log("spend-all (no change note)");
  }
}

async function main(): Promise<void> {
  if (process.argv.includes("--help") || process.argv.includes("-h")) {
    console.log(AGENT_HELP);
    return;
  }

  let bundle;
  try {
    bundle = loadAgentBundle();
  } catch (e) {
    console.error(e instanceof Error ? e.message : e);
    process.exit(1);
  }

  const cluster = agentCluster();
  const ctx: AgentCtx = {
    bundle,
    relayerUrl: process.env.RELAYER_URL ?? DEFAULT_RELAYER_URL,
    statePath: defaultStatePath(),
    rpcUrl: agentRpcUrl(),
  };

  console.log(AGENT_HELP);
  console.log(`cluster ${cluster}  rpc ${ctx.rpcUrl}`);
  if (cluster === "devnet") {
    console.log(`mint A ${ctx.bundle.mintA}  (${amtDec()} decimals; type 33 for 33 tokens)`);
  }
  publishUiNotes(
    await syncSpentFromChain(
      ctx,
      loadAgentState(ctx.statePath),
      (line) => console.log(line),
    ),
  );

  const http = await startAgentHttp({
    decimals: 0,
    buy: (amount) => buyMintB(ctx, amount, (line) => console.log(`[ui] ${line}`)),
    deposit: (amount) => {
      console.log(`[ui] deposit ${fmtAmt(amount)} (no Groth16)`);
      return depositMintA(ctx, amount);
    },
    status: () => {
      const state = loadAgentState(ctx.statePath);
      return {
        ok: true as const,
        cluster,
        programId: ctx.bundle.programId,
        vaultTokenA: ctx.bundle.vaultTokenA,
        vaultTokenB: ctx.bundle.vaultTokenB,
        userTokenA: ctx.bundle.userTokenA,
        userTokenB: ctx.bundle.userTokenB,
        mintA: ctx.bundle.mintA,
        mintB: ctx.bundle.mintB,
        decimals: amtDec(),
        notes: publicNotesSnapshot(state),
      };
    },
  });
  console.log(`agent http ${http.origin}  (UI deposit A / buy B)`);
  console.log(
    `chain sync: nullifiers at boot, before buy, and every ${NULLIFIER_SYNC_TTL_MS / 1000}s (not on UI poll)`,
  );

  const syncId = setInterval(() => {
    void syncSpentFromChain(
      ctx,
      loadAgentState(ctx.statePath),
      (line) => console.log(line),
    )
      .then(publishUiNotes)
      .catch((e) => console.error(e instanceof Error ? e.message : e));
  }, NULLIFIER_SYNC_TTL_MS);

  const rl = readline.createInterface({ input, output });
  try {
    for (;;) {
      const line = await rl.question("agent> ");
      const cmd = parseAgentCommand(line);
      if (cmd === null) {
        console.log(AGENT_HELP);
        continue;
      }
      try {
        if (cmd === "help") console.log(AGENT_HELP);
        else if (cmd === "exit") break;
        else if (cmd === "balance") await cmdBalance(rl, ctx);
        else if (cmd === "deposit") await cmdDeposit(rl, ctx);
        else if (cmd === "buy") await cmdBuy(rl, ctx);
      } catch (e) {
        console.error(e instanceof Error ? e.message : e);
      }
    }
  } finally {
    clearInterval(syncId);
    rl.close();
    await new Promise<void>((r) => http.server.close(() => r()));
  }
}

main().catch((e: unknown) => {
  console.error(e);
  process.exit(1);
});
