import { describe, expect, it } from "vitest";
import {
  DEMO_SESSION_KEY,
  readDemoSession,
  writeDemoSession,
} from "../src/session.ts";

describe("demo session", () => {
  it("round-trips journal so a reload can show the note", () => {
    const mem = new Map<string, string>();
    const store = {
      getItem: (k: string) => mem.get(k) ?? null,
      setItem: (k: string, v: string) => {
        mem.set(k, v);
      },
    };
    writeDemoSession(
      {
        proofB64: "cHJvb2Y=",
        journalB64: "am91cm5hbA==",
        programIdStr: "74UW993wrDasAMyBQ53PjHpYLK5vMkWbRRWFZ7kCmM3G",
        vaultTokenA: "a",
        vaultTokenB: "b",
        userTokenA: "c",
        userTokenB: "d",
        txSig: "sig",
      },
      store,
    );
    expect(mem.has(DEMO_SESSION_KEY)).toBe(true);
    const got = readDemoSession(store);
    expect(got?.journalB64).toBe("am91cm5hbA==");
    expect(got?.txSig).toBe("sig");
  });
});
