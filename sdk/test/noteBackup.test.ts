import { describe, expect, it } from "vitest";
import {
  exportEncryptedNote,
  importEncryptedNote,
  type NoteBackupPlain,
} from "../src/noteBackup.js";

function sampleNote(balance: bigint, leafIndex: number): NoteBackupPlain {
  return {
    secret: new Uint8Array(32).fill(1),
    userAddress: new Uint8Array(32).fill(2),
    balance,
    assetIdMint: new Uint8Array(32).fill(3),
    leafIndex,
  };
}

describe("encrypted note backup", () => {
  it("round-trips a change note remainder", async () => {
    const note = sampleNote(900n, 1);
    const blob = await exportEncryptedNote(note, "correct horse battery");
    expect(blob.includes("correct horse")).toBe(false);
    expect(JSON.parse(blob).ct.length).toBeGreaterThan(0);

    const restored = await importEncryptedNote(blob, "correct horse battery");
    expect(restored.balance).toBe(900n);
    expect(restored.leafIndex).toBe(1);
    expect(Buffer.from(restored.secret).equals(Buffer.from(note.secret))).toBe(
      true,
    );
    expect(
      Buffer.from(restored.userAddress).equals(Buffer.from(note.userAddress)),
    ).toBe(true);
    expect(
      Buffer.from(restored.assetIdMint).equals(Buffer.from(note.assetIdMint)),
    ).toBe(true);
  });

  it("wipe-restore: in-memory note gone, blob still spends the same secret", async () => {
    let live: NoteBackupPlain | undefined = sampleNote(900n, 1);
    const blob = await exportEncryptedNote(live, "pw");
    live = undefined;
    const restored = await importEncryptedNote(blob, "pw");
    expect(live).toBeUndefined();
    expect(restored.balance).toBe(900n);
    expect(restored.secret[0]).toBe(1);
  });

  it("rejects a wrong passphrase", async () => {
    const blob = await exportEncryptedNote(sampleNote(1000n, 0), "right");
    await expect(importEncryptedNote(blob, "wrong")).rejects.toThrow(
      /decrypt failed/,
    );
  });
});
