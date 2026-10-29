import { describe, expect, it } from "vitest";
import { parseJournal } from "../src/decode.ts";
import { isSpendSettled, noteOldBalance } from "../src/noteStory.ts";

describe("note story", () => {
  it("100 spend + 900 change is a 1000 note", () => {
    const j = new Uint8Array(144);
    new DataView(j.buffer).setBigUint64(0, 100n, true);
    new DataView(j.buffer).setBigUint64(104, 900n, true);
    const pub = parseJournal(j);
    expect(noteOldBalance(pub)).toBe(1000n);
    expect(isSpendSettled("100", 100n)).toBe(true);
    expect(isSpendSettled("0", 100n)).toBe(false);
  });
});
