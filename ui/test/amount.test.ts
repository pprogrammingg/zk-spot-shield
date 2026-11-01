import { describe, expect, it } from "vitest";
import {
  DEVNET_DECIMALS,
  decimalsForCluster,
  formatUiAmount,
  formatUiPair,
  parseTokenAmount,
} from "../src/amount.ts";

describe("UI amount display", () => {
  it("Devnet 6 decimals, localnet unchanged", () => {
    expect(decimalsForCluster("devnet")).toBe(6);
    expect(decimalsForCluster("localnet")).toBe(0);
    expect(parseTokenAmount("33", DEVNET_DECIMALS)).toBe(33_000_000n);
    expect(parseTokenAmount("0.9", DEVNET_DECIMALS)).toBe(900_000n);
    expect(formatUiAmount("33000000", DEVNET_DECIMALS)).toBe("33");
    expect(formatUiAmount("900000", DEVNET_DECIMALS)).toBe("0.9");
    expect(formatUiAmount("1000033", DEVNET_DECIMALS)).toBe("1.000033");
    expect(formatUiAmount("1000", 0)).toBe("1000");
    expect(formatUiAmount("—", 6)).toBe("—");
    expect(formatUiPair("1000000", "900000", 6)).toBe("1 / 0.9");
  });
});
