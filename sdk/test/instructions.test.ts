import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { Wallet } from "@coral-xyz/anchor";
import { describe, expect, it } from "vitest";
import { createLocalhostProvider } from "../src/provider.js";
import {
  SETTLE_ALT_STATIC_LABELS,
  SETTLE_CU_LIMIT,
  buildSettleTransaction,
  createProgram,
  findCleanFundsRootPda,
  findGlobalConfigPda,
  findNullifierPda,
  findVaultPda,
  loadIdl,
  programIdFromIdl,
  settleAltStaticAddresses,
  settleComputeBudgetIxs,
} from "../src/instructions.js";

describe("Day 35 instruction wrappers", () => {
  it("compute budget ix uses settle CU limit", () => {
    const ixs = settleComputeBudgetIxs();
    expect(ixs).toHaveLength(1);
    expect(SETTLE_CU_LIMIT).toBe(1_400_000);
  });

  it("ALT static labels are 6 keys", () => {
    expect(SETTLE_ALT_STATIC_LABELS).toHaveLength(6);
  });

  it("PDA helpers are stable", async () => {
    const programId = programIdFromIdl(loadIdl());
    const [g1] = await findGlobalConfigPda(programId);
    const [g2] = await findGlobalConfigPda(programId);
    expect(g1.equals(g2)).toBe(true);
    const [v] = await findVaultPda(programId);
    expect(v).toBeInstanceOf(PublicKey);
    const root = Buffer.alloc(32, 9);
    const [c] = await findCleanFundsRootPda(programId, root);
    const [n] = await findNullifierPda(programId, Buffer.alloc(32, 1));
    expect(c.equals(n)).toBe(false);
  });

  it("buildSettleTransaction prefixes CU limit", async () => {
    const wallet = new Wallet(Keypair.generate());
    const provider = createLocalhostProvider({ wallet, setGlobal: false });
    const program = createProgram(provider);
    const programId = program.programId;
    const [globalConfig] = await findGlobalConfigPda(programId);
    const [vault] = await findVaultPda(programId);
    const accounts = {
      payer: wallet.publicKey,
      globalConfig,
      vault,
      cleanFundsRoot: Keypair.generate().publicKey,
      nullifierAccount: Keypair.generate().publicKey,
      vaultTokenA: Keypair.generate().publicKey,
      vaultTokenB: Keypair.generate().publicKey,
      userTokenA: Keypair.generate().publicKey,
      userTokenB: Keypair.generate().publicKey,
    };
    const tx = await buildSettleTransaction(
      program,
      accounts,
      Buffer.alloc(356, 1),
      Buffer.alloc(104, 2),
    );
    expect(tx.instructions.length).toBe(2);
    const alt = settleAltStaticAddresses(accounts);
    expect(alt[0].equals(TOKEN_PROGRAM_ID)).toBe(true);
    expect(alt[1].equals(SystemProgram.programId)).toBe(true);
  });
});
