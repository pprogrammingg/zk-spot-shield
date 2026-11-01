import { describe, expect, it } from "vitest";
import {
  BuyTimer,
  formatBuyWalletTiming,
  formatClock,
  formatDuration,
} from "../src/agentTiming.js";

describe("buy Token B timing", () => {
  it("formats clock and durations", () => {
    expect(formatDuration(0)).toBe("0ms");
    expect(formatDuration(412)).toBe("412ms");
    expect(formatDuration(28700)).toBe("28.7s");
    const clock = formatClock(Date.parse("2026-10-30T16:22:01.410Z"));
    expect(clock).toMatch(/^\d{2}:\d{2}:\d{2}\.\d{3}$/);
  });

  it("prints a wallet breakdown without secrets", () => {
    const startedAt = 1_000_000;
    const text = formatBuyWalletTiming({
      startedAt,
      laps: [
        { name: "402 quote", ms: 250 },
        { name: "witness", ms: 80 },
        { name: "Groth16", ms: 28_700 },
        { name: "settle", ms: 1_400 },
      ],
      walletMs: 30_430,
    });
    expect(text).toContain("Token B in wallet");
    expect(text).toContain("total 30.4s");
    expect(text).toContain("402 quote");
    expect(text).toContain("Groth16");
    expect(text).toContain("settle");
    expect(text).not.toMatch(/secret|witness\.json|keypair/i);
  });

  it("BuyTimer laps from the constructor instant", () => {
    const t = new BuyTimer(1000);
    t.mark("402 quote", 1250);
    t.mark("witness", 1330);
    t.mark("Groth16", 30_030);
    t.mark("settle", 31_430);
    const text = t.walletLines(31_430);
    expect(t.laps.map((l) => l.name)).toEqual([
      "402 quote",
      "witness",
      "Groth16",
      "settle",
    ]);
    expect(text).toContain("total 30.4s");
    expect(t.elapsed(31_430)).toBe(30_430);
  });
});
