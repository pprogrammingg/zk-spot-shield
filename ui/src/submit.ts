import {
  Connection,
  Keypair,
  PublicKey,
  Transaction,
} from "@solana/web3.js";
import { encodeBase64 } from "./decode.ts";
import { buildSettleTx, type SettleKeys } from "./settle.ts";

export type RelayerIntent = {
  proof_base64: string;
  journal_base64: string;
  global_config: string;
  vault: string;
  clean_funds_root: string;
  nullifier_account: string;
  vault_token_a: string;
  vault_token_b: string;
  user_token_a: string;
  user_token_b: string;
  amount: number;
};

export function toRelayerIntent(
  keys: SettleKeys,
  proof: Uint8Array,
  journal: Uint8Array,
  amount: number,
): RelayerIntent {
  return {
    proof_base64: encodeBase64(proof),
    journal_base64: encodeBase64(journal),
    global_config: keys.globalConfig.toBase58(),
    vault: keys.vault.toBase58(),
    clean_funds_root: keys.cleanFundsRoot.toBase58(),
    nullifier_account: keys.nullifierAccount.toBase58(),
    vault_token_a: keys.vaultTokenA.toBase58(),
    vault_token_b: keys.vaultTokenB.toBase58(),
    user_token_a: keys.userTokenA.toBase58(),
    user_token_b: keys.userTokenB.toBase58(),
    amount,
  };
}

export async function submitViaRelayer(
  relayerUrl: string,
  intent: RelayerIntent,
): Promise<{ job_id: string; status: string }> {
  const intentBytes = new TextEncoder().encode(JSON.stringify(intent));
  const intent_base64 = encodeBase64(intentBytes);
  const res = await fetch(`${relayerUrl.replace(/\/$/, "")}/submit-intent`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ intent_base64 }),
  });
  if (!res.ok) {
    throw new Error(`relayer ${res.status}: ${await res.text()}`);
  }
  return (await res.json()) as { job_id: string; status: string };
}

export type WalletSigner = {
  publicKey: PublicKey;
  signTransaction: (tx: Transaction) => Promise<Transaction>;
};

export function keypairSigner(kp: Keypair): WalletSigner {
  return {
    publicKey: kp.publicKey,
    signTransaction: async (tx) => {
      tx.partialSign(kp);
      return tx;
    },
  };
}

/** Self-submit: user wallet (or localnet keypair) is the fee payer. Relayer not required. */
export async function submitMyself(
  connection: Connection,
  programId: PublicKey,
  keys: SettleKeys,
  proof: Uint8Array,
  journal: Uint8Array,
  signer: WalletSigner,
): Promise<string> {
  const tx = await buildSettleTx(connection, programId, keys, proof, journal);
  const signed = await signer.signTransaction(tx);
  return connection.sendRawTransaction(signed.serialize(), {
    skipPreflight: false,
  });
}
