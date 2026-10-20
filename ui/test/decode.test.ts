import { describe, expect, it } from "vitest";
import { Keypair } from "@solana/web3.js";
import {
  decodeGlobalConfig,
  decodeVaultState,
  encodeBase64,
  parseJournal,
} from "../src/decode.ts";

describe("UI account decode", () => {
  it("reads vault reserves from zero-copy body", () => {
    const disc = new Uint8Array(8);
    const body = new Uint8Array(120);
    const authority = Keypair.generate().publicKey.toBytes();
    body.set(authority, 0);
    body.fill(3, 32, 64);
    body.fill(4, 64, 96);
    new DataView(body.buffer).setBigUint64(96, 300n, true);
    new DataView(body.buffer).setBigUint64(104, 900n, true);
    body[112] = 255;
    const data = new Uint8Array(8 + 120);
    data.set(disc, 0);
    data.set(body, 8);
    const v = decodeVaultState(data);
    expect(v.reserveA).toBe(300n);
    expect(v.reserveB).toBe(900n);
    expect(v.bump).toBe(255);
  });

  it("reads pause flag on GlobalConfig", () => {
    const data = new Uint8Array(8 + 65);
    data[8 + 64] = 1;
    expect(decodeGlobalConfig(data).pauseFlag).toBe(true);
  });

  it("parses packed 104-byte journal", () => {
    const j = new Uint8Array(104);
    new DataView(j.buffer).setBigUint64(0, 100n, true);
    j.fill(3, 8, 40);
    const pub = parseJournal(j);
    expect(pub.requestedSwapAmount).toBe(100n);
    expect(() => parseJournal(j.subarray(0, 103))).toThrow();
    expect(encodeBase64(j).length).toBeGreaterThan(0);
  });
});
