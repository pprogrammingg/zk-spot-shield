/** Parse one HELP line: 1 Buy · 2 Deposit · 3 Balance · 4 Non-Shielded · 5 Admin · EXIT. */

import { spawn } from "node:child_process";
import { parseTokenAmount } from "./tokenAmount.js";

export type AgentCommand =
  | "buy"
  | "deposit"
  | "balance"
  | "public"
  | "admin"
  | "exit"
  | "help";

export type AdminCommand = "deposit-b" | "mint-a" | "back" | "help";

export const DEFAULT_UI_URL = "http://127.0.0.1:5173";

/** Vite uses 5173, then 5174… if that port is taken. */
export const VITE_PROBE_PORTS = [5173, 5174, 5175, 5176, 5177, 5178, 5179, 5180];

/** macOS Vite often binds `[::1]` (`localhost`) and not `127.0.0.1`. */
export const VITE_PROBE_HOSTS = ["localhost", "127.0.0.1", "[::1]"];

export function uiUrl(): string {
  return process.env.UI_URL ?? DEFAULT_UI_URL;
}

/** Existing swap UI, matching agent cluster, On-chain data open. */
export function balanceUiUrl(
  origin: string = uiUrl(),
  cluster: "localnet" | "devnet" = "localnet",
): string {
  const base = origin.replace(/\/$/, "");
  return `${base}/?cluster=${cluster}#on-chain`;
}

/** First origin that answers (env `UI_URL`, else probe Vite ports). */
export async function discoverUiOrigin(
  fetchFn: typeof fetch = fetch,
): Promise<string | null> {
  if (process.env.UI_URL) {
    return process.env.UI_URL.replace(/\/$/, "");
  }
  for (const port of VITE_PROBE_PORTS) {
    for (const host of VITE_PROBE_HOSTS) {
      const origin = `http://${host}:${port}`;
      try {
        const res = await fetchFn(origin + "/", {
          signal: AbortSignal.timeout(1500),
        });
        const text = await res.text();
        if (text.includes("@vite/client") || text.includes("ZK Spot Shield")) {
          return origin;
        }
      } catch {
        /* try next host/port */
      }
    }
  }
  return null;
}

/** Open the swap UI in the default browser (`open` on macOS). */
export function openBrowser(url: string): void {
  const cmd =
    process.platform === "darwin"
      ? "open"
      : process.platform === "win32"
        ? "cmd"
        : "xdg-open";
  const args = process.platform === "win32" ? ["/c", "start", "", url] : [url];
  spawn(cmd, args, { stdio: "ignore", detached: true }).unref();
}

/** Collapse "baalancce" → "balance". */
export function collapseRepeats(s: string): string {
  return s.replace(/(.)\1+/g, "$1");
}

export function parseAgentCommand(line: string): AgentCommand | null {
  const t = line.trim().toLowerCase();
  if (t === "" || t === "help" || t === "h" || t === "?") return "help";
  if (t === "1" || t === "buy") return "buy";
  if (t === "2" || t === "deposit") return "deposit";
  const collapsed = collapseRepeats(t);
  if (
    t === "3" ||
    t === "balance" ||
    t === "balances" ||
    t === "blance" ||
    t === "bal" ||
    collapsed === "balance" ||
    collapsed === "balances" ||
    collapsed === "blance"
  ) {
    return "balance";
  }
  if (
    t === "4" ||
    t === "public" ||
    t === "nonshielded" ||
    t === "non-shielded" ||
    t === "swap"
  ) {
    return "public";
  }
  if (t === "5" || t === "admin") return "admin";
  if (t === "exit" || t === "quit" || t === "q") return "exit";
  return null;
}

export function parseAdminCommand(line: string): AdminCommand | null {
  const t = line.trim().toLowerCase();
  if (t === "" || t === "help" || t === "h" || t === "?") return "help";
  if (t === "1" || t === "deposit-b" || t === "depositb") return "deposit-b";
  if (t === "2" || t === "mint-a" || t === "minta" || t === "mint") return "mint-a";
  return null;
}

/** `B` / `back` from a submenu returns to HELP. Not `buy` — that stays `1`. */
export function isBackToMenu(line: string): boolean {
  const t = line.trim().toLowerCase();
  return t === "b" || t === "back";
}

export const BACK_HINT = "(B to go back)";

export function parseAmount(line: string, decimals: number = 0): bigint {
  return parseTokenAmount(line, decimals);
}

export function parseYesNo(line: string): boolean {
  const t = line.trim().toLowerCase();
  return t === "y" || t === "yes";
}

export const AGENT_HELP = `1  Buy Token B (shielded)
2  Deposit Token A
3  Balance
4  Non-Shielded Swap A→B
5  Admin
EXIT`;

export const ADMIN_HELP = `Admin (mint authority wallet):
1  Deposit B to vault (mint + deposit)
2  Mint A to user wallet
B  back`;
