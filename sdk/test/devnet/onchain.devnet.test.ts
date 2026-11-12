/**
 * Live Devnet E2E (reads + flows + negatives).
 *
 * CI skips unless `DEVNET_ONCHAIN=1`. Passed cases are recorded in
 * `sdk/.devnet-e2e-done.json` and skipped later unless forced.
 *
 * See `notes/devnet.md` (Day 68) for the case table and commands.
 */

import { describe, expect, it } from "vitest";
import { TOKEN_PROGRAM_ID, getAccount } from "@solana/spl-token";
import { Transaction } from "@solana/web3.js";
import BN from "bn.js";
import {
  buildPublicSpotSwapIx,
  confirmSignature,
} from "../../src/index.js";
import {
  PIN,
  bytesToHex,
  ensureAta,
  ensureSol,
  expectFail,
  markDone,
  mintAndDeposit,
  mintVaultToken,
  openDevnetCtx,
  readConfig,
  readVault,
  shouldRunCase,
  vaultAtas,
  type DevnetCaseId,
} from "./helpers.js";

const on = process.env.DEVNET_ONCHAIN === "1";
/** Public Devnet RPC 429s; each case may need many confirms. */
const CASE_MS = 120_000;

function sleep(ms: number): Promise<void> {
  return new Promise((r) => setTimeout(r, ms));
}

function caseIt(id: DevnetCaseId, title: string, fn: () => Promise<void>) {
  it.skipIf(!shouldRunCase(id))(
    `${id}: ${title}`,
    async () => {
      await sleep(1_500); // soften burst rate-limits between cases
      let ok = false;
      try {
        await fn();
        ok = true;
      } finally {
        // Only persist after a clean finish (not a late completion after vitest timeout).
        if (ok) markDone(id);
      }
    },
    CASE_MS,
  );
}

