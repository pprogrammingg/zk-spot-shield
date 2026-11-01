import { describe, expect, it } from "vitest";
import { decodeSplAmount, encodeBase64, parseJournal } from "../src/decode.ts";
import { demoAssetUrl, loadAgentNotesPublic, loadDemoPack } from "../src/demo.ts";

describe("localnet demo pack", () => {
  it("reads SPL token amounts at offset 64", () => {
    const data = new Uint8Array(165);
    new DataView(data.buffer).setBigUint64(64, 100n, true);
    expect(decodeSplAmount(data)).toBe(100n);
  });

  it("demoAssetUrl is under BASE_URL/demo", () => {
    expect(demoAssetUrl("accounts.json")).toMatch(/demo\/accounts\.json$/);
  });

  it("loadDemoPack decodes 356+144 fixtures", async () => {
    const proof = new Uint8Array(356).fill(7);
    const journal = new Uint8Array(144);
    journal.set(new Uint8Array(new BigUint64Array([100n]).buffer), 0);
    const accounts = {
      programId: "74UW993wrDasAMyBQ53PjHpYLK5vMkWbRRWFZ7kCmM3G",
      authority: "11111111111111111111111111111111",
      vaultTokenA: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
      vaultTokenB: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
      userTokenA: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
      userTokenB: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
    };
    const fetchFn: typeof fetch = async (input) => {
      const url = String(input);
      if (url.endsWith("accounts.json")) {
        return new Response(JSON.stringify(accounts), { status: 200 });
      }
      if (url.endsWith("groth16.bin")) {
        return new Response(proof, { status: 200 });
      }
      if (url.endsWith("journal.bin")) {
        return new Response(journal, { status: 200 });
      }
      return new Response("missing", { status: 404 });
    };
    const pack = await loadDemoPack(fetchFn);
    expect(pack.proofB64).toBe(encodeBase64(proof));
    expect(parseJournal(journal).requestedSwapAmount).toBe(100n);
    expect(pack.accounts.programId).toBe(accounts.programId);
  });

  it("loadAgentNotesPublic ignores secrets and 404", async () => {
    const empty = await loadAgentNotesPublic(async () => new Response("", { status: 404 }));
    expect(empty).toEqual([]);
    const notes = await loadAgentNotesPublic(async () =>
      new Response(
        JSON.stringify({
          notes: [
            { leafIndex: 0, balance: "333", spent: false, leafPrefix: "2308b649d7e1" },
          ],
        }),
        { status: 200 },
      ),
    );
    expect(notes).toEqual([
      { leafIndex: 0, balance: "333", spent: false, leafPrefix: "2308b649d7e1" },
    ]);
  });
});
