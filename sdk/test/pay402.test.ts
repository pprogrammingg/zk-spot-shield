import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import { journalToPayBody, parseJournalPublic, resourceUrl } from "../src/index.js";

describe("Day 50 402 client helpers", () => {
  it("pay body is journal-only (no wallet)", () => {
    const journal = readFileSync(
      join(
        dirname(fileURLToPath(import.meta.url)),
        "../../zk-circuit/fixtures/happy/journal.bin",
      ),
    );
    const pub = parseJournalPublic(journal);
    expect(pub.requestedSwapAmount).toBe(100n);
    const body = journalToPayBody(journal);
    expect(Object.keys(body)).toEqual(["journal_base64"]);
    expect(body.journal_base64).not.toMatch(/wallet|payer|secret/i);
    const url = resourceUrl("http://127.0.0.1:8787", pub.nullifier.toString("hex"));
    expect(url).toContain("nullifier=0x");
    expect(url).not.toContain("wallet");
  });
});