describe.skipIf(!on)("Devnet on-chain E2E", () => {
  caseIt("reads", "GlobalConfig + VaultState + ATAs match pins", async () => {
    const ctx = await openDevnetCtx();
    const cfg = await readConfig(ctx);
    const vault = await readVault(ctx);
    expect(bytesToHex(cfg.vkeyHash)).toBe(PIN.vkeyHex);
    expect(cfg.pauseFlag).toBe(false);
    expect(vault.mintA.toBase58()).toBe(PIN.mintA);
    expect(vault.mintB.toBase58()).toBe(PIN.mintB);
    expect(vault.bump).toBeGreaterThan(0);
    const atas = await vaultAtas(vault, ctx.vault);
    const ataA = await getAccount(ctx.connection, atas.a);
    const ataB = await getAccount(ctx.connection, atas.b);
    expect(ataA.amount).toBe(vault.reserveA);
    expect(ataB.amount).toBe(vault.reserveB);
    // eslint-disable-next-line no-console
    console.log(
      JSON.stringify({
        globalConfig: ctx.globalConfig.toBase58(),
        vault: ctx.vault.toBase58(),
        reserveA: vault.reserveA.toString(),
        reserveB: vault.reserveB.toString(),
      }),
    );
  });

  caseIt("pause", "pause→unpause write lands", async () => {
    const ctx = await openDevnetCtx();
    await ensureSol(ctx.connection, ctx.payer);
    const before = await readConfig(ctx);
    expect(before.authority.equals(ctx.payer)).toBe(true);
    expect(before.pauseFlag).toBe(false);

    const sigPause = await ctx.program.methods
      .pause()
      .accountsPartial({
        authority: ctx.payer,
        globalConfig: ctx.globalConfig,
      })
      .rpc();
    expect((await readConfig(ctx)).pauseFlag).toBe(true);

    const sigUnpause = await ctx.program.methods
      .unpause()
      .accountsPartial({
        authority: ctx.payer,
        globalConfig: ctx.globalConfig,
      })
      .rpc();
    expect((await readConfig(ctx)).pauseFlag).toBe(false);
    // eslint-disable-next-line no-console
    console.log(JSON.stringify({ sigPause, sigUnpause }));
  });

  caseIt("deposit_a", "mint + deposit Token A", async () => {
    const ctx = await openDevnetCtx();
    const amount = 100_000n; // 0.1 A
    const before = await readVault(ctx);
    const out = await mintAndDeposit(ctx, "a", amount);
    expect(BigInt(out.reserveA)).toBe(before.reserveA + amount);
    // eslint-disable-next-line no-console
    console.log(JSON.stringify(out));
  });

  caseIt("deposit_b", "mint + deposit Token B (vault liquidity)", async () => {
    const ctx = await openDevnetCtx();
    const amount = 100_000n; // 0.1 B
    const before = await readVault(ctx);
    const out = await mintAndDeposit(ctx, "b", amount);
    expect(BigInt(out.reserveB)).toBe(before.reserveB + amount);
    // eslint-disable-next-line no-console
    console.log(JSON.stringify(out));
  });

  caseIt("public_swap", "public_spot_swap A→B 1:1", async () => {
    const ctx = await openDevnetCtx();
    await ensureSol(ctx.connection, ctx.payer);
    const amount = 10_000n; // 0.01
    const vault = await readVault(ctx);
    expect(vault.reserveB).toBeGreaterThanOrEqual(amount);
    const send = (tx: Transaction) => ctx.provider.sendAndConfirm(tx);
    const userA = await ensureAta(
      ctx.connection,
      ctx.payer,
      vault.mintA,
      ctx.payer,
      send,
    );
    const userB = await ensureAta(
      ctx.connection,
      ctx.payer,
      vault.mintB,
      ctx.payer,
      send,
    );
    await mintVaultToken(ctx, vault.mintA, userA, amount);
    const atas = await vaultAtas(vault, ctx.vault);
    const userBBefore = (await getAccount(ctx.connection, userB)).amount;
    const ix = await buildPublicSpotSwapIx(
      ctx.program,
      {
        payer: ctx.payer,
        globalConfig: ctx.globalConfig,
        vault: ctx.vault,
        vaultTokenA: atas.a,
        vaultTokenB: atas.b,
        userTokenA: userA,
        userTokenB: userB,
      },
      amount,
    );
    const sig = await ctx.provider.sendAndConfirm(new Transaction().add(ix));
    await confirmSignature(ctx.connection, sig);
    const after = await readVault(ctx);
    expect(after.reserveA).toBe(vault.reserveA + amount);
    expect(after.reserveB).toBe(vault.reserveB - amount);
    const userBAfter = (await getAccount(ctx.connection, userB)).amount;
    expect(userBAfter).toBe(userBBefore + amount);
    // eslint-disable-next-line no-console
    console.log(JSON.stringify({ sig, amount: amount.toString() }));
  });

  caseIt("neg_zero_swap", "public_spot_swap rejects amount 0", async () => {
    const ctx = await openDevnetCtx();
    const vault = await readVault(ctx);
    const send = (tx: Transaction) => ctx.provider.sendAndConfirm(tx);
    const userA = await ensureAta(
      ctx.connection,
      ctx.payer,
      vault.mintA,
      ctx.payer,
      send,
    );
    const userB = await ensureAta(
      ctx.connection,
      ctx.payer,
      vault.mintB,
      ctx.payer,
      send,
    );
    const atas = await vaultAtas(vault, ctx.vault);
    await expectFail(
      () =>
        ctx.program.methods
          .publicSpotSwap(new BN(0))
          .accountsPartial({
            payer: ctx.payer,
            globalConfig: ctx.globalConfig,
            vault: ctx.vault,
            vaultTokenA: atas.a,
            vaultTokenB: atas.b,
            userTokenA: userA,
            userTokenB: userB,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .rpc(),
      /ZeroSwapAmount|0x177a|6010/i,
    );
  });

  caseIt("neg_underflow", "public_spot_swap rejects when vault B empty for quote", async () => {
    const ctx = await openDevnetCtx();
    await ensureSol(ctx.connection, ctx.payer);
    const vault = await readVault(ctx);
    // apply_swap fails before SPL transfer — no need to mint `tooMuch` A.
    const tooMuch = vault.reserveB + 1n;
    const send = (tx: Transaction) => ctx.provider.sendAndConfirm(tx);
    const userA = await ensureAta(
      ctx.connection,
      ctx.payer,
      vault.mintA,
      ctx.payer,
      send,
    );
    const userB = await ensureAta(
      ctx.connection,
      ctx.payer,
      vault.mintB,
      ctx.payer,
      send,
    );
    const atas = await vaultAtas(vault, ctx.vault);
    await expectFail(
      () =>
        ctx.program.methods
          .publicSpotSwap(new BN(tooMuch.toString()))
          .accountsPartial({
            payer: ctx.payer,
            globalConfig: ctx.globalConfig,
            vault: ctx.vault,
            vaultTokenA: atas.a,
            vaultTokenB: atas.b,
            userTokenA: userA,
            userTokenB: userB,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .rpc(),
      /VaultReserveUnderflow|0x177b|6011/i,
    );
  });

  caseIt("neg_paused_swap", "public_spot_swap rejects while paused", async () => {
    const ctx = await openDevnetCtx();
    await ensureSol(ctx.connection, ctx.payer);
    const vault = await readVault(ctx);
    const amount = 1_000n;
    const send = (tx: Transaction) => ctx.provider.sendAndConfirm(tx);
    const userA = await ensureAta(
      ctx.connection,
      ctx.payer,
      vault.mintA,
      ctx.payer,
      send,
    );
    const userB = await ensureAta(
      ctx.connection,
      ctx.payer,
      vault.mintB,
      ctx.payer,
      send,
    );
    await mintVaultToken(ctx, vault.mintA, userA, amount);
    const atas = await vaultAtas(vault, ctx.vault);

    await ctx.program.methods
      .pause()
      .accountsPartial({
        authority: ctx.payer,
        globalConfig: ctx.globalConfig,
      })
      .rpc();
    try {
      await expectFail(
        () =>
          ctx.program.methods
            .publicSpotSwap(new BN(amount.toString()))
            .accountsPartial({
              payer: ctx.payer,
              globalConfig: ctx.globalConfig,
              vault: ctx.vault,
              vaultTokenA: atas.a,
              vaultTokenB: atas.b,
              userTokenA: userA,
              userTokenB: userB,
              tokenProgram: TOKEN_PROGRAM_ID,
            })
            .rpc(),
        /ProtocolPaused|0x177c|6012/i,
      );
    } finally {
      await ctx.program.methods
        .unpause()
        .accountsPartial({
          authority: ctx.payer,
          globalConfig: ctx.globalConfig,
        })
        .rpc();
      expect((await readConfig(ctx)).pauseFlag).toBe(false);
    }
  });
});

describe.skipIf(!on)("Devnet E2E — not automated here", () => {
  it.skip("shielded settle / buy (needs network Groth16 + $PROVE)", () => {
    // Agent CLI `1` or UI Buy Token B. Record sigs in notes/devnet.md.
  });
});
