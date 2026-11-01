import { existsSync, mkdtempSync, readFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import {
  appendLeafHex,
  emptyAgentState,
  formatNotesTable,
  loadAgentState,
  markSpent,
  noteFromParts,
  pickSpendableNote,
  publicNotesSnapshot,
  saveAgentState,
  uiNotesSnapshotPath,
} from "../src/agentStore.js";

const secret = new Uint8Array(32).fill(1);
const user = new Uint8Array(32).fill(2);
const mint = new Uint8Array(32).fill(3);
const leafA = new Uint8Array(32).fill(0xaa);
const leafB = new Uint8Array(32).fill(0xbb);

describe("agent store", () => {
  it("picks the smallest unspent note that covers the quote", () => {
    const state = emptyAgentState();
    state.notes.push(
      noteFromParts({
        secret,
        userAddress: user,
        balance: 50n,
        assetIdMint: mint,
        leaf: leafA,
        leafIndex: 0,
      }),
      noteFromParts({
        secret,
        userAddress: user,
        balance: 200n,
        assetIdMint: mint,
        leaf: leafB,
        leafIndex: 1,
      }),
    );
    expect(pickSpendableNote(state, 100n)?.leafIndex).toBe(1);
    markSpent(state, 1);
    expect(pickSpendableNote(state, 100n)).toBeUndefined();
    expect(pickSpendableNote(state, 50n)?.leafIndex).toBe(0);
    const changeLeaf = new Uint8Array(32).fill(0xcc);
    state.notes.push(
      noteFromParts({
        secret,
        userAddress: user,
        balance: 100n,
        assetIdMint: mint,
        leaf: changeLeaf,
        leafIndex: 2,
      }),
    );
    expect(pickSpendableNote(state, 100n)?.leafIndex).toBe(2);
  });

  it("round-trips gitignored JSON without a wallet field", () => {
    const dir = mkdtempSync(join(tmpdir(), "zkss-agent-"));
    const path = join(dir, "state.json");
    const state = emptyAgentState();
    const note = noteFromParts({
      secret,
      userAddress: user,
      balance: 200n,
      assetIdMint: mint,
      leaf: leafA,
      leafIndex: 0,
    });
    state.notes.push(note);
    appendLeafHex(state, note.leafHex);
    saveAgentState(state, path);
    const raw = readFileSync(path, "utf8");
    expect(raw).not.toMatch(/wallet|payer/i);
    const loaded = loadAgentState(path);
    expect(loaded.notes[0]?.balance).toBe("200");
    expect(loaded.leavesHex).toHaveLength(1);
    expect(formatNotesTable(loaded)).toContain("200");
    expect(formatNotesTable(loaded)).not.toContain(note.secretHex);
  });

  it("UI snapshot has balances and no secret", () => {
    const state = emptyAgentState();
    const note = noteFromParts({
      secret,
      userAddress: user,
      balance: 333n,
      assetIdMint: mint,
      leaf: leafA,
      leafIndex: 0,
    });
    state.notes.push(note);
    const snap = publicNotesSnapshot(state);
    expect(snap).toEqual([
      {
        leafIndex: 0,
        balance: "333",
        spent: false,
        leafPrefix: "a".repeat(12),
      },
    ]);
    expect(JSON.stringify(snap)).not.toMatch(/01010101|secret/i);
  });

  it("temp saveAgentState does not overwrite the UI snapshot", () => {
    const uiPath = uiNotesSnapshotPath();
    const before = existsSync(uiPath) ? readFileSync(uiPath, "utf8") : "";
    const dir = mkdtempSync(join(tmpdir(), "zkss-agent-"));
    const path = join(dir, "state.json");
    const state = emptyAgentState();
    state.notes.push(
      noteFromParts({
        secret,
        userAddress: user,
        balance: 999999n,
        assetIdMint: mint,
        leaf: leafA,
        leafIndex: 0,
      }),
    );
    saveAgentState(state, path);
    const after = existsSync(uiPath) ? readFileSync(uiPath, "utf8") : "";
    expect(after).toBe(before);
    expect(after).not.toContain("999999");
  });
});
