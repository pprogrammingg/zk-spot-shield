import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { Wallet } from "@coral-xyz/anchor";
import { describe, expect, it } from "vitest";
import { createLocalhostProvider } from "../src/provider.js";
import {
  SETTLE_ALT_STATIC_LABELS,
  SETTLE_CU_LIMIT,
  buildSettleIx,
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

describe("instruction wrappers", () => {
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

  it("change leaf sits before Token program; spend-all uses program-id sentinel", async () => {
    const wallet = new Wallet(Keypair.generate());
    const provider = createLocalhostProvider({ wallet, setGlobal: false });
    const program = createProgram(provider);
    const [globalConfig] = await findGlobalConfigPda(program.programId);
    const [vault] = await findVaultPda(program.programId);
    const changeLeaf = Keypair.generate().publicKey;
    const base = {
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
    const withChange = await buildSettleIx(
      program,
      { ...base, changeLeaf },
      Buffer.alloc(356, 1),
      Buffer.alloc(144, 2),
    );
    expect(withChange.keys).toHaveLength(12);
    expect(withChange.keys[9]?.pubkey.equals(changeLeaf)).toBe(true);
    expect(withChange.keys[10]?.pubkey.equals(TOKEN_PROGRAM_ID)).toBe(true);

    const without = await buildSettleIx(
      program,
      base,
      Buffer.alloc(356, 1),
      Buffer.alloc(104, 2),
    );
    expect(without.keys).toHaveLength(12);
    expect(without.keys[9]?.pubkey.equals(program.programId)).toBe(true);
    expect(without.keys[9]?.isWritable).toBe(false);
    expect(without.keys[10]?.pubkey.equals(TOKEN_PROGRAM_ID)).toBe(true);
    expect(without.keys[11]?.pubkey.equals(SystemProgram.programId)).toBe(true);
  });
});
