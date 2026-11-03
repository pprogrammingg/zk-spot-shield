import { describe, expect, it } from "vitest";
import {
  AGENT_HELP,
  BACK_HINT,
  DEFAULT_UI_URL,
  balanceUiUrl,
  isBackToMenu,
  parseAgentCommand,
  parseAmount,
  parseYesNo,
  uiUrl,
} from "../src/agentCmd.js";

describe("agent HELP parser", () => {
  it("maps 1/2/3/4/EXIT", () => {
    expect(parseAgentCommand("1")).toBe("buy");
    expect(parseAgentCommand("2")).toBe("deposit");
    expect(parseAgentCommand("3")).toBe("balance");
    expect(parseAgentCommand("4")).toBe("public");
    expect(parseAgentCommand("non-shielded")).toBe("public");
    expect(parseAgentCommand("blance")).toBe("balance");
    expect(parseAgentCommand("balances")).toBe("balance");
    expect(parseAgentCommand("baalancce")).toBe("balance");
    expect(parseAgentCommand("EXIT")).toBe("exit");
    expect(parseAgentCommand("help")).toBe("help");
    expect(parseAgentCommand("nope")).toBeNull();
    expect(parseAgentCommand("b")).toBeNull();
  });

  it("B / back leaves a submenu", () => {
    expect(isBackToMenu("B")).toBe(true);
    expect(isBackToMenu("b")).toBe(true);
    expect(isBackToMenu("back")).toBe(true);
    expect(isBackToMenu("1")).toBe(false);
    expect(isBackToMenu("200")).toBe(false);
    expect(BACK_HINT).toBe("(B to go back)");
  });

  it("parses amounts and y/N", () => {
    expect(parseAmount("200")).toBe(200n);
    expect(() => parseAmount("0")).toThrow();
    expect(() => parseAmount("1.5")).toThrow();
    expect(parseAmount("33", 6)).toBe(33_000_000n);
    expect(parseAmount("0.9", 6)).toBe(900_000n);
    expect(parseYesNo("y")).toBe(true);
    expect(parseYesNo("yes")).toBe(true);
    expect(parseYesNo("n")).toBe(false);
    expect(parseYesNo("")).toBe(false);
  });

  it("HELP is options and titles only", () => {
    expect(AGENT_HELP).toBe(
      [
        "1  Buy Token B (shielded)",
        "2  Deposit Token A",
        "3  Balance",
        "4  Non-Shielded Swap A→B",
        "EXIT",
      ].join("\n"),
    );
    expect(AGENT_HELP).not.toMatch(/\$PROVE|merchant|Groth16|wallet/i);
    expect(uiUrl()).toBe(DEFAULT_UI_URL);
    expect(balanceUiUrl()).toContain("cluster=localnet");
    expect(balanceUiUrl()).toContain("#on-chain");
    expect(balanceUiUrl("http://localhost:5174")).toContain("localhost:5174");
    expect(balanceUiUrl("http://127.0.0.1:5173", "devnet")).toContain(
      "cluster=devnet",
    );
    expect(balanceUiUrl("http://127.0.0.1:5173", "devnet")).toContain(
      "#on-chain",
    );
  });

  it("discoverUiOrigin skips dead 5173 and uses a live Vite port", async () => {
    const { discoverUiOrigin } = await import("../src/agentCmd.js");
    const origin = await discoverUiOrigin(async (input) => {
      const url = String(input);
      if (url.includes(":5173")) {
        throw new Error("connection refused");
      }
      if (url.includes("localhost:5174")) {
        return new Response(`<script src="/@vite/client"></script>`, {
          status: 200,
        });
      }
      throw new Error("empty");
    });
    expect(origin).toBe("http://localhost:5174");
  });
});
