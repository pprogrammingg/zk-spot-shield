/**
 * Client helpers for the relayer HTTP 402 shielded payer.
 * Payment credential is the settle **journal** (nullifier path), not a wallet.
 * Relayer logs quote vs journal MATCH/MISS; receipt is the nullifier.
 */

export type PaymentQuote = {
  amount: number;
  mint_hex: string;
  pay_via: string;
  hint: string;
  http_status?: number;
  unlock?: string;
  contrast?: string;
};

export type PaidResource = {
  data: string;
  nullifier_hex: string;
};

export type PayAck = {
  paid: boolean;
  nullifier_hex: string;
  amount: number;
};

export function quoteUrl(
  base: string,
  opts?: { amount?: number | string; mintHex?: string },
): string {
  const path = `${base.replace(/\/$/, "")}/402/resource`;
  const p = new URLSearchParams();
  if (opts?.amount != null) p.set("amount", String(opts.amount));
  if (opts?.mintHex) {
    const m = opts.mintHex.startsWith("0x") ? opts.mintHex : `0x${opts.mintHex}`;
    p.set("mint", m);
  }
  const q = p.toString();
  return q ? `${path}?${q}` : path;
}

export function payUrl(base: string): string {
  return `${base.replace(/\/$/, "")}/402/pay`;
}

/** Public unlock: Solana transfer/swap signature + payer pubkey (not a nullifier). */
export function payPublicUrl(base: string): string {
  return `${base.replace(/\/$/, "")}/402/pay-public`;
}

export type PayPublicAck = {
  paid: boolean;
  signature: string;
  payer: string;
  amount: number;
};

export function resourcePublicUrl(base: string, signature: string): string {
  return `${quoteUrl(base)}?signature=${encodeURIComponent(signature)}`;
}

export function resourceUrl(base: string, nullifierHex: string): string {
  const nf = nullifierHex.startsWith("0x") ? nullifierHex : `0x${nullifierHex}`;
  return `${quoteUrl(base)}?nullifier=${encodeURIComponent(nf)}`;
}

export function journalToPayBody(journal: Buffer | Uint8Array): {
  journal_base64: string;
} {
  return { journal_base64: Buffer.from(journal).toString("base64") };
}
