import {
  ComputeBudgetProgram,
  PublicKey,
  SystemProgram,
  Transaction,
  TransactionInstruction,
  type Connection,
} from "@solana/web3.js";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { SETTLE_CU_LIMIT, SETTLE_DISCRIMINATOR } from "./config.ts";

export type SettleKeys = {
  payer: PublicKey;
  globalConfig: PublicKey;
  vault: PublicKey;
  cleanFundsRoot: PublicKey;
  nullifierAccount: PublicKey;
  vaultTokenA: PublicKey;
  vaultTokenB: PublicKey;
  userTokenA: PublicKey;
  userTokenB: PublicKey;
};

function borshVec(bytes: Uint8Array): Uint8Array {
  const out = new Uint8Array(4 + bytes.length);
  const view = new DataView(out.buffer);
  view.setUint32(0, bytes.length, true);
  out.set(bytes, 4);
  return out;
}

/** Anchor `settle_shielded_spot` data: disc + Borsh Vec proof + Borsh Vec journal. */
export function encodeSettleData(
  proof: Uint8Array,
  journal: Uint8Array,
): Uint8Array {
  const p = borshVec(proof);
  const j = borshVec(journal);
  const out = new Uint8Array(SETTLE_DISCRIMINATOR.length + p.length + j.length);
  out.set(SETTLE_DISCRIMINATOR, 0);
  out.set(p, SETTLE_DISCRIMINATOR.length);
  out.set(j, SETTLE_DISCRIMINATOR.length + p.length);
  return out;
}

export function buildSettleIx(
  programId: PublicKey,
  keys: SettleKeys,
  proof: Uint8Array,
  journal: Uint8Array,
): TransactionInstruction {
  return new TransactionInstruction({
    programId,
    keys: [
      { pubkey: keys.payer, isSigner: true, isWritable: true },
      { pubkey: keys.globalConfig, isSigner: false, isWritable: false },
      { pubkey: keys.vault, isSigner: false, isWritable: true },
      { pubkey: keys.cleanFundsRoot, isSigner: false, isWritable: false },
      { pubkey: keys.nullifierAccount, isSigner: false, isWritable: true },
      { pubkey: keys.vaultTokenA, isSigner: false, isWritable: true },
      { pubkey: keys.vaultTokenB, isSigner: false, isWritable: true },
      { pubkey: keys.userTokenA, isSigner: false, isWritable: true },
      { pubkey: keys.userTokenB, isSigner: false, isWritable: true },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    data: Buffer.from(encodeSettleData(proof, journal)),
  });
}

export async function buildSettleTx(
  connection: Connection,
  programId: PublicKey,
  keys: SettleKeys,
  proof: Uint8Array,
  journal: Uint8Array,
): Promise<Transaction> {
  const tx = new Transaction();
  tx.add(ComputeBudgetProgram.setComputeUnitLimit({ units: SETTLE_CU_LIMIT }));
  tx.add(buildSettleIx(programId, keys, proof, journal));
  tx.feePayer = keys.payer;
  const latest = await connection.getLatestBlockhash("confirmed");
  tx.recentBlockhash = latest.blockhash;
  return tx;
}
