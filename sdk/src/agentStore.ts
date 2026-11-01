/**
 * Localnet agent note store. Secrets live in a gitignored JSON file — not a wallet.
 */

import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { bytes32ToHex, hexToBytes32 } from "./merkle.js";
import type { DepositNote } from "./depositTree.js";
import { formatTokenAmount } from "./tokenAmount.js";

const HERE = dirname(fileURLToPath(import.meta.url));
export const SDK_ROOT = join(HERE, "..");

export type StoredNote = {
  id: string;
  secretHex: string;
  userAddressHex: string;
  balance: string;
  assetIdMintHex: string;
  leafHex: string;
  leafIndex: number;
  spent: boolean;
};

export type AgentState = {
  notes: StoredNote[];
  leavesHex: string[];
};

export function defaultStatePath(): string {
  if (process.env.AGENT_STATE) return process.env.AGENT_STATE;
  const cluster = (process.env.AGENT_CLUSTER ?? "localnet").toLowerCase();
  return join(
    SDK_ROOT,
    cluster === "devnet" ? ".agent-state-devnet.json" : ".agent-state.json",
  );
}

export function emptyAgentState(): AgentState {
  return { notes: [], leavesHex: [] };
}

export function loadAgentState(path: string = defaultStatePath()): AgentState {
  if (!existsSync(path)) return emptyAgentState();
  const raw = JSON.parse(readFileSync(path, "utf8")) as AgentState;
  if (!Array.isArray(raw.notes) || !Array.isArray(raw.leavesHex)) {
    throw new Error(`invalid agent state at ${path}`);
  }
  return raw;
}

export type PublicNoteView = {
  leafIndex: number;
  balance: string;
  spent: boolean;
  leafPrefix: string;
};

/** Secret-free note list for the swap UI (`ui/public/demo/agent-notes.json`). */
export function publicNotesSnapshot(state: AgentState): PublicNoteView[] {
  return state.notes.map((n) => ({
    leafIndex: n.leafIndex,
    balance: n.balance,
    spent: n.spent,
    leafPrefix: n.leafHex.replace(/^0x/, "").slice(0, 12),
  }));
}

export function uiNotesSnapshotPath(): string {
  return join(SDK_ROOT, "../ui/public/demo/agent-notes.json");
}

export function publishUiNotes(state: AgentState): void {
  const path = uiNotesSnapshotPath();
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(
    path,
    JSON.stringify({ notes: publicNotesSnapshot(state) }, null, 2) + "\n",
  );
}

export function saveAgentState(
  state: AgentState,
  path: string = defaultStatePath(),
): void {
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, JSON.stringify(state, null, 2) + "\n");
  // Tests pass a temp path — do not clobber the Vite `public/demo` snapshot.
  if (path === defaultStatePath()) {
    publishUiNotes(state);
  }
}

export function storedToDepositNote(note: StoredNote): DepositNote {
  return {
    secret: hexToBytes32(note.secretHex),
    userAddress: hexToBytes32(note.userAddressHex),
    balance: BigInt(note.balance),
    assetIdMint: hexToBytes32(note.assetIdMintHex),
  };
}

export function noteFromParts(args: {
  secret: Uint8Array;
  userAddress: Uint8Array;
  balance: bigint;
  assetIdMint: Uint8Array;
  leaf: Uint8Array;
  leafIndex: number;
  spent?: boolean;
}): StoredNote {
  const leafHex = bytes32ToHex(args.leaf);
  return {
    id: leafHex.slice(0, 10),
    secretHex: bytes32ToHex(args.secret),
    userAddressHex: bytes32ToHex(args.userAddress),
    balance: args.balance.toString(),
    assetIdMintHex: bytes32ToHex(args.assetIdMint),
    leafHex,
    leafIndex: args.leafIndex,
    spent: args.spent === true,
  };
}

/** Smallest unspent note whose balance covers the quote (change note before a larger deposit). */
export function pickSpendableNote(
  state: AgentState,
  quote: bigint,
): StoredNote | undefined {
  if (quote <= 0n) return undefined;
  const candidates = state.notes.filter(
    (n) => !n.spent && BigInt(n.balance) >= quote,
  );
  if (candidates.length === 0) return undefined;
  return candidates.reduce((best, n) =>
    BigInt(n.balance) < BigInt(best.balance) ? n : best,
  );
}

export function formatNotesTable(
  state: AgentState,
  decimals: number = 0,
): string {
  if (state.notes.length === 0) {
    return "(no notes — deposit A with 2)";
  }
  const rows = ["idx  leaf            A      spent"];
  for (const n of state.notes) {
    const leaf = n.leafHex.replace(/^0x/, "").slice(0, 12);
    const spent = n.spent ? "yes" : "no";
    const amt = formatTokenAmount(n.balance, decimals);
    rows.push(
      `${String(n.leafIndex).padEnd(4)} ${leaf}…  ${amt.padStart(12)}  ${spent}`,
    );
  }
  return rows.join("\n");
}

export function appendLeafHex(state: AgentState, leafHex: string): void {
  state.leavesHex.push(leafHex.startsWith("0x") ? leafHex : `0x${leafHex}`);
}

export function markSpent(state: AgentState, leafIndex: number): StoredNote {
  const note = state.notes.find((n) => n.leafIndex === leafIndex && !n.spent);
  if (!note) throw new Error(`no unspent note at leafIndex ${leafIndex}`);
  note.spent = true;
  return note;
}
