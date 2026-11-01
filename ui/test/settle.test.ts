import { describe, expect, it } from "vitest";
import { Keypair, PublicKey } from "@solana/web3.js";
import { encodeSettleData, buildSettleIx } from "../src/settle.ts";
import { DEFAULT_PROGRAM_ID } from "../src/config.ts";
import { toRelayerIntent } from "../src/submit.ts";

describe("UI settle encoding", () => {
  it("packs disc + proof vec + 144-byte journal to 516 bytes", () => {
    const proof = new Uint8Array(356).fill(1);
    const journal = new Uint8Array(144).fill(2);
    const data = encodeSettleData(proof, journal);
    expect(data.length).toBe(8 + 4 + 356 + 4 + 144);
    expect([...data.slice(0, 8)]).toEqual([233, 96, 13, 60, 148, 116, 38, 171]);
  });

  it("self-submit ix has fee payer as signer; relayer intent is JSON-shaped", () => {
    const payer = Keypair.generate().publicKey;
    const changeLeaf = Keypair.generate().publicKey;
    const keys = {
      payer,
      globalConfig: Keypair.generate().publicKey,
      vault: Keypair.generate().publicKey,
      cleanFundsRoot: Keypair.generate().publicKey,
      nullifierAccount: Keypair.generate().publicKey,
      vaultTokenA: Keypair.generate().publicKey,
      vaultTokenB: Keypair.generate().publicKey,
      userTokenA: Keypair.generate().publicKey,
      userTokenB: Keypair.generate().publicKey,
      changeLeaf,
    };
    const ix = buildSettleIx(
      new PublicKey(DEFAULT_PROGRAM_ID),
      keys,
      new Uint8Array(356),
      new Uint8Array(144),
    );
    expect(ix.keys[0]?.isSigner).toBe(true);
    expect(ix.keys[0]?.pubkey.equals(payer)).toBe(true);
    expect(ix.keys).toHaveLength(12);
    expect(ix.keys[9]?.pubkey.equals(changeLeaf)).toBe(true);
    const spendAll = buildSettleIx(
      new PublicKey(DEFAULT_PROGRAM_ID),
      { ...keys, changeLeaf: undefined },
      new Uint8Array(356),
      new Uint8Array(104),
    );
    expect(spendAll.keys).toHaveLength(12);
    expect(spendAll.keys[9]?.pubkey.equals(new PublicKey(DEFAULT_PROGRAM_ID))).toBe(
      true,
    );
    expect(spendAll.keys[9]?.isWritable).toBe(false);
    const intent = toRelayerIntent(keys, new Uint8Array(8), new Uint8Array(8), 100);
    expect(intent.amount).toBe(100);
    expect(intent.user_token_a).toBe(keys.userTokenA.toBase58());
  });
});
