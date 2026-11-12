import { describe, expect, it } from "vitest";
import {
  parseUiBuyAmount,
  pickSpendableNoteIndex,
  discoverAgentOrigin,
  fetchAgentStatus,
  postAgentBuy,
  postAgentDeposit,
  postAgentDepositB,
  postAgentMintA,
  postAgentPublicSwap,
} from "../src/agentBuy.ts";

describe("UI agent buy", () => {
  it("pickSpendableNoteIndex takes the smallest covering note", () => {
    const notes = [
      { leafIndex: 0, balance: "100", spent: false },
      { leafIndex: 1, balance: "40", spent: false },
      { leafIndex: 2, balance: "200", spent: true },
    ];
    expect(pickSpendableNoteIndex(notes, 50n)).toBe(0);
    expect(pickSpendableNoteIndex(notes, 30n)).toBe(1);
    expect(pickSpendableNoteIndex(notes, 150n)).toBeNull();
  });

  it("parseUiBuyAmount validates human amounts for the cluster decimals", () => {
    expect(parseUiBuyAmount("50")).toBe("50");
    expect(parseUiBuyAmount("1.5", 6)).toBe("1.5");
    expect(() => parseUiBuyAmount("0")).toThrow();
    expect(() => parseUiBuyAmount("1.5")).toThrow();
    expect(() => parseUiBuyAmount("")).toThrow();
  });

  it("discoverAgentOrigin skips dead ports and uses /health", async () => {
    const origin = await discoverAgentOrigin(async (input) => {
      const url = String(input);
      if (url.includes(":8790")) throw new Error("refused");
      if (url.includes("127.0.0.1:8791") && url.includes("/health")) {
        return new Response(JSON.stringify({ ok: true }), { status: 200 });
      }
      throw new Error("empty");
    });
    expect(origin).toBe("http://127.0.0.1:8791");
  });

  it("postAgentBuy maps underfunded errors", async () => {
    await expect(
      postAgentBuy("http://127.0.0.1:8790", "50", async () =>
        new Response(
          JSON.stringify({ ok: false, error: "no note covers 50. Deposit more A (CLI 2)." }),
          { status: 422 },
        ),
      ),
    ).rejects.toThrow(/no note covers 50/);
  });

  it("postAgentBuy returns public settle fields", async () => {
    const result = await postAgentBuy("http://127.0.0.1:8790", "50", async () =>
      new Response(
        JSON.stringify({
          ok: true,
          settleSig: "Sig111",
          spentIndex: 0,
          spentBalance: "333",
          changeBalance: "283",
          changeIndex: 2,
          rootSig: "Root111",
          paid: true,
        }),
        { status: 200 },
      ),
    );
    expect(result.settleSig).toBe("Sig111");
    expect(result.changeBalance).toBe("283");
  });

  it("postAgentDeposit returns public leaf fields", async () => {
    const result = await postAgentDeposit("http://127.0.0.1:8790", "111", async () =>
      new Response(
        JSON.stringify({
          ok: true,
          sig: "DepSig",
          leafIndex: 2,
          balance: "111",
          rootSig: "RootDep",
        }),
        { status: 200 },
      ),
    );
    expect(result.leafIndex).toBe(2);
    expect(result.balance).toBe("111");
  });

  it("postAgentPublicSwap returns wallet unlock fields", async () => {
    const result = await postAgentPublicSwap("http://127.0.0.1:8790", "66", async () =>
      new Response(
        JSON.stringify({
          ok: true,
          swapSig: "PubSig",
          paid: true,
          payer: "Payer111",
        }),
        { status: 200 },
      ),
    );
    expect(result.swapSig).toBe("PubSig");
    expect(result.payer).toBe("Payer111");
    expect(result.paid).toBe(true);
  });

  it("postAgentDepositB and postAgentMintA map admin fields", async () => {
    const dep = await postAgentDepositB("http://127.0.0.1:8790", "500", async () =>
      new Response(
        JSON.stringify({
          ok: true,
          sig: "DepB",
          amount: "500000000",
          reserveB: "600000000",
        }),
        { status: 200 },
      ),
    );
    expect(dep.reserveB).toBe("600000000");
    const mint = await postAgentMintA("http://127.0.0.1:8790", "50", async () =>
      new Response(
        JSON.stringify({
          ok: true,
          sig: "MintA",
          amount: "50000000",
          userTokenA: "AtaA",
        }),
        { status: 200 },
      ),
    );
    expect(mint.userTokenA).toBe("AtaA");
  });

  it("fetchAgentStatus reads notes and ATAs", async () => {
    const st = await fetchAgentStatus("http://127.0.0.1:8790", async () =>
      new Response(
        JSON.stringify({
          ok: true,
          cluster: "devnet",
          programId: "74UW",
          vaultTokenA: "VaultA",
          vaultTokenB: "VaultB",
          userTokenA: "UserA",
          userTokenB: "UserB",
          mintA: "MintA",
          mintB: "MintB",
          notes: [
            { leafIndex: 0, balance: "33", spent: false, leafPrefix: "aabbcc" },
          ],
        }),
        { status: 200 },
      ),
    );
    expect(st?.cluster).toBe("devnet");
    expect(st?.userTokenA).toBe("UserA");
    expect(st?.notes[0]?.balance).toBe("33");
  });
});
