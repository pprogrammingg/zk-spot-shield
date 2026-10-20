/**
 * Day 68: live Devnet reads (+ optional pause→unpause write).
 *
 * CI-default does not set DEVNET_ONCHAIN, so this file is skipped.
 *
 *   DEVNET_ONCHAIN=1 \
 *   ANCHOR_PROVIDER_URL=https://api.devnet.solana.com \
 *   npx vitest run --config vitest.config.ts test/devnet/onchain.devnet.test.ts
 *
 * Optional write (protocol admin key must match GlobalConfig.authority):
 *   DEVNET_WRITE=1 ANCHOR_WALLET=~/.config/solana/id.json
 */

import { describe, expect, it } from "vitest";
import { Connection, PublicKey } from "@solana/web3.js";
import {
  createLocalhostProvider,
  createProgram,
  decodeGlobalConfig,
  decodeVaultState,
  findGlobalConfigPda,
  findVaultPda,
  loadKeypairWallet,
  programIdFromIdl,
} from "../../src/index.js";

const run = process.env.DEVNET_ONCHAIN === "1";
const url =
  process.env.ANCHOR_PROVIDER_URL ?? "https://api.devnet.solana.com";

describe.skipIf(!run)("Day 68 Devnet on-chain", () => {
  it("fetches GlobalConfig + VaultState from Devnet", async () => {
    if (!url.includes("devnet")) {
      throw new Error(`ANCHOR_PROVIDER_URL must be a Devnet RPC, got ${url}`);
    }
    const connection = new Connection(url, "confirmed");
    const programId = process.env.DEVNET_PROGRAM_ID
      ? new PublicKey(process.env.DEVNET_PROGRAM_ID)
      : programIdFromIdl();
    const [globalConfig] = await findGlobalConfigPda(programId);
    const [vault] = await findVaultPda(programId);
    const cfgAcc = await connection.getAccountInfo(globalConfig);
    const vaultAcc = await connection.getAccountInfo(vault);
    if (!cfgAcc) {
      throw new Error(
        `GlobalConfig missing at ${globalConfig.toBase58()} — finish Days 58–60`,
      );
    }
    if (!vaultAcc) {
      throw new Error(`VaultState missing at ${vault.toBase58()} — finish Day 61`);
    }
    const cfg = decodeGlobalConfig(cfgAcc.data);
    const vaultState = decodeVaultState(vaultAcc.data);
    expect(cfg.vkeyHash.length).toBe(32);
    expect(cfg.authority.toBase58().length).toBeGreaterThan(30);
    expect(vaultState.bump).toBeGreaterThan(0);
    // eslint-disable-next-line no-console
    console.log(
      JSON.stringify({
        cluster: url,
        programId: programId.toBase58(),
        globalConfig: globalConfig.toBase58(),
        vault: vault.toBase58(),
        pauseFlag: cfg.pauseFlag,
        reserveA: vaultState.reserveA.toString(),
        reserveB: vaultState.reserveB.toString(),
      }),
    );
  });

  it("optional pause→unpause write lands on Devnet", async () => {
    if (process.env.DEVNET_WRITE !== "1") {
      return;
    }
    const wallet = loadKeypairWallet();
    const provider = createLocalhostProvider({
      wallet,
      url,
      setGlobal: false,
    });
    const program = createProgram(provider);
    const [globalConfig] = await findGlobalConfigPda(program.programId);
    const sigPause = await program.methods
      .pause()
      .accountsPartial({
        authority: wallet.publicKey,
        globalConfig,
      })
      .rpc();
    const sigUnpause = await program.methods
      .unpause()
      .accountsPartial({
        authority: wallet.publicKey,
        globalConfig,
      })
      .rpc();
    expect(sigPause.length).toBeGreaterThan(30);
    expect(sigUnpause.length).toBeGreaterThan(30);
    // eslint-disable-next-line no-console
    console.log(JSON.stringify({ sigPause, sigUnpause }));
  });
});
