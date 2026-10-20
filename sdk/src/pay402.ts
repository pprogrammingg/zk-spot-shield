/**
 * Day 50: client helpers for the relayer HTTP 402 shielded payer.
 * Payment credential is the settle **journal** (nullifier path), not a wallet.
 */

export type PaymentQuote = {
  amount: number;
  mint_hex: string;
  pay_via: string;
  hint: string;
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

export function quoteUrl(base: string): string {
  return `${base.replace(/\/$/, "")}/402/resource`;
}

export function payUrl(base: string): string {
  return `${base.replace(/\/$/, "")}/402/pay`;
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
