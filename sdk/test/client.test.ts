import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { Keypair, PublicKey } from "@solana/web3.js";
import { Wallet } from "@coral-xyz/anchor";
import { describe, expect, it } from "vitest";
import { createLocalhostProvider } from "../src/provider.js";
import {
  DepositTree,
  createProgram,
  decodeVaultState,
  encodeSettleIntent,
  generateProofInputs,
  hexToBytes32,
  requestShieldedSwap,
  vaultStateDiscriminator,
} from "../src/index.js";

const root = join(dirname(fileURLToPath(import.meta.url)), "../..");
const happy = JSON.parse(
  readFileSync(join(root, "zk-circuit/fixtures/happy.json"), "utf8"),
) as {
  secret: string;
  user_address: string;
  balance: number;
  asset_id_mint: string;
  expected_root: string;
  requested_swap_amount: number;
};

describe("Day 49 SDK surface", () => {
  it("generateProofInputs matches happy fixture root", async () => {
    const tree = await DepositTree.create();
    const inputs = await generateProofInputs(
      {
        secret: hexToBytes32(happy.secret),
        userAddress: hexToBytes32(happy.user_address),
        balance: happy.balance,
        assetIdMint: hexToBytes32(happy.asset_id_mint),
      },
      tree,
      happy.requested_swap_amount,
    );
    expect(inputs.leafIndex).toBe(0);
    expect(inputs.requestedSwapAmount).toBe(100n);
    expect(Buffer.from(inputs.expectedRoot).toString("hex")).toBe(
      happy.expected_root.slice(2),
    );
    expect(inputs.merklePath).toHaveLength(20);
  });

  it("requestShieldedSwap allows relayer fee payer + distinct recipient ATA", async () => {
    const relayer = new Wallet(Keypair.generate());
    const recipientA = Keypair.generate().publicKey;
    const recipientB = Keypair.generate().publicKey;
    const depositorA = Keypair.generate().publicKey;
    const provider = createLocalhostProvider({
      wallet: relayer,
      setGlobal: false,
    });
    const program = createProgram(provider);
    const plan = await requestShieldedSwap({
      program,
      feePayer: relayer.publicKey,
      vaultTokenA: Keypair.generate().publicKey,
      vaultTokenB: Keypair.generate().publicKey,
      userTokenA: depositorA,
      userTokenB: Keypair.generate().publicKey,
      recipientTokenA: recipientA,
      recipientTokenB: recipientB,
    });
    expect(plan.feePayer.equals(relayer.publicKey)).toBe(true);
    expect(plan.accounts.payer.equals(relayer.publicKey)).toBe(true);
    expect(plan.accounts.userTokenA.equals(recipientA)).toBe(true);
    expect(plan.accounts.userTokenB.equals(recipientB)).toBe(true);
    expect(plan.accounts.userTokenA.equals(depositorA)).toBe(false);
    expect(plan.requestedSwapAmount).toBe(100n);
    expect(plan.nullifier.length).toBe(32);
    expect(plan.tx.instructions.length).toBe(2);

    const intent = encodeSettleIntent(plan);
    expect(intent.amount).toBe(100);
    expect(intent.user_token_a).toBe(recipientA.toBase58());
    expect(intent.proof_base64.length).toBeGreaterThan(0);
  });

  it("decodeVaultState reads zero-copy reserves", () => {
    const disc = vaultStateDiscriminator();
    const body = Buffer.alloc(120);
    const authority = Keypair.generate().publicKey;
    const mintA = new PublicKey(Buffer.alloc(32, 3));
    const mintB = new PublicKey(Buffer.alloc(32, 4));
    authority.toBuffer().copy(body, 0);
    mintA.toBuffer().copy(body, 32);
    mintB.toBuffer().copy(body, 64);
    body.writeBigUInt64LE(300n, 96);
    body.writeBigUInt64LE(900n, 104);
    body[112] = 255;
    const decoded = decodeVaultState(Buffer.concat([disc, body]));
    expect(decoded.reserveA).toBe(300n);
    expect(decoded.reserveB).toBe(900n);
    expect(decoded.mintA.equals(mintA)).toBe(true);
    expect(decoded.bump).toBe(255);
  });
});
