import { Keypair } from "@solana/web3.js";
import { Wallet } from "@coral-xyz/anchor";
import { describe, expect, it } from "vitest";
import { createLocalhostProvider } from "../src/provider.js";
import {
  DepositTree,
  createProgram,
  generateProofInputs,
  getSettleProof,
  hexToBytes32,
  requestShieldedSwap,
} from "../src/index.js";

describe("Day 51 latency probe", () => {
  it("times fixture prove + SDK swap plan", async () => {
    const t0 = performance.now();
    await getSettleProof();
    const fixtureMs = performance.now() - t0;

    const tree = await DepositTree.create();
    const t1 = performance.now();
    await generateProofInputs(
      {
        secret: hexToBytes32("0x" + "01".repeat(32)),
        userAddress: hexToBytes32("0x" + "02".repeat(32)),
        balance: 1000,
        assetIdMint: hexToBytes32("0x" + "03".repeat(32)),
      },
      tree,
      100,
    );
    const witnessMs = performance.now() - t1;

    const wallet = new Wallet(Keypair.generate());
    const program = createProgram(
      createLocalhostProvider({ wallet, setGlobal: false }),
    );
    const t2 = performance.now();
    await requestShieldedSwap({
      program,
      feePayer: wallet.publicKey,
      vaultTokenA: Keypair.generate().publicKey,
      vaultTokenB: Keypair.generate().publicKey,
      userTokenA: Keypair.generate().publicKey,
      userTokenB: Keypair.generate().publicKey,
    });
    const planMs = performance.now() - t2;

    console.log(
      `Day 51 latency fixture_ms=${fixtureMs.toFixed(1)} witness_ms=${witnessMs.toFixed(1)} plan_ms=${planMs.toFixed(1)}`,
    );
    expect(fixtureMs).toBeLessThan(500);
    expect(planMs).toBeLessThan(2_000);
  });
});
