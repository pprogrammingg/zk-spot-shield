/** Human token amounts ↔ on-chain base units. */

import type { Cluster } from "./cluster.ts";

export const LOCALNET_DECIMALS = 0;
export const DEVNET_DECIMALS = 6;

export function decimalsForCluster(cluster: Cluster): number {
  return cluster === "devnet" ? DEVNET_DECIMALS : LOCALNET_DECIMALS;
}

/** `33` + 6 decimals → `33000000`. `0.9` + 6 → `900000`. */
export function parseTokenAmount(human: string, decimals: number): bigint {
  const t = human.trim();
  if (decimals <= 0) {
    if (!/^[0-9]+$/.test(t)) {
      throw new Error("amount must be a positive integer");
    }
    const n = BigInt(t);
    if (n <= 0n) throw new Error("amount must be > 0");
    return n;
  }
  if (!/^[0-9]+(\.[0-9]+)?$/.test(t)) {
    throw new Error("amount must be a positive number (e.g. 33 or 0.9)");
  }
  const [wholeRaw, fracRaw = ""] = t.split(".");
  if (fracRaw.length > decimals) {
    throw new Error(`at most ${decimals} decimal places`);
  }
  const n = BigInt(wholeRaw + fracRaw.padEnd(decimals, "0"));
  if (n <= 0n) throw new Error("amount must be > 0");
  return n;
}

/** `33000000` + 6 decimals → `33`. Trailing zeros dropped. */
export function formatUiAmount(raw: string, decimals: number): string {
  if (raw === "—" || raw === "missing" || raw === "") return raw;
  let n: bigint;
  try {
    n = BigInt(raw);
  } catch {
    return raw;
  }
  if (decimals <= 0) return n.toString();
  const neg = n < 0n;
  if (neg) n = -n;
  const s = n.toString().padStart(decimals + 1, "0");
  const i = s.length - decimals;
  const whole = s.slice(0, i);
  const frac = s.slice(i).replace(/0+$/, "");
  const out = frac.length > 0 ? `${whole}.${frac}` : whole;
  return neg ? `-${out}` : out;
}

export function formatUiPair(
  a: string,
  b: string,
  decimals: number,
): string {
  return `${formatUiAmount(a, decimals)} / ${formatUiAmount(b, decimals)}`;
}
