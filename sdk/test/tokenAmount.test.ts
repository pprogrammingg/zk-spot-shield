import { describe, expect, it } from "vitest";
import {
  DEVNET_DECIMALS,
  decimalsForCluster,
  formatTokenAmount,
  parseTokenAmount,
} from "../src/tokenAmount.js";

describe("token amounts", () => {
  it("Devnet 6 decimals, localnet unchanged", () => {
    expect(decimalsForCluster("devnet")).toBe(6);
    expect(decimalsForCluster("localnet")).toBe(0);
    expect(parseTokenAmount("33", DEVNET_DECIMALS)).toBe(33_000_000n);
    expect(parseTokenAmount("0.9", DEVNET_DECIMALS)).toBe(900_000n);
    expect(parseTokenAmount("1.000033", DEVNET_DECIMALS)).toBe(1_000_033n);
    expect(parseTokenAmount("33", 0)).toBe(33n);
    expect(() => parseTokenAmount("0.9", 0)).toThrow(/integer/);
    expect(formatTokenAmount(33_000_000n, DEVNET_DECIMALS)).toBe("33");
    expect(formatTokenAmount("900000", DEVNET_DECIMALS)).toBe("0.9");
    expect(formatTokenAmount("1000033", DEVNET_DECIMALS)).toBe("1.000033");
    expect(formatTokenAmount("33", 0)).toBe("33");
  });
});
